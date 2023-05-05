pub use redis;

use libloading::Library;
use redis::{Commands, ConnectionLike};
use serde::Serialize;

pub type Prefix = u64;

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
        let serialized_secret = serde_json::to_string(secret)?;
        self.0.set(secret_key(prefix), serialized_secret)?;
        Ok(())
    }

    pub fn serialized_secret(&mut self, prefix: Prefix) -> Result<String, anyhow::Error> {
        Ok(self.0.get::<String, String>(secret_key(prefix))?)
    }

    fn library(&mut self, prefix: Prefix) -> Result<Library, anyhow::Error> {
        let path = self.0.get::<String, String>(plugin_key(prefix))?;
        let library = unsafe { Library::new(path) }?;
        Ok(library)
    }

    pub fn call<Call, In, Out>(&mut self, prefix: Prefix, name: &[u8], input: In) -> Call::Output
    where
        Call: Fn(In) -> Result<Out, anyhow::Error> + Copy,
    {
        let library = self.library(prefix)?;
        let dynamic_call: Call = *unsafe { library.get(name) }?;
        dynamic_call(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use redis_test::{MockCmd, MockRedisConnection};

    type TestInA = ();
    type TestInB<'a> = &'a str;
    type TestOut = String;
    type TestCallA = fn(TestInA) -> Result<TestOut, anyhow::Error>;
    type TestCallB = fn(TestInB) -> Result<TestOut, anyhow::Error>;

    #[test]
    fn load_test_libraries() {
        let module_a = "./plugins/libtest_lib_a.module";
        let module_b = "./plugins/libtest_lib_b.module";
        let secret = String::from("secret");
        let serialized_secret = serde_json::to_string(&secret).unwrap();

        let mut mock_redis = MockRedisConnection::new(vec![
            MockCmd::new(redis::cmd("SET").arg(plugin_key(0)).arg(module_a), Ok(0)),
            MockCmd::new(redis::cmd("SET").arg(plugin_key(1)).arg(module_b), Ok(1)),
            MockCmd::new(
                redis::cmd("SET")
                    .arg(secret_key(0))
                    .arg(serialized_secret.clone()),
                Ok(2),
            ),
            MockCmd::new(redis::cmd("GET").arg(plugin_key(0)), Ok(module_a)),
            MockCmd::new(redis::cmd("GET").arg(plugin_key(1)), Ok(module_b)),
            MockCmd::new(
                redis::cmd("GET").arg(secret_key(0)),
                Ok(serialized_secret.clone()),
            ),
        ]);

        let mut plugin_manager = PluginManager::new(&mut mock_redis);
        assert!(plugin_manager.insert_plugin(0, module_a).is_ok());
        assert!(plugin_manager.insert_plugin(1, module_b).is_ok());
        assert!(plugin_manager.insert_secret(0, &secret).is_ok());

        assert_eq!(
            plugin_manager
                .call::<TestCallA, _, _>(0, b"call", ())
                .unwrap(),
            "test-lib-a"
        );
        assert_eq!(
            plugin_manager
                .call::<TestCallB, _, _>(1, b"call", "hello")
                .unwrap(),
            "test-lib-b-hello"
        );

        assert_eq!(
            plugin_manager.serialized_secret(0).unwrap(),
            serialized_secret
        );

        assert!(plugin_manager.insert_plugin(2, "nonexistent/path").is_err());
    }
}
