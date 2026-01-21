use mavlink::ardupilotmega::MavMessage;

pub type MavlinkMessageType = MavMessage;


pub struct MavQueues{
    tx: crossbeam_channel::Sender<MavlinkMessageType>,
    rx: crossbeam_channel::Receiver<MavlinkMessageType>,
}

impl MavQueues {
    pub fn new() -> Self {
        let (tx, rx) = crossbeam_channel::bounded(1000); 
        Self { tx, rx }
    }
}