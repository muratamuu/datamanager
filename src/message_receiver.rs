use crate::utils::{self, AppResult};
use crate::message::Message;
use async_std::prelude::*;
use async_std::io::BufReader;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Outbound<S>(Rc<RefCell<S>>);

impl<S> Outbound<S>
where
    S: async_std::io::Write + std::marker::Unpin,
{
    pub fn new(to_client: S) -> Self {
        Self(Rc::new(RefCell::new(to_client)))
    }

    pub async fn send(&self, message: &Message) -> AppResult<()> {
        let mut outbound = self.0.borrow_mut();
        utils::send_as_json(&mut *outbound, message).await?;
        outbound.flush().await?;
        Ok(())
    }
}

pub fn receive_message<T>(async_io: T) -> impl Stream<Item = AppResult<(Message, Outbound<T>)>>
where
    T: async_std::io::Write + async_std::io::Read + std::marker::Unpin + std::clone::Clone,
{
    let outbound = Outbound::new(async_io.clone());
    utils::receive_as_json(BufReader::new(async_io))
        .map(move |message_result| {
            let message: Message = message_result?;
            Ok((message, outbound.clone()))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::task;
    use async_std::io::Cursor;

    #[test]
    fn test_message_receive() {
        let test_message = format!("{}\n{}\n",
            r#"{"command":"GetDataRequest","params":["SP1"]}"#,
            r#"{"command":"SetDataRequest","params":[{"label":"SP1","value":34.5}]}"#);
        let buf: Vec<u8> = test_message.as_bytes().iter().map(|b| b.clone()).collect();
        let cursor = Cursor::new(buf);

        task::block_on(async {
            let receiver_fut = async {
                let mut message_count = 0;
                let mut receiver = receive_message(cursor);
                while let Some(message_result) = receiver.next().await {
                    let (message, outbound) = message_result?;
                    outbound.send(&message).await?;
                    message_count += 1;
                }
                assert_eq!(message_count, 2);
                Ok(()) as AppResult<()>
            };

            let result = receiver_fut.await;

            assert!(matches!(result, Ok(..)));
        });
    }
}
