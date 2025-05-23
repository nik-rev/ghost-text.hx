use std::error::Error;

use tokio::{io::AsyncWriteExt, net::TcpListener};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("localhost:4001").await?;

    loop {
        let (mut socket, addr) = listener.accept().await?;

        socket
            .write_all(
                br#"200 OK
Content-Type: application/json

{
  "ProtocolVersion": 1,
  "WebSocketPort": 4001
}"#,
            )
            .await?;
        println!("{:?}, {:?}", socket, addr);
    }
}
