use crate::link::MavlinkMessageType;

pub enum QuadAppCommandType{
    MavlinkRaw(MavlinkMessageType),
}


pub struct QuadAppCommand{
    cmd_type: QuadAppCommandType,
}

impl QuadAppCommand{
    pub fn new(cmd_type: QuadAppCommandType) -> Self {
        Self {
            cmd_type,
        }
    }
}