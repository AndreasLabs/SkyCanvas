use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "args")]
pub enum MavlinkConnectionType {
    Serial(String, u32),
    Udp(String, u32),
    Tcp(String, u32),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MavConfig{
    pub connection: MavlinkConnectionType,
    pub telemetry_rate_hz: u32,
}

impl Default for MavConfig{
    fn default() -> Self {
        Self::new(
            MavlinkConnectionType::Tcp("127.0.0.1".to_string(), 5760),
            20,
        )
    }
}


impl MavConfig {
    pub fn new(connection: MavlinkConnectionType, telemetry_rate_hz: u32) -> Self {
        Self { connection, telemetry_rate_hz }
    }

    pub fn connection_string(&self) -> String {
        match &self.connection {
            MavlinkConnectionType::Serial(path, baud) => format!("serial:{}:{}", path, *baud),
            MavlinkConnectionType::Udp(address, port) => format!("udpin:{}:{}", address, *port),
            MavlinkConnectionType::Tcp(address, port) => format!("tcpout:{}:{}", address, *port),
        }
    }

    pub fn get_port(&self) -> u32 {
        match &self.connection {
            MavlinkConnectionType::Serial(_, port) => *port,
            MavlinkConnectionType::Udp(_, port) => *port,
            MavlinkConnectionType::Tcp(_, port) => *port,
        }
    }
}
