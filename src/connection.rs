use crate::common::{Label, Value};
use crate::utils::{self, AppResult};
use crate::message::*;
use async_std::prelude::*;
use async_std::net::TcpStream;
use async_std::io::BufReader;
use std::collections::HashMap;

pub async fn serve(socket: TcpStream, store: &mut HashMap<Label, Value>) -> AppResult<()> {

    let mut outbound = socket.clone();
    let mut from_client = utils::receive_as_json(BufReader::new(socket));
    while let Some(message_result) = from_client.next().await {
        let message = message_result?;
        match message {
            Message::GetDataRequest(r) => {
                let status =
                    if r.params.iter().all(|label| store.contains_key(label)) { Status::OK }
                    else { Status::NotFound };
                let results = r.params.into_iter().map(|label| LabeledValue {
                    value: store.get(&label).unwrap_or(&Value::Null).clone(),
                    label,
                }).collect();
                let response = Message::GetDataResponse(GetDataResponse {
                    tag: r.tag,
                    status,
                    results,
                });
                utils::send_as_json(&mut outbound, &response).await?;
            }
            Message::SetDataRequest(r) => {
                for LabeledValue { label, value } in r.params {
                    store.insert(label, value);
                }
                let response = Message::SetDataResponse(SetDataResponse {
                    tag: r.tag,
                    status: Status::OK,
                });
                utils::send_as_json(&mut outbound, &response).await?;
            }
            _ => (),
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::common::Value;
    use crate::utils::{self, AppResult};
    use crate::message::*;
    use std::collections::HashMap;
    use async_std::prelude::*;
    use async_std::{net, task};
    use async_std::io::BufReader;

    #[test]
    fn test_serve_set_request_and_get_request() {

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
                // connect
                let mut socket = net::TcpStream::connect("localhost:8888").await?;
                let mut from_client = utils::receive_as_json(BufReader::new(socket.clone()));

                // send SetDataRequest
                let message = Message::SetDataRequest(SetDataRequest {
                    tag: Some("ABC".to_string()),
                    params: vec![
                        LabeledValue { label: "SP1".to_string(), value: Value::Float(3.0) },
                        LabeledValue { label: "NE1".to_string(), value: Value::Int(10) },
                    ],
                });
                utils::send_as_json(&mut socket, &message).await?;

                task::yield_now().await;

                // recv SendDataResponse
                while let Some(message_result) = from_client.next().await {
                    let message: Message = message_result?;
                    if let Message::SetDataResponse(r) = message {
                        assert_eq!(r.tag, Some("ABC".to_string()));
                        assert_eq!(r.status, Status::OK);
                    } else {
                        assert!(false);
                    }
                    break;
                }

                // send GetDataRequest
                let message = Message::GetDataRequest(GetDataRequest {
                    tag: Some("123".to_string()),
                    params: vec!["NE1".to_string(), "SP1".to_string()],
                });
                utils::send_as_json(&mut socket, &message).await?;

                task::yield_now().await;

                // recv GetDataResponse
                while let Some(message_result) = from_client.next().await {
                    let message: Message = message_result?;
                    if let Message::GetDataResponse(r) = message {
                        assert_eq!(r.tag, Some("123".to_string()));
                        assert_eq!(r.status, Status::OK);
                        assert_eq!(r.results[0].label, "NE1".to_string());
                        assert_eq!(r.results[0].value, Value::Int(10));
                        assert_eq!(r.results[1].label, "SP1".to_string());
                        assert_eq!(r.results[1].value, Value::Float(3.0));
                    } else {
                        assert!(false);
                    }
                    break;
                }

                Ok(()) as AppResult<()>
            };

            let result = server_fut.race(client_fut).await;

            assert!(matches!(result, Ok(..)));
        });
    }

    #[test]
    fn test_get_request_not_found_label() {

        task::block_on(async {

            let mut store = HashMap::new();

            // server
            let server_fut = async {
                // TODO: I want to change bind port to 8888.
                let listener = net::TcpListener::bind("localhost:8889").await?;
                let mut new_connections = listener.incoming();
                while let Some(socket_result) = new_connections.next().await {
                    let socket = socket_result?;
                    super::serve(socket, &mut store).await?;
                }
                Ok(()) as AppResult<()>
            };

            // client
            let client_fut = async {
                // connect
                let mut socket = net::TcpStream::connect("localhost:8889").await?;
                let mut from_client = utils::receive_as_json(BufReader::new(socket.clone()));

                // send SetDataRequest
                let message = Message::SetDataRequest(SetDataRequest {
                    tag: None,
                    params: vec![
                        LabeledValue { label: "SP1".to_string(), value: Value::Float(3.0) },
                    ],
                });
                utils::send_as_json(&mut socket, &message).await?;

                task::yield_now().await;

                // recv SendDataResponse
                while let Some(message_result) = from_client.next().await {
                    let message: Message = message_result?;
                    assert!(matches!(message, Message::SetDataResponse(..)));
                    break;
                }

                // send GetDataRequest
                let message = Message::GetDataRequest(GetDataRequest {
                    tag: None,
                    params: vec!["NE1".to_string(), "SP1".to_string()],
                });
                utils::send_as_json(&mut socket, &message).await?;

                task::yield_now().await;

                // recv GetDataResponse
                while let Some(message_result) = from_client.next().await {
                    let message: Message = message_result?;
                    if let Message::GetDataResponse(r) = message {
                        assert_eq!(r.status, Status::NotFound);
                        assert_eq!(r.results[0].label, "NE1".to_string());
                        assert_eq!(r.results[0].value, Value::Null);
                        assert_eq!(r.results[1].label, "SP1".to_string());
                        assert_eq!(r.results[1].value, Value::Float(3.0));
                    } else {
                        assert!(false);
                    }
                    break;
                }

                Ok(()) as AppResult<()>
            };

            let result = server_fut.race(client_fut).await;
            assert!(matches!(result, Ok(..)));
        });
    }
}
