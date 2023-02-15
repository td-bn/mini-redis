use std::collections::HashMap;

use mini_redis::{Command, Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        // Second item containts SockAddr: IP and port
        let (socket, _) = listener.accept().await.unwrap();

        // New task is spawned, socked is **moved** to the new task and processed there
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    // Read/write redis **frames** instead of byte streams.
    let mut connection = Connection::new(socket);
    let mut db = HashMap::new();

    // Use read_frame to receive a command from the connection
    while let Some(frame) = connection.read_frame().await.unwrap() {
        println!("GOT: {:?}", frame);

        // Respond with an error
        let response = match Command::from_frame(frame).unwrap() {
            Command::Set(cmd) => {
                // Value is stored as a Vec<u8>
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Command::Get(cmd) => {
                if let Some(value) = db.get(cmd.key()) {
                    // Expects type to be Bytes, it implements From<Vec<u8>>
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            _ => panic!("unimplemented"),
        };
        connection.write_frame(&response).await.unwrap();
    }
}
