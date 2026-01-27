use mavlink::ardupilotmega::MavMessage;

pub fn mavlink_msg_type_str(msg: &MavMessage) -> String {
    let message_type = format!("{:?}", msg);
    // Extract just the enum variant name without the data
    let message_type = message_type
        .split('(')
        .next()
        .unwrap_or("UNKNOWN")
        .trim()
        .to_string();
    let message_type = message_type
        .split(' ')
        .next()
        .unwrap_or("UNKNOWN")
        .to_string();
    message_type
}

#[derive(Debug, Clone, Default)]
pub struct EkfStatus {
    pub attitude: bool,
    pub vel_horiz: bool,
    pub vel_vert: bool,
    pub pos_horiz_rel: bool,
    pub pos_horiz_abs: bool,
    pub pos_vert_abs: bool,
    pub pos_vert_agl: bool,
    pub const_pos_mode: bool,
    pub pred_pos_horiz_rel: bool,
    pub pred_pos_horiz_abs: bool,
    pub uninitialized: bool,
}

impl EkfStatus {
    pub fn from_flags(flags: mavlink::ardupilotmega::EkfStatusFlags) -> Self {
        Self {
            attitude: flags.intersects(mavlink::ardupilotmega::EkfStatusFlags::EKF_ATTITUDE),
            vel_horiz: flags.intersects(mavlink::ardupilotmega::EkfStatusFlags::EKF_VELOCITY_HORIZ),
            vel_vert: flags.intersects(mavlink::ardupilotmega::EkfStatusFlags::EKF_VELOCITY_VERT),
            pos_horiz_rel: flags.intersects(mavlink::ardupilotmega::EkfStatusFlags::EKF_POS_HORIZ_REL),
            pos_horiz_abs: flags.intersects(mavlink::ardupilotmega::EkfStatusFlags::EKF_POS_HORIZ_ABS),
            pos_vert_abs: flags.intersects(mavlink::ardupilotmega::EkfStatusFlags::EKF_POS_VERT_ABS),
            pos_vert_agl: flags.intersects(mavlink::ardupilotmega::EkfStatusFlags::EKF_POS_VERT_AGL),
            const_pos_mode: flags.intersects(mavlink::ardupilotmega::EkfStatusFlags::EKF_CONST_POS_MODE),
            pred_pos_horiz_rel: flags.intersects(mavlink::ardupilotmega::EkfStatusFlags::EKF_PRED_POS_HORIZ_REL),
            pred_pos_horiz_abs: flags.intersects(mavlink::ardupilotmega::EkfStatusFlags::EKF_PRED_POS_HORIZ_ABS),
            uninitialized: flags.intersects(mavlink::ardupilotmega::EkfStatusFlags::EKF_UNINITIALIZED),
        }
    }
    //TODO: This Error Types
    pub fn is_healthy(&self) -> Result<(), String> {
        if !self.attitude {
            return Err("EKF is not healthy - Attitude is not healthy".to_string());
        }

        if !self.vel_horiz {
            return Err("EKF is not healthy - Horizontal velocity is not healthy".to_string());
        }

        if !self.vel_vert {
            return Err("EKF is not healthy - Vertical velocity is not healthy".to_string());
        }

        if !self.pos_horiz_rel {
            return Err("EKF is not healthy - Horizontal position (relative) is not healthy".to_string());
        }

        if !self.pos_horiz_abs {
            return Err("EKF is not healthy - Horizontal position (absolute) is not healthy".to_string());
        }

        if !self.pos_vert_abs {
            return Err("EKF is not healthy - Vertical position (absolute) is not healthy".to_string());
        }

        if self.const_pos_mode {
            return Err("EKF is not healthy - EKF is in constant position mode".to_string());
        }

        if self.uninitialized {
            return Err("EKF is not healthy - EKF has not been initialized yet".to_string());
        }

        Ok(())
    }
}
