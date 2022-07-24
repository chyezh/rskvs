use std::collections::HashMap;

use super::{Engine, Result};

pub struct Store {
    map: HashMap<String, String>,
}

impl Engine for Store {
    fn get(&self, key: &String) -> Result<Option<String>> {
        Ok(self.map.get(key).map(|s| s.clone()))
    }

    fn set(&mut self, key: &String, val: &String) -> Result<()> {
        self.map.insert(key.clone(), val.clone());
        Ok(())
    }

    fn del(&mut self, key: &String) -> Result<()> {
        self.map.remove(key);
        Ok(())
    }
}
