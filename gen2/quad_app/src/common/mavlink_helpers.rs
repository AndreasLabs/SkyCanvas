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