use log::{debug, info};

use crate::{
    common::{context::QuadAppContext, state::NED},
    link::{mav_queues::MavlinkMessageType, tasks::MavTaskTrait},
};

pub struct MavTaskLocalNed {}

impl MavTaskLocalNed {
    pub fn new() -> Self {
        Self {}
    }
}

impl MavTaskTrait for MavTaskLocalNed {
    fn handle_mavlink_message(
        &self,
        context: &QuadAppContext,
        message: MavlinkMessageType,
    ) -> Result<(), anyhow::Error> {
        let res_local_position = match message {
            MavlinkMessageType::LOCAL_POSITION_NED(local_position_ned_data) => {
                local_position_ned_data
            }
            _ => return Ok(()),
        };
        let mut state = context.state.write().unwrap();
        let ned_pos = NED::new(
            res_local_position.x,
            res_local_position.y,
            res_local_position.z,
        );
        state.record_ned(ned_pos);

        debug!("MavTaskLocalNed // Received local position NED: {:?}", res_local_position);
        Ok(())
    }
}
