use crate::{KvsError, KvsErrorKind, Result};
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

pub struct KvStore {
    // log
    log: Log<File, File>,
}

impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let log = Log::new_from_dir(path.into())?;
        Ok(KvStore { log: log })
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        // write to log
        let cmd = Command::set(key, value);
        let pos = self.log.writer.pos;
        serde_json::to_writer(&mut self.log.writer, &cmd)?;
        self.log.writer.flush()?;

        if let Command::Set { key, .. } = cmd {
            self.log.index.insert(
                key,
                CommandPos {
                    file_id: self.log.current_file_id,
                    pos: pos,
                    len: self.log.writer.pos - pos,
                },
            );
        }

        if self.log.uncompact_len > 100 {
            self.log.compact()?;
        }

        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(pos) = self.log.index.get(&key) {
            let reader = self.log.readers.get_mut(&pos.file_id).unwrap();
            reader.seek(SeekFrom::Start(pos.pos))?;

            let taker = reader.take(pos.len);
            if let Command::Set { value, .. } = serde_json::from_reader(taker)? {
                Ok(Some(value))
            } else {
                Err(KvsError::from(KvsErrorKind::UnexpectedCommandType))
            }
        } else {
            Ok(None)
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        // write to log
        let cmd = Command::remove(key);
        serde_json::to_writer(&mut self.log.writer, &cmd)?;
        self.log.writer.flush()?;

        if let Command::Remove { key, .. } = cmd {
            if let None = self.log.index.remove(&key) {
                return Err(KvsError::from(KvsErrorKind::KeyNotFound));
            }
        }

        if self.log.uncompact_len > 100 {
            self.log.compact()?;
        }

        Ok(())
    }
}

struct CommandPos {
    file_id: u64,
    pos: u64,
    len: u64,
}

// Command represent single kv operation
#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

impl Command {
    // set is the construction for Command::Set
    fn set(key: String, value: String) -> Command {
        Command::Set {
            key: key,
            value: value,
        }
    }

    // remove is the construction for Command::remove
    fn remove(key: String) -> Command {
        Command::Remove { key: key }
    }
}

struct Log<R: Read + Seek, W: Write + Seek> {
    path: PathBuf,

    readers: HashMap<u64, LogReader<R>>,

    writer: LogWriter<W>,

    index: BTreeMap<String, CommandPos>,

    current_file_id: u64,

    uncompact_len: u64,
}

fn load_index(
    file_id: u64,
    reader: &mut LogReader<File>,
    index: &mut BTreeMap<String, CommandPos>,
) -> Result<u64> {
    let mut uncompact_len = 0;
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();

    while let Some(cmd) = stream.next() {
        let new_pos = stream.byte_offset() as u64;
        match cmd? {
            Command::Set { key, .. } => {
                if let Some(old_pos) = index.insert(
                    key,
                    CommandPos {
                        file_id: file_id,
                        pos: pos,
                        len: new_pos - pos,
                    },
                ) {
                    uncompact_len += old_pos.len;
                }
            }
            Command::Remove { key, .. } => {
                if let Some(old_pos) = index.remove(&key) {
                    uncompact_len += old_pos.len;
                }
                uncompact_len += new_pos - pos;
            }
        }
        pos = new_pos;
    }
    Ok(uncompact_len)
}

fn open_log(
    path: &Path,
    file_id: u64,
    readers: &mut HashMap<u64, LogReader<File>>,
) -> Result<LogWriter<File>> {
    let writer = LogWriter::new(
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(log_file_name(path, file_id))?,
    );
    let reader = LogReader::new(File::open(log_file_name(path, file_id))?)?;
    readers.insert(file_id, reader);

    writer
}

fn log_file_name(path: &Path, file_id: u64) -> PathBuf {
    path.join(format!("{}.log", file_id))
}

impl Log<File, File> {
    fn new_from_dir(path: PathBuf) -> Result<Self> {
        fs::create_dir_all(&path)?;
        let mut file_id_list: Vec<u64> = fs::read_dir(&path)?
            .flat_map(|res| -> Result<_> { Ok(res?.path()) })
            .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
            .flat_map(|path| {
                path.file_name()
                    .and_then(OsStr::to_str)
                    .map(|s| s.trim_end_matches(".log"))
                    .map(str::parse::<u64>)
            })
            .flatten()
            .collect();
        file_id_list.sort_unstable();

        let mut readers: HashMap<u64, LogReader<File>> = HashMap::new();
        let mut index = BTreeMap::new();
        let mut uncompact_len: u64 = 0;

        for &file_id in &file_id_list {
            let mut reader = LogReader::new(File::open(path.join(format!("{}.log", file_id)))?)?;
            uncompact_len += load_index(file_id, &mut reader, &mut index)?;
            readers.insert(file_id, reader);
        }

        let current_file_id = file_id_list.last().unwrap_or(&0) + 1;
        let writer = open_log(&path, current_file_id, &mut readers)?;

        Ok(Log {
            path: path,
            readers: readers,
            writer: writer,
            index: index,
            current_file_id: current_file_id,
            uncompact_len: uncompact_len,
        })
    }

    fn compact(&mut self) -> Result<()> {
        let new_writer_file_id = self.current_file_id + 2;
        let compact_file_id = self.current_file_id + 1;
        self.writer = open_log(self.path.as_path(), new_writer_file_id, &mut self.readers)?;

        let mut compact_writer = open_log(self.path.as_path(), compact_file_id, &mut self.readers)?;

        let mut new_pos = 0;
        for cmd_pos in &mut self.index.values_mut() {
            let reader = self.readers.get_mut(&cmd_pos.file_id).unwrap();
            if reader.pos != cmd_pos.pos {
                reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            }

            let mut entry_reader = reader.take(cmd_pos.len);
            let len = io::copy(&mut entry_reader, &mut compact_writer)?;
            *cmd_pos = CommandPos {
                file_id: compact_file_id,
                pos: new_pos,
                len: len,
            }
            .into();
            new_pos += len;
        }
        compact_writer.flush()?;

        let need_clear_file_id: Vec<_> = self
            .readers
            .keys()
            .filter(|&&file_id| file_id < compact_file_id)
            .cloned()
            .collect();
        for file_id in need_clear_file_id {
            self.readers.remove(&file_id);
            fs::remove_file(log_file_name(self.path.as_path(), file_id))?;
        }
        self.current_file_id = new_writer_file_id;
        self.uncompact_len = 0;

        Ok(())
    }
}

struct LogReader<R: Read + Seek> {
    reader: R,
    pos: u64,
}

impl<R: Read + Seek> LogReader<R> {
    fn new(mut reader: R) -> Result<Self> {
        let pos = reader.seek(SeekFrom::Current(0))?;
        Ok(LogReader {
            reader: reader,
            pos: pos,
        })
    }
}

impl<R: Read + Seek> Read for LogReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

impl<R: Read + Seek> Seek for LogReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }
}

struct LogWriter<W: Write + Seek> {
    writer: W, // underlying wrtier, TODO: BufWrite optimized
    pos: u64,  // write offset
}

impl<W: Write + Seek> LogWriter<W> {
    fn new(mut writer: W) -> Result<Self> {
        let pos = writer.seek(SeekFrom::Current(0))?;
        Ok(LogWriter {
            writer: writer,
            pos: pos,
        })
    }
}

impl<W: Write + Seek> Write for LogWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> Seek for LogWriter<W> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}
