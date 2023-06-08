use std::net::SocketAddr;

use anyhow::Context;
use tokio::{
    io::{AsyncWriteExt, BufReader, BufWriter},
    net::{TcpListener, TcpStream, UdpSocket},
};

use crate::dgram;

pub struct ServerArgs {
    pub address: SocketAddr,
    pub target: SocketAddr,
}

pub async fn run(args: ServerArgs) -> anyhow::Result<()> {
    let listener = TcpListener::bind(args.address).await?;

    while let Ok((stream, addr)) = listener.accept().await {
        log::info!("Accepted connection from {}", addr);
        tokio::spawn(async move {
            if let Err(e) = stream_task(stream, args.target).await {
                log::error!("Error handling connection from {}\n{:?}", addr, e);
            }
        });
    }

    Ok(())
}

async fn stream_task(mut stream: TcpStream, target: SocketAddr) -> anyhow::Result<()> {
    let local_addr = local_addr_same_family(&target);
    let socket = UdpSocket::bind(&local_addr)
        .await
        .context(format!("Failed to bind to local address: {}", local_addr))?;
    log::debug!("Bound to local address: {}", socket.local_addr()?);

    let mut buffer = vec![0u8; 65535];
    socket
        .connect(target)
        .await
        .context(format!("Failed to connect to target address: {}", target))?;

    let (reader, writer) = stream.split();
    let mut reader = dgram::Reader::new(BufReader::new(reader));
    let mut writer = BufWriter::new(writer);

    loop {
        tokio::select! {
            result = socket.recv(&mut buffer) => {
                let size = result?;
                let message = &buffer[..size];
                assert!(message.len() <= usize::from(u16::MAX));
                writer.write_u16(size as u16).await?;
                writer.write_all(message).await?;
                writer.flush().await?;
            }
            result = reader.read() => {
                let message = match result {
                    Ok(message) => message,
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                    Err(e) => return Err(e.into()),
                };
                log::trace!("Received message from client with size: {}", message.len());
                socket.send(message).await?;
            }
        }
    }

    log::info!("client {} disconnected", stream.peer_addr()?);
    Ok(())
}

fn local_addr_same_family(addr: &SocketAddr) -> SocketAddr {
    match addr {
        SocketAddr::V4(_) => SocketAddr::from(([0, 0, 0, 0], 0)),
        SocketAddr::V6(_) => SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], 0)),
    }
}
