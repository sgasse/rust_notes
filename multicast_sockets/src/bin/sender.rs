use std::{
    io::IoSlice,
    net::{IpAddr, Ipv6Addr, SocketAddrV6},
    num::Wrapping,
    time::Duration,
};

use anyhow::{Context, Result};
use clap::Parser;
use humantime::parse_duration;
use multicast_sockets::get_interface_by_name;
use nix::{
    libc::{in6_addr, in6_pktinfo},
    sys::socket::{
        bind, sendmsg, socket, ControlMessage, MsgFlags, SockFlag, SockType, SockaddrIn6,
    },
};

#[derive(Parser)]
pub struct Cli {
    /// Source port.
    #[arg(long, default_value_t = 20202)]
    pub src_port: u16,

    /// Name of interface to use.
    #[arg(short, long, default_value = "eth0")]
    pub interface_name: String,

    /// Target group address.
    #[arg(short, long, default_value = "[ff14::1a]:30000")]
    pub target_group: String,

    /// Send period.
    #[arg(short, long, value_parser = parse_duration)]
    pub period: Option<Duration>,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let period = args.period.unwrap_or(Duration::from_secs(1));

    let socket = socket(
        nix::sys::socket::AddressFamily::Inet6,
        SockType::Datagram,
        SockFlag::SOCK_NONBLOCK,
        None,
    )
    .context("failed to open socket")?;

    let bind_addr: SockaddrIn6 =
        SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, args.src_port, 0, 0).into();
    bind(socket, &bind_addr).context("failed to bind socket")?;

    let net_if = get_interface_by_name(&args.interface_name).context("interface not found")?;
    let src_addr = net_if
        .ips
        .iter()
        .find(|ip| ip.is_ipv6())
        .and_then(|ip| {
            if let IpAddr::V6(ip) = ip.ip() {
                Some(ip)
            } else {
                None
            }
        })
        .context("no IPv6 address on interface")?;

    // Set source IP address and interface.
    let ipv6_info = in6_pktinfo {
        ipi6_addr: in6_addr {
            s6_addr: src_addr.octets(),
        },
        #[cfg(target_os = "android")]
        ipi6_ifindex: net_if.index as i32,
        #[cfg(not(target_os = "android"))]
        ipi6_ifindex: net_if.index,
    };

    let cmsgs = &[ControlMessage::Ipv6PacketInfo(&ipv6_info)];
    let dst_addr: SockaddrIn6 = args.target_group.parse::<SocketAddrV6>()?.into();

    let mut counter = Wrapping(0_u32);
    let mut payload = counter.0.to_be_bytes();

    loop {
        println!("Sending {}", counter.0);
        sendmsg(
            socket,
            &[IoSlice::new(&payload)],
            cmsgs,
            MsgFlags::empty(),
            Some(&dst_addr),
        )?;

        std::thread::sleep(period);

        counter += 1;
        payload = counter.0.to_be_bytes();
    }
}
