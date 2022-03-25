use crate::utils::{self, AppResult};
use crate::message::*;
use async_std::prelude::*;
use async_std::net::TcpStream;
use async_std::io::BufReader;
use std::collections::HashMap;

pub async fn serve(socket: TcpStream, store: &mut HashMap<Label, Value>) -> AppResult<()> {

    let mut from_client = utils::receive_as_json(BufReader::new(socket));
    while let Some(message_result) = from_client.next().await {
        let message = message_result?;
        match message {
            Message::GetDataRequest(r) => {
                for label in r.params {
                    println!("{:?}", store.get(&label));
                }
            }
            Message::SetDataRequest(r) => {
                for LabeledValue { label, value } in r.params {
                    store.insert(label, value);
                }
            }
            _ => (),
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::utils::AppResult;
    use std::collections::HashMap;

    #[test]
    fn test_serve() {
        use async_std::prelude::*;
        use async_std::{net, task};

        task::block_on(async {
            let mut store = HashMap::new();
            // server
            let server_fut = async {
                let listener = net::TcpListener::bind("localhost:8888").await?;
                let mut new_connections = listener.incoming();
                while let Some(socket_result) = new_connections.next().await {
                    let socket = socket_result?;
                    super::serve(socket, &mut store).await?;
                }
                Ok(()) as AppResult<()>
            };

            // client
            let client_fut = async {
                let mut socket = net::TcpStream::connect("localhost:8888").await?;
                let message = format!("{}\n{}\n",
                    r#"{"command":"SetDataRequest", "params": [{"label":"SP1", "value":3.0}, {"label":"NE1", "value":10}]}"#,
                    r#"{"command":"GetDataRequest", "tag":"ABC", "params": ["SP1"]}"#);
                socket.write_all(message.as_bytes()).await?;
                task::yield_now().await;
                Ok(()) as AppResult<()>
            };

            let result = server_fut.race(client_fut).await;

            assert!(matches!(result, Ok(..)));
        });
    }
}
