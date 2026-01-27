use crate::link::mav_queues::MavQueues;
use mavlink::ardupilotmega::MavMessage;

pub enum QuadAppCommandType{
    MavlinkRaw(MavMessage),
    QuadGuidedArm(),
    QuadTakeoff(),
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