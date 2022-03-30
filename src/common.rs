use serde::{Serialize, Deserialize};

pub type Label = String;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Null,
}
