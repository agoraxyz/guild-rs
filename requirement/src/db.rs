use redis::{Client, Commands, Connection};
use serde_json::Value;

pub struct RedisCache {
    pub conn: Option<Connection>,
}

impl Default for RedisCache {
    fn default() -> Self {
        Self {
            conn: match Client::open("redis://127.0.0.1/") {
                Ok(client) => match client.get_connection() {
                    Ok(conn) => Some(conn),
                    _ => None,
                },
                _ => None,
            },
        }
    }
}

impl RedisCache {
    pub fn read(&mut self, key: &str) -> Option<Value> {
        if let Some(con) = self.conn.as_mut() {
            if let Ok(entry) = con.get::<&str, String>(key) {
                if let Ok(value) = serde_json::from_str(&entry) {
                    return Some(value);
                } else {
                    let _: Result<(), _> = con.del(key);
                }
            }
        }

        None
    }

    pub fn write(&mut self, key: &str, value: &Value) {
        if let Some(con) = self.conn.as_mut() {
            let _: Result<(), _> = con.set(key, serde_json::to_string(value).unwrap_or_default());
        }
    }
}
