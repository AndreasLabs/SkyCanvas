pub mod mav_io;
pub mod mav_tasks;
pub mod tasks;

use mav_io::MavIO;
use mav_tasks::MavTasks;

use mavlink::ardupilotmega::MavMessage;

use tokio::sync::mpsc;
pub struct QuadLink{
    io: MavIO,
    tasks: MavTasks,
}

pub type MavlinkMessageType = MavMessage;
pub type MavlinkQueues = (mpsc::Sender<MavlinkMessageType>, mpsc::Receiver<MavlinkMessageType>);

impl QuadLink{
    pub fn new() -> Self {
        Self {
            io: MavIO::new(),
            tasks: MavTasks::new(),
        }
    }
}