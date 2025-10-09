use clap::Parser;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::{
    io::Read,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    str::FromStr,
    sync::{
        LazyLock,
        atomic::{AtomicBool, Ordering},
    },
};

const PATH: &str = "/home/catornot/.local/share/Steam/steamapps/common/Titanfall2/vpk/";

const PORT_START: u16 = 37020;
static FREE_INTERFACES: LazyLock<[AtomicBool; 64]> =
    LazyLock::new(|| std::array::from_fn(|_| AtomicBool::new(true)));

#[derive(Parser)] // requires `derive` feature
#[command(name = "bspeater")]
#[command(bin_name = "bspeater")]
pub struct Cli {
    #[arg(long, short = 'f')]
    pub forward: IpAddr,

    #[arg(long, short = 'p')]
    pub port: u16,
}

fn main() -> Result<(), String> {
    println!("hello!");
    // let Cli { forward, port } = Cli::parse();
    // let forward = SocketAddr::from((forward, port));
    let port = 37015;
    let forward = SocketAddr::from((
        IpAddr::from_str("100.71.46.39")
            .map_err(|err| with_context(err.to_string(), "poor ip bro"))?,
        port,
    ));

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
        .map_err(|err| with_context(err.to_string(), "poor socket"))?;
    // socket
    //     .join_multicast_v6(
    //         &Ipv6Addr::UNSPECIFIED,
    //         200 + FREE_INTERFACES
    //             .iter()
    //             .enumerate()
    //             .find_map(|(i, interface)| {
    //                 interface
    //                     .load(Ordering::Acquire)
    //                     .then(|| interface.store(false, Ordering::Release))
    //                     .map(|_| i)
    //             })
    //             .ok_or_else(|| "couldn't get a interface for this bruh".to_string())?
    //             as u32,
    //     )
    //     .map_err(|err| with_context(err.to_string(), "multi not casting"))?;
    socket
        .bind(&SockAddr::from(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port,
        )))
        .map_err(|err| with_context(err.to_string(), "bind failed lost all silk"))?;

    socket
        .listen(30000)
        .map_err(|err| with_context(err.to_string(), "listen socket"))?;

    let mut foreign = None::<SocketAddr>;

    while let Ok((mut socket, addr)) = socket.accept() {
        let send_to = match addr.as_socket() {
            Some(addr) if addr == forward && foreign.is_some() => foreign.unwrap(),
            Some(addr) if addr == forward => {
                println!("dropped packet");
                continue;
            }
            Some(addr) => {
                foreign = Some(addr);
                forward
            }
            None => {
                println!("received smth without an address what?");
                continue;
            }
        };

        let mut buf = Vec::new();
        socket.read_to_end(&mut buf);
        socket.send_to(&buf, &SockAddr::from(send_to));
    }

    println!("goodbye!");
    Ok(())
}

fn with_context<T: AsRef<str>>(err: T, context: &str) -> String {
    err.as_ref().to_string() + " " + context
}
