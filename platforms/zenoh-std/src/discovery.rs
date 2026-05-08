use std::net::SocketAddr;
use std::time::Duration;

use async_io::Timer;
use async_net::UdpSocket;
use futures_lite::future;
use zenoh_proto::{
    ZDecode, ZEncode,
    fields::{WhatAmI, ZenohIdProto},
    msgs::{Hello, InitIdentifier, Scout},
};

/// Timeout for receiving a HELLO response after sending a SCOUT.
const SCOUT_RECV_TIMEOUT: Duration = Duration::from_millis(500);

/// A discovered node from SCOUT/HELLO.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscNode {
    pub zid: ZenohIdProto,
    pub whatami: WhatAmI,
    pub locators: Option<String>,
}

/// Send a SCOUT via multicast and return any HELLO responses received.
/// Waits at most `SCOUT_RECV_TIMEOUT` for a single HELLO before returning.
pub async fn scout(endpoint: &str, what: u8) -> Result<Vec<DiscNode>, String> {
    let addr: SocketAddr = endpoint
        .parse()
        .map_err(|_| format!("invalid endpoint: {endpoint}"))?;

    let bind_addr = SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
        addr.port(),
    );
    let socket = UdpSocket::bind(bind_addr)
        .await
        .map_err(|e| format!("bind: {e}"))?;

    // Encode and send SCOUT
    let scout = Scout { what };
    let mut buf = [0u8; 64];
    let scout_len = {
        let mut writer = &mut buf[..];
        let start_len = writer.len();
        scout
            .z_encode(&mut writer)
            .map_err(|_| "encode SCOUT".to_string())?;
        start_len - writer.len()
    };

    socket
        .send_to(&buf[..scout_len], addr)
        .await
        .map_err(|e| format!("send SCOUT: {e}"))?;

    // Try to receive a HELLO response within the timeout.
    // `future::or` drops the losing branch when the winning branch completes,
    // which properly cancels the `recv_from` future via async-io's drop mechanism.
    let mut nodes = Vec::new();
    let mut recv_buf = [0u8; 1024];

    let recv_result = future::or(
        async { socket.recv_from(&mut recv_buf).await },
        async {
            Timer::after(SCOUT_RECV_TIMEOUT).await;
            Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "scout recv timeout",
            ))
        },
    )
    .await;

    if let Ok((n, _)) = recv_result {
        if let Ok(hello) = <Hello as ZDecode>::z_decode(&mut &recv_buf[..n]) {
            nodes.push(DiscNode {
                zid: hello.identifier.zid,
                whatami: hello.identifier.whatami,
                locators: hello.locators.map(|s| s.to_string()),
            });
        }
    }

    Ok(nodes)
}

/// Listen for SCOUTs and respond with HELLO indefinitely.
pub async fn respond_hellos(
    endpoint: &str,
    zid: ZenohIdProto,
    whatami: WhatAmI,
    locators: &str,
) -> Result<(), String> {
    let addr: SocketAddr = endpoint
        .parse()
        .map_err(|_| format!("invalid endpoint: {endpoint}"))?;

    let bind_addr = SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
        addr.port(),
    );

    let socket = UdpSocket::bind(bind_addr)
        .await
        .map_err(|e| format!("bind: {e}"))?;

    let multi_ip = match addr.ip() {
        std::net::IpAddr::V4(ip) => ip,
        _ => return Err("IPv6 not supported".into()),
    };
    socket
        .join_multicast_v4(multi_ip, std::net::Ipv4Addr::UNSPECIFIED)
        .map_err(|e| format!("join multicast: {e}"))?;

    let mut buf = [0u8; 1024];

    loop {
        let (n, src) = socket
            .recv_from(&mut buf)
            .await
            .map_err(|e| format!("recv: {e}"))?;

        if let Ok(scout) = <Scout as ZDecode>::z_decode(&mut &buf[..n]) {
            let bit = 1u8 << (whatami as u8);
            if scout.what == 0 || (scout.what & bit) != 0 {
                let hello = Hello {
                    version: zenoh_proto::VERSION,
                    identifier: InitIdentifier { zid, whatami },
                    locators: Some(locators),
                };

                let mut hbuf = [0u8; 256];
                let hello_len = {
                    let mut writer = &mut hbuf[..];
                    let start_len = writer.len();
                    hello
                        .z_encode(&mut writer)
                        .map_err(|_| "encode HELLO".to_string())?;
                    start_len - writer.len()
                };
                let _ = socket.send_to(&hbuf[..hello_len], src).await;
            }
        }
    }
}
