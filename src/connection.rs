use std::io::Cursor;

use bytes::{Buf, BytesMut};
use mini_redis::frame::Error::Incomplete;
use mini_redis::{Frame, Result};
use tokio::io::{BufWriter, AsyncWriteExt};
use tokio::{io::AsyncReadExt, net::TcpStream};

struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4096),
        }
    }
    // Read a frame from the connection, return None is EOF is reached
    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            // Read from buffer
            if let Some(frame) = self.parse_frame().await? {
                return Ok(Some(frame));
            }

            // Read into buffer
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // Nothing else to read, buffer is empty
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    // Nothing to read, but buffer not empty
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    // Write a frame to the connection
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        match frame {
            Frame::Simple(val) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write(b"\r\n").await?;
            },
            Frame::Error(val) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write(b"\r\n").await?;
            },
            Frame::Integer(val) => {
                self.stream.write_u8(b':').await?;
                self.stream.write_u64(*val).await?;
                self.stream.write(b"\r\n").await?;
            },
            Frame::Null => {
                self.stream.write_all(b"$-1\r\n").await?;
            },
            Frame::Bulk(val) => {
                self.stream.write_u8(b'$').await?;
                self.stream.write_u64(val.len() as u64).await?;
                self.stream.write_all(val).await?;
                self.stream.write(b"\r\n").await?;
            },
            Frame::Array(_val) => unimplemented!(),
        }
        self.stream.flush().await?;
        Ok(())
    }

    async fn parse_frame(&mut self) -> Result<Option<Frame>> {
        // Seek implementation for buffer
        let mut buf = Cursor::new(&self.buffer[..]);

        // Check if frame is available
        match Frame::check(&mut buf) {
            Ok(_) => {
                // Get len of frame
                let len = buf.position() as usize;
                // Reset internal buf
                buf.set_position(0);

                // Read frame from buffer
                let frame = Frame::parse(&mut buf)?;

                // Advance self.buffer by len
                self.buffer.advance(len);
                Ok(Some(frame))
            }
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
