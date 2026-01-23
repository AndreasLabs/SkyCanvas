use crate::link::{mav_config::MavConfig, mav_queues::MavQueues};


use log::{debug, error, info, trace};
use mavlink::{ardupilotmega::MavMessage};
use std::{
    sync::{
        Arc, Mutex, atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender, channel}
    },
    thread,
    time::Duration,
};

type MavlinkMessageType = MavMessage;


pub struct MavIO{
    config: MavConfig,
    mav_con: Option<Box<dyn mavlink::MavConnection<MavlinkMessageType> + Send + Sync>>,
    enabled: AtomicBool,
    queues: MavQueues,
}

impl MavIO{
    pub fn new(config: MavConfig, queues: MavQueues) -> Self {
        Self { config, mav_con: None, enabled: AtomicBool::new(false), queues }
    }   

    pub fn start(&mut self) -> Result<(), anyhow::Error> {
        self.enabled.store(true, Ordering::Relaxed);
        info!("SkyCanvas // MavIO // Connecting to MAVLink: {}", self.config.connection_string());
        let mut mav_con = mavlink::connect::<MavlinkMessageType>(&self.config.connection_string().as_str())?;
        self.mav_con = Some(Box::new(mav_con));

        info!("SkyCanvas // MavIO // Setting protocol version to V2");
        let mav_con = self.mav_con.as_mut().unwrap();
        mav_con.set_protocol_version(mavlink::MavlinkVersion::V2);
        self.send_request_stream()?;
        info!("SkyCanvas // MavIO // Starting IO Tick loop");
        while self.enabled.load(Ordering::Relaxed) {

            //  First on each tick - send out any commands that are sent to IO by the quad app
            self.tick_send()?;
            // 2. Recv any messages from the MAVLink connection
            self.tick_recv()?;

            // For now rate limit by adding 10ms
            thread::sleep(Duration::from_millis(10));
        }
       
        Ok(())
    }

    fn tick_send(&mut self) -> Result<(), anyhow::Error> {
        let commands = match self.queues.recv() {
            Ok(Some(msg)) => msg,
            Ok(None) => return Ok(()),
            Err(e) => {
                error!("SkyCanvas // MavIO // Error receiving message: {}", e);
                return Err(anyhow::anyhow!("Error receiving message: {}", e));
            }
        };
        let mav_con = self.mav_con.as_ref().unwrap();
        mav_con.send(&mavlink::MavHeader::default(), &commands)?;
        Ok(())
    }

    fn tick_recv(&self) -> Result<(), anyhow::Error> {
        let mav_con = self.mav_con.as_ref().unwrap();
        match mav_con.try_recv(){
            Ok(msg) => {
                //info!("SkyCanvas // MavIO // Received message: {:#?}", msg);
             //   let message_type = crate::common::mavlink_helpers::mavlink_msg_type_str(&msg.1.clone());
                //trace!("SkyCanvas // MavIO // Received message: {}", message_type);
                self.queues.send(msg.1)?;
                Ok(())
            },
            Err(mavlink::error::MessageReadError::Io(e)) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    // No messages currently available to receive return Ok
                    //debug!("SkyCanvas // MavIO // No messages currently available to receive");
                    Ok(())
                } else{
                    error!("SkyCanvas // MavIO // IO Error: {}", e);
                    Err(anyhow::anyhow!("IO Error: {}", e))
                }
            },
            Err(mavlink::error::MessageReadError::Parse(e)) => {
                error!("SkyCanvas // MavIO // Parse Error: {}", e);
                Err(anyhow::anyhow!("Parse Error: {}", e))
            }
        }
    }
  

    fn send_request_stream(&self) -> Result<(), anyhow::Error> {
        #[allow(deprecated)]
        let packet = MavMessage::REQUEST_DATA_STREAM(mavlink::ardupilotmega::REQUEST_DATA_STREAM_DATA {
            target_system: 0,
            target_component: 0,
            req_stream_id: 0,
            req_message_rate: self.config.telemetry_rate_hz as u16,
            start_stop: 1,
        });
        info!("SkyCanvas // MavIO // Sending request stream: {:#?}", packet);
        self.queues.send(packet)?;
        Ok(())
    }
}