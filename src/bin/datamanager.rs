use async_std::prelude::*;
use datamanager::utils::AppResult;

fn main() -> AppResult<()> {

    async_std::task::block_on(async {
        use async_std::net;
        let listener = net::TcpListener::bind("localhost:8080").await?;

        let mut new_connections = listener.incoming();
        while let Some(socket_result) = new_connections.next().await {
            let _socket = socket_result?;
            println!("accept!");
        }
        println!("Hello world!");
        Ok(())
    })
}
