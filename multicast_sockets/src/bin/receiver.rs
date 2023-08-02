//! Multicast receiver.
//!
//! Run with `cargo run --target aarch64-linux-android --release --bin receiver`.

use std::{
    io::IoSliceMut,
    net::{Ipv6Addr, SocketAddrV6},
    time::Duration,
};

use anyhow::{Context, Result};
use bytes::Buf;
use clap::Parser;
use humantime::parse_duration;
use multicast_sockets::get_interface_by_name;
use nix::{
    cmsg_space,
    errno::Errno,
    sys::{
        socket::{
            bind, recvmmsg, setsockopt, socket, sockopt, AddressFamily, Ipv6MembershipRequest,
            MsgFlags, MultiHeaders, SockFlag, SockType, SockaddrIn, SockaddrIn6,
        },
        time::TimeVal,
    },
};

const FRAMES: usize = 32;
const BUFFER_SIZE: usize = 1400;

#[derive(Parser)]
pub struct Cli {
    /// Group addresses (without port) to listen to.
    #[arg(long, default_values_t = vec!["ff14::1a".to_string()])]
    pub group_addr: Vec<String>,

    /// Group port.
    #[arg(long, default_value_t = 30000)]
    pub group_port: u16,

    /// Name of interface to join on.
    #[arg(long, default_value = "eth0")]
    pub interface_name: String,

    /// Throttle between `recvmmsg` syscalls.
    #[arg(short, long, value_parser = parse_duration)]
    pub period: Option<Duration>,

    /// Handle only received iovs that have the right length to be a counter value.
    #[arg(short, long, action)]
    pub counter_only: bool,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let addresses: Vec<Ipv6Addr> = args
        .group_addr
        .iter()
        .map(|addr| addr.parse())
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let throttle = args.period.unwrap_or(Duration::from_millis(100));

    let socket = socket(
        AddressFamily::Inet6,
        SockType::Datagram,
        SockFlag::SOCK_NONBLOCK,
        None,
    )
    .context("failed to open socket")?;

    setsockopt(socket, sockopt::ReceiveTimestamp, &true)
        .context("failed to enable receive timestamps")?;
    setsockopt(socket, sockopt::ReuseAddr, &true).context("failed to enable reuseaddr")?;
    setsockopt(socket, sockopt::RxqOvfl, &1)
        .context("failed to enable receive queue overflow tracking")?;
    // The receive buffer gets some overhead size by the kernel but may be
    // limited by parameters.
    setsockopt(socket, sockopt::RcvBuf, &512000).context("failed to set receive buffer size")?;
    let bind_addr: SockaddrIn6 =
        SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, args.group_port, 0, 0).into();
    bind(socket, &bind_addr)?;

    let interface =
        get_interface_by_name(&args.interface_name).with_context(|| "failed to find interface")?;
    for group_addr in addresses {
        let join_req = get_ipv6_join_request(&group_addr, interface.index)?;
        setsockopt(socket, sockopt::Ipv6AddMembership, &join_req)
            .context("failed to join multicast group")?;
    }

    let cmsg_buffer = cmsg_space!(TimeVal, u32);
    let mut multi_headers = MultiHeaders::<SockaddrIn>::preallocate(FRAMES, Some(cmsg_buffer));

    let mut buffers = [[0u8; BUFFER_SIZE]; FRAMES];
    let msgs: Vec<_> = buffers
        .iter_mut()
        .map(|buf| [IoSliceMut::new(&mut buf[..])])
        .collect();

    loop {
        let _ = match recvmmsg(
            socket,
            &mut multi_headers,
            msgs.iter(),
            MsgFlags::empty(),
            None,
        ) {
            Ok(multi_results) => {
                let mut frames = 0;

                for recv_msg in multi_results {
                    frames += 1;

                    for iov in recv_msg.iovs() {
                        decode_payload(iov, args.counter_only);
                    }
                }

                frames
            }

            Err(Errno::EAGAIN) => 0,

            Err(e) => {
                println!("Error in recvmmsg: {e}");
                0
            }
        };

        std::thread::sleep(throttle);
    }
}

fn decode_payload(mut payload: &[u8], counter_only: bool) {
    match counter_only {
        true => {
            if payload.len() == 4 {
                let counter = payload.get_u32();
                println!("Counter: {counter}");
            }
        }
        false => {
            println!("Received iov with length {}", payload.len());
        }
    }
}

fn get_ipv6_join_request(group: &Ipv6Addr, interface_index: u32) -> Result<Ipv6MembershipRequest> {
    // nix does not provide a public method to create a
    // `Ipv6MembershipRequest` with a specific interface index. See:
    // https://github.com/nix-rust/nix/issues/323
    // To work around it, we create the inner type and transmute it to the
    // required type. Static assertions ensure the safety.

    // Depending on the target platform, this is a `u32` or `i32`.
    #[cfg(target_os = "android")]
    let interface_index = interface_index as i32;

    static_assertions::assert_eq_size!(Ipv6Addr, nix::libc::in6_addr);
    let join_request = nix::libc::ipv6_mreq {
        // SAFETY:
        // Safe as long as `std::net::Ipv6Addr` and `libc::in6_addr` are the
        // same size as asserted above.
        ipv6mr_multiaddr: unsafe { std::mem::transmute(*group) },
        ipv6mr_interface: interface_index,
    };

    // SAFETY:
    // Safe as long as `Ipv6MembershipRequest` is just a wrapper of
    // `libc::ipv6_mreq` as asserted.
    static_assertions::assert_eq_size!(Ipv6MembershipRequest, nix::libc::ipv6_mreq);
    let join_request: Ipv6MembershipRequest = unsafe { std::mem::transmute(join_request) };

    Ok(join_request)
}
