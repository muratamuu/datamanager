use async_std::prelude::*;
use serde::Serialize;

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

#[cfg(test)]
mod test {
    #[test]
    fn test_send_as_json() {
        use crate::message::*;

        let json = async_std::task::block_on(async {
            let message = Message::GetDataRequest(GetDataRequest {
                tag: Some("ABC".to_string()),
                params: vec!["SP1".to_string(),],
            });

            let mut buf = async_std::io::Cursor::new(Vec::new());
            super::send_as_json(&mut buf, &message).await.unwrap();
            String::from_utf8(buf.into_inner())
        });

        let expected = format!("{}\n", r#"{"command":"GetDataRequest","tag":"ABC","params":["SP1"]}"#);

        assert_eq!(json, Ok(expected));
    }
}
