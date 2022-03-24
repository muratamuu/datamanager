use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "command")]
pub enum Message {
    GetDataRequest(GetDataRequest),
    SetDataRequest(SetDataRequest),
}

pub type Label = String;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GetDataRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    pub params: Vec<Label>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SetDataRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    pub params: Vec<LabeledValue>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct LabeledValue {
    pub label: String,
    pub value: Value,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
}

#[cfg(test)]
mod tests {
    use crate::message::*;

    #[test]
    fn test_serialize_set_request() {
        let message = Message::SetDataRequest(SetDataRequest {
            tag: Some("ABC".to_string()),
            params: vec![LabeledValue {
                label: "SP1".to_string(),
                value: Value::Float(3.0),
            }],
        });

        let json = serde_json::to_string(&message).unwrap();

        assert_eq!(json,
            r#"{"command":"SetDataRequest","tag":"ABC","params":[{"label":"SP1","value":3.0}]}"#);
    }

    #[test]
    fn test_deserialize_set_request() {
        let json = r#"
            {
                "command": "SetDataRequest",
                "tag": "123",
                "params": [
                    {"label": "SP1", "value": 3.14}
                ]
            }
        "#;

        if let Message::SetDataRequest(message) = serde_json::from_str(json).unwrap() {
            assert_eq!(message.tag, Some("123".to_string()));
            assert_eq!(message.params[0].label, "SP1");
            assert_eq!(message.params[0].value, Value::Float(3.14));
        } else {
            panic!("not SetDataRequest");
        }
    }

    #[test]
    fn test_serialize_no_tagged_request() {
        let message = Message::SetDataRequest(SetDataRequest {
            tag: None,
            params: vec![LabeledValue {
                label: "SP1".to_string(),
                value: Value::Float(3.0),
            }],
        });

        let json = serde_json::to_string(&message).unwrap();
        assert_eq!(json,
            r#"{"command":"SetDataRequest","params":[{"label":"SP1","value":3.0}]}"#);
    }

    #[test]
    fn test_deserialize_no_tagged_request() {
        let json = r#"
            {
                "command": "SetDataRequest",
                "params": [
                    {"label": "userName", "value": "murata"}
                ]
            }
        "#;

        if let Message::SetDataRequest(message) = serde_json::from_str(json).unwrap() {
            assert_eq!(message.tag, None);
            assert_eq!(message.params[0].label, "userName");
            assert_eq!(message.params[0].value, Value::String("murata".to_string()));
        } else {
            panic!("not SetDataRequest");
        }
    }

    #[test]
    fn test_serialize_get_request() {
        let message = Message::GetDataRequest(GetDataRequest {
            tag: Some("ABC".to_string()),
            params: vec![
                "SP1".to_string(),
                "NE1".to_string(),
            ],
        });

        let json = serde_json::to_string(&message).unwrap();

        assert_eq!(json,
            r#"{"command":"GetDataRequest","tag":"ABC","params":["SP1","NE1"]}"#);
    }

    #[test]
    fn test_deserialize_get_request() {
        let json = r#"
            {
                "command": "GetDataRequest",
                "tag": "123",
                "params": ["SP1", "NE1"]
            }
        "#;

        match serde_json::from_str(json).unwrap() {
            Message::GetDataRequest(message) => {
                assert_eq!(message.tag, Some("123".to_string()));
                assert_eq!(message.params[0], "SP1");
                assert_eq!(message.params[1], "NE1");
            }
            _ => panic!("not GetDataRequest"),
        };
    }
}
