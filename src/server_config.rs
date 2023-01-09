use std::net::{ SocketAddrV4, Ipv4Addr };


pub struct ServerConfig {
    pub port: u16,
    pub database_url: String,
}

impl ServerConfig {
    pub fn load() -> Self {
        Self {
            port: { std::env::var("ACTIX_PORT")
                .expect("Please set env: ACTIX_PORT")
                .parse::<u16>()
                .expect("Invalid ACTIX_PORT") },
            database_url: { std::env::var("DB_FOR_CLIENT_URL")
                .expect("Please set env: DB_FOR_CLIENT_URL") },
        }
    }

    pub fn actix_socket_addr(&self) -> SocketAddrV4 {
        SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), self.port)
    }
}
