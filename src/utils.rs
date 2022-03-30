use async_std::prelude::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub type AppError = Box<dyn std::error::Error>;
pub type AppResult<T> = Result<T, AppError>;

pub async fn send_as_json<S, P>(outbound: &mut S, packet: &P) -> AppResult<()>
where
    S: async_std::io::Write + std::marker::Unpin,
    P: Serialize,
{
    let mut json = serde_json::to_string(&packet)?;
    json.push('\n');
    outbound.write_all(json.as_bytes()).await?;
    Ok(())
}

pub fn receive_as_json<S, P>(inbound: S) -> impl Stream<Item = AppResult<P>>
where
    S: async_std::io::BufRead + std::marker::Unpin,
    P: DeserializeOwned,
{
    inbound.lines()
        .map(|line_result| -> AppResult<P> {
            let line = line_result?;
            let parsed = serde_json::from_str::<P>(&line)?;
            Ok(parsed)
        })
}

pub fn type_of<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
}

#[cfg(test)]
mod test {
    use crate::common::Value;
    use crate::utils::AppResult;
    use crate::message::*;
    use async_std::prelude::*;
    use async_std::task;
    use async_std::io::{Cursor, BufReader};

    #[test]
    fn test_send_as_json() {
        let json = task::block_on(async {
            let message = Message::GetDataRequest(GetDataRequest {
                tag: Some("ABC".to_string()),
                params: vec!["SP1".to_string(),],
            });

            let mut buf = Cursor::new(Vec::new());
            super::send_as_json(&mut buf, &message).await.unwrap();
            String::from_utf8(buf.into_inner())
        });

        let expected = format!("{}\n", r#"{"command":"GetDataRequest","tag":"ABC","params":["SP1"]}"#);

        assert_eq!(json, Ok(expected));
    }

    #[test]
    fn test_receive_as_json() {
        let input = format!("{}\n{}\n",
            r#"{"command":"GetDataRequest", "tag":"ABC", "params": ["SP1"]}"#,
            r#"{"command":"SetDataRequest", "params":[{"label": "NE1", "value": -3.14}]}"#
        );

        let messages_result: AppResult<Vec<Message>> = task::block_on(async {
            let mut reply_stream = super::receive_as_json(BufReader::new(input.as_bytes()));

            let mut messages = Vec::new();
            while let Some(reply) = reply_stream.next().await {
                match reply? {
                    m @ Message::GetDataRequest(..) => messages.push(m),
                    m @ Message::SetDataRequest(..) => messages.push(m),
                    _ => (),
                }
            };

           Ok(messages)
        });

        let messages = messages_result.unwrap();

        if let Message::GetDataRequest(req) = &messages[0] {
            assert_eq!(req.tag, Some("ABC".to_string()));
            assert_eq!(req.params[0], "SP1".to_string());
        } else {
            assert!(false);
        }

        if let Message::SetDataRequest(req) = &messages[1] {
            assert_eq!(req.tag, None);
            assert_eq!(req.params[0].label, "NE1".to_string());
            assert_eq!(req.params[0].value, Value::Float(-3.14));
        } else {
            assert!(false);
        }
    }
}
