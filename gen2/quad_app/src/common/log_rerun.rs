use crate::common::state::LLA;

pub struct LogRerun {
    pub name: String,
    pub rec: rerun::RecordingStream,
}

impl LogRerun {
    pub fn new(name: String) -> Self {
        let rec = rerun::RecordingStreamBuilder::new(name.clone())
            .spawn()
            .unwrap();
        Self { name, rec: rec }
    }

    pub fn log_status_text(&self, topic: &str, status_text: &str) -> Result<(), anyhow::Error> {
        log::info!("LogRerun // MAVLINK: {}", status_text);
        self.rec.log(
            topic.to_string(),
            &rerun::TextLog::new(status_text.to_string()).with_level(rerun::TextLogLevel::INFO),
        )?;
        Ok(())
    }

    pub fn log_lla(&self, topic: &str, lla: &LLA) -> Result<(), anyhow::Error> {
        self.rec.log(
            topic.to_string(),
            &rerun::GeoPoints::from_lat_lon(&[(lla.latitude as f64, lla.longitude as f64)])
                .with_radii([rerun::Radius::new_ui_points(5.0)])
                .with_colors([rerun::Color::from_rgb(255, 0, 0)]),
        )?;
        Ok(())
    }
}
