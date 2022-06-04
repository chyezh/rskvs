use clap::{Parser, Subcommand};
use rskvs::{KvStore, Result};

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    set { key: String, value: String },

    get { key: String },

    rm { key: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut store = KvStore::open("log")?;

    match cli.commands {
        Commands::set { key, value } => store.set(key, value),
        Commands::get { key } => match store.get(key) {
            Ok(Some(result)) => {
                println!("{}", result);
                Ok(())
            }
            Ok(None) => {
                println!("{}", "nil");
                Ok(())
            }
            Err(e) => Err(e),
        },
        Commands::rm { key } => store.remove(key),
    }
}
