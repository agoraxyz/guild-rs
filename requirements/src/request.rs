use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Method {
    Get,
    Put,
    Post,
    Delete,
    Patch,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Auth {
    None,
    ApiKey(String),
    Bearer(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Data {
    None,
    UrlEncoded(String),
    JsonBody(Value),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Request {
    pub base_url: String,
    pub method: Method,
    pub data: Data,
    pub auth: Auth,
    pub path: Vec<Value>,
}
