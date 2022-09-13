use std::net::SocketAddr;

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
                log::error!("Error handling connection from {}: {}", addr, e);
            }
        });
    }

    Ok(())
}

async fn stream_task(mut stream: TcpStream, target: SocketAddr) -> anyhow::Result<()> {
    let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).await?;
    let mut buffer = vec![0u8; 65535];
    socket.connect(target).await?;

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
                let message = result?;
                log::trace!("Received message from client with size: {}", message.len());
                socket.send(message).await?;
            }
        }
    }
}
