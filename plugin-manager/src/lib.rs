use guild_common::Scalar;
use libloading::{Library, Symbol};
pub use reqwest::Client;

use std::collections::HashMap;
use std::path::Path;

pub type Prefix = u64;
pub type CallOneResult = Result<Vec<Scalar>, anyhow::Error>;
pub type CallOne = extern "C" fn(CallOneInput) -> CallOneResult;
//type CallBatch = extern "C" fn(CallBatchInput) -> CallBatchResult;

pub struct CallOneInput<'a> {
    pub client: Client,
    pub user: &'a [String],
    pub serialized_secrets: Vec<u8>,
    pub serialized_metadata: Vec<u8>,
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

    pub fn insert(&mut self, prefix: Prefix, path: &Path) -> Result<(), anyhow::Error> {
        let library = unsafe { Library::new(path) }?;
        self.plugins.insert(prefix, library);
        Ok(())
    }

    fn symbol<'a, T>(
        &'a self,
        prefix: Prefix,
        name: &[u8],
    ) -> Result<Symbol<'a, T>, anyhow::Error> {
        let plugin = self
            .plugins
            .get(&prefix)
            .ok_or(anyhow::anyhow!("no such prefix"))?;
        unsafe { plugin.get(name) }.map_err(|e| anyhow::anyhow!(e))
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
        pub fn name(&self, prefix: Prefix) -> Result<String, anyhow::Error> {
            let dynamic_call: TestCall = *self.symbol(prefix, b"name")?;
            Ok(dynamic_call())
        }
    }

    #[test]
    fn load_test_libraries() {
        let client = Client::new();
        let dummy_input = CallOneInput {
            client: client,
            user: &[String::from("")],
            serialized_secrets: Vec::new(),
            serialized_metadata: Vec::new(),
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
