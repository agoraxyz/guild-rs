use libloading::{Library, Symbol};
use reqwest::Client;

use std::collections::HashMap;
use std::path::Path;

const CALL_ONE: &[u8] = b"call_one";

pub type Prefix = [u8; 8];
pub type Scalar = f64;
pub type Error = String;
pub type CallOneResult = Result<Scalar, Error>;
pub type CallOne = extern "C" fn(CallOneInput) -> CallOneResult;
//type CallBatch = extern "C" fn(CallBatchInput) -> CallBatchResult;

pub struct CallOneInput<'a> {
    pub client: &'a Client,
    pub user: &'a str,
    pub secrets: &'a str,
    pub metadata: &'a [u8],
}

pub struct PluginManager {
    plugins: HashMap<Prefix, Library>
}

impl PluginManager {
    pub fn load(path_map: HashMap<Prefix, &Path>) -> Result<Self, String> {
        let mut plugins = HashMap::new();
        for (prefix, path) in path_map.into_iter() {
            let library = unsafe { Library::new(path) }.map_err(|e| e.to_string())?;
            plugins.insert(prefix, library);
        }

        Ok(Self { plugins })
    }

    pub fn call_one(&self, prefix: Prefix, input: CallOneInput) -> CallOneResult {
        let plugin = self.plugins.get(&prefix).ok_or(String::from("no such prefix"))?;
        let call_one: Symbol<'_, CallOne> = unsafe { plugin.get(CALL_ONE) }.map_err(|e| e.to_string())?;
        call_one(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
