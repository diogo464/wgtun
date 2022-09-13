#![feature(try_blocks)]

mod client;
mod dgram;
mod server;

use std::{
    net::{SocketAddr, ToSocketAddrs},
    str::FromStr,
    time::Duration,
};

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Client(ClientArgs),
    Server(ServerArgs),
}

#[derive(Parser, Debug)]
struct ClientArgs {
    /// UDP port to bind to on the localhost.
    #[clap(long, default_value_t = 51820)]
    port: u16,
    /// Number of seconds of inactivity before the connection is closed.
    /// It is automatically reastablished when a new packet is received.
    #[clap(long, default_value_t = 30)]
    timeout: u32,
    /// The address of the server.
    #[clap(long)]
    server: String,
}

#[derive(Parser, Debug)]
struct ServerArgs {
    /// TCP socket address to bind to.
    #[clap(long, default_value = "0.0.0.0:51820")]
    address: SocketAddr,
    /// The address to send udp packets to.
    #[clap(long, default_value = "127.0.0.1:51280")]
    target: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();

    match args.command {
        Commands::Client(args) => run_client(args).await,
        Commands::Server(args) => run_server(args).await,
    }
}

async fn run_client(args: ClientArgs) -> anyhow::Result<()> {
    let server = args
        .server
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid server address"))?;

    client::run(client::ClientParams {
        address: SocketAddr::from(([127, 0, 0, 1], args.port)),
        timeout: Duration::from_secs(u64::from(args.timeout)),
        server,
    })
    .await
}

async fn run_server(args: ServerArgs) -> anyhow::Result<()> {
    let target = args
        .target
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid target address: {}", args.target))?;

    server::run(server::ServerArgs {
        address: args.address,
        target,
    })
    .await
}
