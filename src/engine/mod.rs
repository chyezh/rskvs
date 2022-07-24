use super::{Error, Result};

mod store;

pub trait Engine {
    fn get(&self, key: &String) -> Result<Option<String>>;

    fn set(&mut self, key: &String, val: &String) -> Result<()>;

    fn del(&mut self, key: &String) -> Result<()>;
}
