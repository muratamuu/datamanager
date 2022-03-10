use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    pub data: Vec<Data>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub label: String,
    pub value: Value,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Value {
    Int(i64),
    Float(f64),
    Text(String),
}

#[cfg(test)]
mod tests {
    use crate::message::*;

    #[test]
    fn test_serialize_basic_request() {

        let req = Request {
            command: "SetDataRequest".to_string(),
            tag: Some("ABC".to_string()),
            data: vec![Data {
                label: "SP1".to_string(),
                value: Value::Float(3.0),
            }],
        };

        let serialized_req = serde_json::to_string(&req).unwrap();
        assert_eq!(serialized_req,
            r#"{"command":"SetDataRequest","tag":"ABC","data":[{"label":"SP1","value":3.0}]}"#);

        let deserialized_req: Request = serde_json::from_str(&serialized_req).unwrap();
        assert_eq!(deserialized_req, req);
    }

    #[test]
    fn test_serialize_non_tagged_request() {

        let req = Request {
            command: "SetDataRequest".to_string(),
            tag: None,
            data: vec![Data {
                label: "SP1".to_string(),
                value: Value::Float(3.0),
            }],
        };

        let serialized_req = serde_json::to_string(&req).unwrap();
        assert_eq!(serialized_req,
            r#"{"command":"SetDataRequest","data":[{"label":"SP1","value":3.0}]}"#);

        let deserialized_req: Request = serde_json::from_str(&serialized_req).unwrap();
        assert_eq!(deserialized_req, req);
    }
}
