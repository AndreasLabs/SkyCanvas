use mavlink::ardupilotmega::MavMessage;

pub type MavlinkMessageType = MavMessage;

#[derive(Debug, Clone)]
pub struct MavQueues{
    tx: crossbeam_channel::Sender<MavlinkMessageType>,
    rx: crossbeam_channel::Receiver<MavlinkMessageType>,
}

impl MavQueues {
    pub fn new() -> Self {
        let (tx, rx) = crossbeam_channel::bounded(1000); 
        Self { tx, rx }
    }

    pub fn send(&self, message: MavlinkMessageType) -> Result<(), anyhow::Error> {
        self.tx.send(message)?;
        Ok(())
    }

    pub fn recv(&self) -> Result<Option<MavlinkMessageType>, anyhow::Error> {
        let message = self.rx.try_recv();
        match message {
            Ok(message) => Ok(Some(message)),
            Err(crossbeam_channel::TryRecvError::Empty) => Ok(None),
            Err(crossbeam_channel::TryRecvError::Disconnected) => Err(anyhow::anyhow!("Channel disconnected")),
        }
    }
}

