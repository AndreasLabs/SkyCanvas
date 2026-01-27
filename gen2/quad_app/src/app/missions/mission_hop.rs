use log::info;

use crate::{
    app::missions::QuadMissionTrait,
    common::{
        commands::{QuadAppCommand, QuadAppCommandType},
        context::QuadAppContext,
    },
    link::mav_mode::ArduMode,
};

pub struct MissionHop {}

impl MissionHop {
    pub fn new() -> Self {
        Self {}
    }
}

impl QuadMissionTrait for MissionHop {
    fn run(&mut self, context: &QuadAppContext) -> Result<(), anyhow::Error> {
        // Wait for quad health to be ok
        loop {
            let health_result = {
                let state = context.state.read().unwrap();
                state.ekf_status.is_healthy()
            };
            
            if let Err(e) = health_result {
                log::warn!("MissionHop // Waiting for quad health to be ok: {}", e);
                std::thread::sleep(std::time::Duration::from_millis(500));
            } else {
                break;
            }
        }
        log::info!("MissionHop // Quad health is ok");
        log::info!("MissionHop // Setting mode to GUIDED");
        // Set the mode to AUTO
        let mode_msg = ArduMode::Guided.build_mode_message();
        context
            .commands
            .lock()
            .unwrap()
            .push(QuadAppCommand::new(QuadAppCommandType::MavlinkRaw(
                mode_msg.unwrap(),
            )));

        // Wait 2s to allow the mode to set
        std::thread::sleep(std::time::Duration::from_millis(2000));
        info!("MissionHop // Arming quad");
        // Arm the quad
        let arm_cmd = mavlink::ardupilotmega::MavMessage::COMMAND_LONG(
            mavlink::ardupilotmega::COMMAND_LONG_DATA {
                param1: 1.0,
                param2: 21196., // 21196 is the code for arm/disarm forcefully
                param3: 0.0,
                param4: 0.0,
                param5: 0.0,
                param6: 0.0,
                param7: 0.0,
                command: mavlink::ardupilotmega::MavCmd::MAV_CMD_COMPONENT_ARM_DISARM,
                target_system: 0,
                target_component: 0,
                confirmation: 0,
            },
        );
        context
            .commands
            .lock()
            .unwrap()
            .push(QuadAppCommand::new(QuadAppCommandType::MavlinkRaw(
                arm_cmd,
            )));
        Ok(())
    }
}
