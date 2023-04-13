use std::net::{Ipv4Addr, SocketAddrV4};

use tokio::net;

pub(crate) struct SafeMulticast {
    socket: net::UdpSocket,

}

impl SafeMulticast {
    pub async fn new() -> Self {
        let ip = Ipv4Addr::new(0, 0, 0, 0);

        for port in 8000..=24000 {
            if let Ok(socket) = net::UdpSocket::bind(SocketAddrV4::new(ip, port)).await {
                return Self { socket }
            }
        }

        panic!("No available UDP port found")
    }
}
