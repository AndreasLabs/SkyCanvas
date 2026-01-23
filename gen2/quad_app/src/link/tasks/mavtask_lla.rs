use log::{debug, info};

use crate::{
    common::{context::QuadAppContext, state::{LLA, NED}},
    link::{mav_queues::MavlinkMessageType, tasks::MavTaskTrait},
};

pub struct MavTaskLla {}

impl MavTaskLla {
    pub fn new() -> Self {
        Self {}
    }
}

impl MavTaskTrait for MavTaskLla {
    fn handle_mavlink_message(
        &self,
        context: &QuadAppContext,
        message: MavlinkMessageType,
    ) -> Result<(), anyhow::Error> {
        let res_global_position_int = match message {
            MavlinkMessageType::GLOBAL_POSITION_INT(global_position_int_data) => {
                global_position_int_data
            }
            _ => return Ok(()),
        };
        let mut state = context.state.write().unwrap();
        let lla = LLA {
            latitude: (res_global_position_int.lat as f32) / 1e7,
            longitude: (res_global_position_int.lon as f32) / 1e7,
            altitude: (res_global_position_int.alt as f32) / 1000.0,
        };
        state.record_lla(lla);
        let log_rerun = context.log_rerun.lock().unwrap();
        log_rerun.log_lla("mavlink/position/lla", &state.lla_current)?;

        debug!("MavTaskLla // Received global position int: {:?}", res_global_position_int);
        Ok(())
    }
}
