use std::io::BufReader;
use std::fs::File;
use std::net::{SocketAddr, ToSocketAddrs};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(alias = "web-server")]
    pub web_server: WebServerConfig
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WebServerConfig {
    pub port: u16,
    pub address: String
}

impl ServerConfig {
    pub fn load() -> ServerConfig {
        let reader = BufReader::new(File::open("server-config.yaml")
            .expect("Unable to open file"));
        serde_yaml::from_reader(reader).unwrap()
    }
}

impl ToSocketAddrs for WebServerConfig {
    type Iter = std::vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        (self.address.as_ref(), self.port).to_socket_addrs()
    }
}