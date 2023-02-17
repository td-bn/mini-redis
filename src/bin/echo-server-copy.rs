use tokio::{
    io::{self, split},
    net::TcpListener,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let (mut rd, mut wr) = split(socket);
            if io::copy(&mut rd, &mut wr).await.is_err() {
                eprintln!("Failed to copy");
            }
        });
    }

    #[allow(unreachable_code)]
    Ok(())
}
