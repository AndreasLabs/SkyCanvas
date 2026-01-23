use log::{debug, info};

use crate::{
    common::{context::QuadAppContext, mavlink_helpers::EkfStatus, state::{LLA, NED}},
    link::{mav_queues::MavlinkMessageType, tasks::MavTaskTrait},
};

pub struct MavTaskHealth {}

impl MavTaskHealth {
    pub fn new() -> Self {
        Self {}
    }
}

impl MavTaskTrait for MavTaskHealth {
    fn handle_mavlink_message(
        &self,
        context: &QuadAppContext,
        message: MavlinkMessageType,
    ) -> Result<(), anyhow::Error> {
        let res_ekf_status_report = match message {
            MavlinkMessageType::EKF_STATUS_REPORT(ekf_status_report_data) => {
                ekf_status_report_data
            }
            _ => return Ok(()),
        };
    
        let mut state = context.state.write().unwrap();
        let efk_status = EkfStatus::from_flags(res_ekf_status_report.flags);
        state.ekf_status = efk_status;
        debug!("MavTaskHealth // Updated EKF status: {:?}", state.ekf_status);
        Ok(())
    }
}
