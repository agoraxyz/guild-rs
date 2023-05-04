use guild_common::Scalar;
pub use redis;

use libloading::{Library, Symbol};
use redis::Commands;
pub use reqwest::Client;
use serde::Serialize;
use serde_cbor::to_vec as cbor_serialize;

use std::path::Path;

pub type Prefix = u64;
pub type CallOneResult = Result<Vec<Scalar>, anyhow::Error>;
pub type CallOne = extern "C" fn(CallOneInput) -> CallOneResult;
//type CallBatch = extern "C" fn(CallBatchInput) -> CallBatchResult;

pub struct CallOneInput<'a> {
    pub client: Client,
    pub user: &'a [String],
    pub serialized_secrets: &'a [u8],
    pub serialized_metadata: &'a [u8],
}

pub fn plugin_key(prefix: Prefix) -> String {
    format!("plugin_{prefix}")
}

pub fn secret_key(prefix: Prefix) -> String {
    format!("secret_{prefix}")
}

pub struct PluginManager<'a>(&'a mut redis::Connection);

impl PluginManager<'_> {
    pub fn insert_plugin(self, prefix: Prefix, path: &str) -> Result<(), anyhow::Error> {
        self.0.set::<String, &str, _>(plugin_key(prefix), path)?;
        Ok(())
    }

    pub fn insert_secret<T: Serialize>(
        self,
        prefix: Prefix,
        secret: &T,
    ) -> Result<(), anyhow::Error> {
        let serialized_secret = cbor_serialize(secret)?;
        self.0.set(secret_key(prefix), serialized_secret)?;
        Ok(())
    }

    fn symbol<'a, T>(self, prefix: Prefix, name: &[u8]) -> Result<Symbol<'a, T>, anyhow::Error> {
        let path = self.0.get::<String, String>(plugin_key(prefix))?;
        let library = unsafe { Library::new(path) }?;
        unsafe { library.get(name) }.map_err(|e| anyhow::anyhow!(e))
    }

    pub fn call_one(self, prefix: Prefix, input: CallOneInput) -> CallOneResult {
        let dynamic_call: CallOne = *self.symbol(prefix, b"call_one")?;
        dynamic_call(input)
    }
}

/*
#[cfg(test)]
mod test {
    use super::*;

    type TestCall = extern "C" fn() -> String;

    fn name(redis: &mut redis::Connection, prefix: Prefix) -> Result<String, anyhow::Error> {
        let dynamic_call: TestCall = symbol(redis, prefix, b"name")?;
        Ok(dynamic_call())
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
*/
