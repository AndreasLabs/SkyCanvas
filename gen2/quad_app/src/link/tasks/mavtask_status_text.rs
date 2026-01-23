use log::info;

use crate::{common::context::QuadAppContext, link::{mav_queues::MavlinkMessageType, tasks::MavTaskTrait}};

pub struct MavTaskStatusText{

}

impl MavTaskStatusText{
    pub fn new() -> Self {
        Self {}
    }
}

impl MavTaskTrait for MavTaskStatusText{
    fn handle_mavlink_message(&self,context: &QuadAppContext, message: MavlinkMessageType) -> Result<(), anyhow::Error> {
        
        match message{
            MavlinkMessageType::STATUSTEXT(status_text_data) => {
                let serverity = match status_text_data.severity {
                    mavlink::ardupilotmega::MavSeverity::MAV_SEVERITY_EMERGENCY => "EMERGENCY",
                    mavlink::ardupilotmega::MavSeverity::MAV_SEVERITY_ALERT => "ALERT",
                    mavlink::ardupilotmega::MavSeverity::MAV_SEVERITY_CRITICAL => "CRITICAL",
                    mavlink::ardupilotmega::MavSeverity::MAV_SEVERITY_ERROR => "ERROR",
                    mavlink::ardupilotmega::MavSeverity::MAV_SEVERITY_WARNING => "WARNING",
                    mavlink::ardupilotmega::MavSeverity::MAV_SEVERITY_NOTICE => "NOTICE",
                    mavlink::ardupilotmega::MavSeverity::MAV_SEVERITY_INFO => "INFO",
                    mavlink::ardupilotmega::MavSeverity::MAV_SEVERITY_DEBUG => "DEBUG",
                };
                let msg = String::from_utf8_lossy(&status_text_data.text.to_vec()).to_string();
                // Trim \0's
                let msg = msg.trim_matches('\0').to_string();
                info!("Task // Status Text // {:?} -> {:?}", serverity, msg);
                let log_rerun = context.log_rerun.lock().unwrap();
                log_rerun.log_status_text("mavlink/status_text", &msg)?;
                Ok(())
            }
            _ => {
                Ok(())
            }
        }
    }
}