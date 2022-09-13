use std::{net::SocketAddr, time::Duration};

use anyhow::Context;
use tokio::{
    io::{self, AsyncWriteExt, BufReader, BufWriter},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream, UdpSocket,
    },
};

use crate::dgram;

#[derive(Debug, Clone)]
pub struct ClientParams {
    pub address: SocketAddr,
    pub timeout: Duration,
    pub server: SocketAddr,
}

struct ServerConn {
    reader: dgram::Reader<BufReader<OwnedReadHalf>>,
    writer: BufWriter<OwnedWriteHalf>,
}

struct LazyServerConn {
    server: SocketAddr,
    conn: Option<ServerConn>,
}

pub async fn run(params: ClientParams) -> anyhow::Result<()> {
    let socket = create_udp_socket(params.address).await?;
    let mut sconn = LazyServerConn::new(params.server);
    let mut last_sender: Option<SocketAddr> = None;
    let mut timeout = tokio::time::interval(params.timeout);

    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    let mut local_buffer = vec![0u8; 65535];
    let mut server_buffer = vec![0u8; 65535];

    loop {
        timeout.reset();
        tokio::select! {
            _ = timeout.tick() => sconn.close(),
            Ok((size, sender)) = socket.recv_from(&mut local_buffer) => {
                log::trace!("Received message from local socket with size: {}", size);
                last_sender = Some(sender);
                match sconn.send(&local_buffer[..size]).await {
                    Ok(_) => (),
                    Err(e) => {
                        log::error!("Error sending to server: {}", e);
                        sconn.close();
                    }
                }
            }
            result = sconn.recv(&mut server_buffer) => {
                match result {
                    Ok(size) => {
                        log::trace!("Received message from server with size: {}", size);
                        let sender = last_sender.expect("sender should be set");
                        match socket.send_to(&server_buffer[..size], sender).await {
                            Ok(_) => (),
                            Err(e) => log::error!("Error sending to client: {}", e),
                        }
                    }
                    Err(e) => {
                        log::error!("Error receiving from server: {}", e);
                        sconn.close();
                    }
                }
            }
            r = &mut ctrl_c => {
                r?;
                break Ok(());
            }
        }
    }
}

impl ServerConn {
    async fn connect(server: SocketAddr) -> io::Result<Self> {
        let stream = TcpStream::connect(server).await?;
        let (reader, writer) = stream.into_split();
        Ok(ServerConn {
            reader: dgram::Reader::new(BufReader::new(reader)),
            writer: BufWriter::new(writer),
        })
    }
}

impl LazyServerConn {
    fn new(server: SocketAddr) -> Self {
        Self { server, conn: None }
    }

    fn close(&mut self) {
        self.conn = None;
    }

    async fn send(&mut self, message: &[u8]) -> io::Result<()> {
        let conn = match self.conn {
            Some(ref mut conn) => conn,
            None => {
                self.conn = Some(ServerConn::connect(self.server).await?);
                self.conn.as_mut().unwrap()
            }
        };
        log::trace!("Sending message to server with size: {}", message.len());
        assert!(message.len() <= usize::from(u16::MAX));
        conn.writer.write_u16(message.len() as u16).await?;
        conn.writer.write_all(message).await?;
        conn.writer.flush().await?;
        Ok(())
    }

    async fn recv(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        match &mut self.conn {
            Some(conn) => match conn.reader.read().await {
                Ok(msg) => {
                    buffer[..msg.len()].copy_from_slice(&msg);
                    Ok(msg.len())
                }
                Err(e) => Err(e),
            },
            None => futures::future::pending().await,
        }
    }
}

async fn create_udp_socket(address: SocketAddr) -> anyhow::Result<UdpSocket> {
    UdpSocket::bind(address)
        .await
        .with_context(|| format!("failed to bind client's UDP socket to {}", address))
}
