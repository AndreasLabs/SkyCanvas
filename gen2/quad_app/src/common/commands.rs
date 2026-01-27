use crate::link::mav_queues::MavQueues;
use mavlink::ardupilotmega::MavMessage;
#[derive(Clone, Debug)]
pub enum QuadAppCommandType{
    MavlinkRaw(MavMessage),
    QuadGuidedArm(),
    QuadTakeoff(),
}


#[derive(Clone, Debug)]
pub struct QuadAppCommand{
    pub cmd_type: QuadAppCommandType,
}

impl QuadAppCommand{
    pub fn new(cmd_type: QuadAppCommandType) -> Self {
        Self {
            cmd_type,
        }
    }
}