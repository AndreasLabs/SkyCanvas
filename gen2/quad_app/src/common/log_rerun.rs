pub struct LogRerun{
    pub name: String,
    pub rec: rerun::RecordingStream,
}

impl LogRerun{
    pub fn new(name: String) -> Self {
        let rec = rerun::RecordingStreamBuilder::new(name.clone()).spawn().unwrap();
        Self { name, rec: rec }
    }


    pub fn log_status_text(&self, topic: &str, status_text: &str) -> Result<(), anyhow::Error> {
        log::info!("LogRerun // Logging status text // {}", status_text);

 
        self.rec.log(topic.to_string(), &rerun::TextLog::new(status_text.to_string()).with_level(rerun::TextLogLevel::INFO))?;
        Ok(())
    }
}