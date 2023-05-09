use redis::{ErrorKind, RedisError, RedisResult, Value};
use std::collections::HashMap;

pub struct MockDb(HashMap<String, String>);

impl MockDb {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn inner_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.0
    }

    pub fn inner(&self) -> &HashMap<String, String> {
        &self.0
    }
}

impl Default for MockDb {
    fn default() -> Self {
        Self::new()
    }
}

impl redis::ConnectionLike for MockDb {
    fn req_packed_command(&mut self, cmd: &[u8]) -> RedisResult<Value> {
        let lines = std::str::from_utf8(cmd)?
            .lines()
            .skip(2)
            .step_by(2)
            .map(|line| String::from(line.trim_end_matches("\r")))
            .collect::<Vec<String>>();

        let redis_cmd = String::from(
            lines
                .get(0)
                .ok_or(RedisError::from((ErrorKind::IoError, "invalid command")))?,
        );
        let key = String::from(
            lines
                .get(1)
                .ok_or(RedisError::from((ErrorKind::IoError, "invalid command")))?,
        );
        match redis_cmd.as_ref() {
            "GET" => Ok(Value::Data(
                self.0
                    .get(&key)
                    .ok_or(RedisError::from((ErrorKind::IoError, "invalid command")))?
                    .as_bytes()
                    .to_vec(),
            )),
            "SET" => {
                let value = String::from(
                    lines
                        .get(2)
                        .ok_or(RedisError::from((ErrorKind::IoError, "invalid command")))?,
                );
                self.0.insert(key, value);
                Ok(Value::Okay)
            }
            _ => unimplemented!(),
        }
    }

    fn req_packed_commands(
        &mut self,
        _cmd: &[u8],
        _offset: usize,
        _count: usize,
    ) -> RedisResult<Vec<Value>> {
        unimplemented!()
    }

    fn get_db(&self) -> i64 {
        unimplemented!()
    }

    fn check_connection(&mut self) -> bool {
        unimplemented!()
    }

    fn is_open(&self) -> bool {
        unimplemented!()
    }
}
