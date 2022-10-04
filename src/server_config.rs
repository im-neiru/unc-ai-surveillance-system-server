use std::io::BufReader;
use std::fs::File;

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