use guild_common::Scalar;
pub use redis;

use libloading::Library;
use redis::{Commands, ConnectionLike};
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

pub struct PluginManager<'a, C>(&'a mut C);

impl<'a, C> PluginManager<'a, C>
where
    C: ConnectionLike,
{
    pub fn new(connection: &'a mut C) -> Self {
        Self(connection)
    }

    pub fn insert_plugin(&mut self, prefix: Prefix, path: &str) -> Result<(), anyhow::Error> {
        self.0.set::<String, &str, _>(plugin_key(prefix), path)?;
        Ok(())
    }

    pub fn insert_secret<T: Serialize>(
        &mut self,
        prefix: Prefix,
        secret: &T,
    ) -> Result<(), anyhow::Error> {
        let serialized_secret = cbor_serialize(secret)?;
        self.0.set(secret_key(prefix), serialized_secret)?;
        Ok(())
    }

    fn library(&mut self, prefix: Prefix) -> Result<Library, anyhow::Error> {
        let path = self.0.get::<String, String>(plugin_key(prefix))?;
        let library = unsafe { Library::new(path) }?;
        Ok(library)
    }

    pub fn call_one(&mut self, prefix: Prefix, input: CallOneInput) -> CallOneResult {
        let library = self.library(prefix)?;
        let dynamic_call: CallOne = *unsafe { library.get(b"call_one") }?;
        dynamic_call(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use redis_test::{MockCmd, MockRedisConnection};

    type TestCall = extern "C" fn() -> String;

    impl PluginManager<'_, MockRedisConnection> {
        fn name(&mut self, prefix: Prefix) -> Result<String, anyhow::Error> {
            let library = self.library(prefix)?;
            let dynamic_call: TestCall = *unsafe { library.get(b"name") }?;
            Ok(dynamic_call())
        }
    }

    #[test]
    fn load_test_libraries() {
        let client = Client::new();
        let dummy_input = CallOneInput {
            client: client,
            user: &[String::from("")],
            serialized_secrets: &[],
            serialized_metadata: &[],
        };


        let module_a = "./plugins/libtest_lib_a.module";
        let module_b = "./plugins/libtest_lib_b.module";

        let mut mock_redis = MockRedisConnection::new(vec![
            MockCmd::new(redis::cmd("SET").arg(plugin_key(0)).arg(module_a), Ok(0)),
            MockCmd::new(redis::cmd("SET").arg(plugin_key(1)).arg(module_b), Ok(1)),
            MockCmd::new(redis::cmd("SET").arg(secret_key(0)).arg(&String::from("secret")), Ok(2)),
            // TODO MockCmd::new(redis::cmd("GET").arg(secret_key(0)), Ok("secret")),
            MockCmd::new(redis::cmd("GET").arg(plugin_key(0)), Ok(module_a)),
            MockCmd::new(redis::cmd("GET").arg(plugin_key(1)), Ok(module_b)),
        ]);

        let mut plugin_manager = PluginManager::new(&mut mock_redis);
        assert!(plugin_manager.insert_plugin(0, module_a).is_ok());
        assert!(plugin_manager.insert_plugin(1, module_b).is_ok());
        assert!(dbg!(plugin_manager.insert_secret(0, &String::from("secret"))).is_ok());

        assert_eq!(plugin_manager.name(0).unwrap(), "test-lib-a");
        assert_eq!(plugin_manager.name(1).unwrap(), "test-lib-b");

        assert!(plugin_manager
            .insert_plugin(2, "nonexistent/path")
            .is_err());
        assert!(plugin_manager.call_one(0, dummy_input).is_err());
    }
}
