use libloading::{Library, Symbol};
use reqwest::Client;

use std::collections::HashMap;
use std::path::Path;

pub type Prefix = u64;
pub type Scalar = f64;
pub type Error = String;
pub type CallOneResult = Result<Scalar, Error>;
pub type CallOne = extern "C" fn(CallOneInput) -> CallOneResult;
//type CallBatch = extern "C" fn(CallBatchInput) -> CallBatchResult;

pub struct CallOneInput<'a> {
    pub client: &'a Client,
    pub user: &'a str,
    pub serialized_secrets: &'a [u8],
    pub serialized_metadata: &'a [u8],
}

#[derive(Default)]
pub struct PluginManager {
    plugins: HashMap<Prefix, Library>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn insert(&mut self, prefix: Prefix, path: &Path) -> Result<(), String> {
        let library = unsafe { Library::new(path) }.map_err(|e| e.to_string())?;
        self.plugins.insert(prefix, library);
        Ok(())
    }

    fn symbol<'a, T>(&'a self, prefix: Prefix, name: &[u8]) -> Result<Symbol<'a, T>, String> {
        let plugin = self
            .plugins
            .get(&prefix)
            .ok_or(String::from("no such prefix"))?;
        unsafe { plugin.get(name) }.map_err(|e| e.to_string())
    }

    pub fn call_one(&self, prefix: Prefix, input: CallOneInput) -> CallOneResult {
        let dynamic_call: CallOne = *self.symbol(prefix, b"call_one")?;
        dynamic_call(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    type TestCall = extern "C" fn() -> String;

    impl PluginManager {
        pub fn name(&self, prefix: Prefix) -> Result<String, String> {
            let dynamic_call: TestCall = *self.symbol(prefix, b"name")?;
            Ok(dynamic_call())
        }
    }

    #[test]
    fn load_test_libraries() {
        let client = Client::new();
        let dummy_input = CallOneInput {
            client: &client,
            user: "",
            serialized_secrets: b"",
            serialized_metadata: b"",
        };

        let path_map = vec![
            (0, Path::new("./plugins/libtest_lib_a.module")),
            (1, Path::new("./plugins/libtest_lib_b.module")),
        ];

        let mut plugin_manager = PluginManager::new();
        for (prefix, path) in path_map {
            plugin_manager.insert(prefix, path).unwrap();
        }

        assert_eq!(plugin_manager.name(0).unwrap(), "test-lib-a");
        assert_eq!(plugin_manager.name(1).unwrap(), "test-lib-b");

        assert!(plugin_manager
            .insert(2, Path::new("nonexistent/path"))
            .is_err());
        assert!(plugin_manager.call_one(0, dummy_input).is_err());
    }
}
