extern crate serde;
extern crate serde_json;
use serde_json::{Value, Error};

pub fn deserialize(data: &str) -> Result<Value, Error> {
    let deserialize: Value = serde_json::from_str(data)?;
    Ok(deserialize)
}
