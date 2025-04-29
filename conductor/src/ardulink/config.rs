
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArdulinkConfig {
    pub connection: ArdulinkConnectionType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "args")]
pub enum ArdulinkConnectionType {
    Serial(String, u32),
    Udp(String, u32),
    Tcp(String, u32),
}

impl ArdulinkConfig {
    pub fn new(connection: ArdulinkConnectionType) -> Self {
        Self { connection }
    }
}

impl ArdulinkConnectionType {
    pub fn connection_string(&self) -> String {
        match self {
            ArdulinkConnectionType::Serial(path, baud) => format!("serial:{}:{}", path, baud),
            ArdulinkConnectionType::Udp(address, port) => format!("udpin:{}:{}", address, port),
            ArdulinkConnectionType::Tcp(address, port) => format!("tcpout:{}:{}", address, port),
        }
    }

    pub fn get_port(&self) -> u32 {
        match self {
            ArdulinkConnectionType::Serial(_, port) => *port,
            ArdulinkConnectionType::Udp(_, port) => *port,
            ArdulinkConnectionType::Tcp(_, port) => *port,
        }
    }
}
