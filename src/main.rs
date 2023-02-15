use std::{collections::HashMap, sync::{Arc, Mutex}};

use bytes::Bytes;
use mini_redis::{Command, Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        // Second item containts SockAddr: IP and port
        let (socket, _) = listener.accept().await.unwrap();

        // Clone the db 
        let db = db.clone();
        // New task is spawned, socked is **moved** to the new task and processed there
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: Db) {
    // Read/write redis **frames** instead of byte streams.
    let mut connection = Connection::new(socket);

    // Use read_frame to receive a command from the connection
    while let Some(frame) = connection.read_frame().await.unwrap() {
        println!("GOT: {:?}", frame);

        // Respond with an error
        let response = match Command::from_frame(frame).unwrap() {
            Command::Set(cmd) => {
                // Lock is unlocked on drop
                let mut db = db.lock().unwrap();
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Command::Get(cmd) => {
                // Lock is unlocked on drop
                let db = db.lock().unwrap();
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
