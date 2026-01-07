use std::process::Command;
use log::{info, error};
use std::error::Error;
use std::path::PathBuf;

pub struct Shell{
    command: String,
    cwd: Option<PathBuf>,
}
impl Shell {
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            cwd: None,
        }
    }

    pub fn with_cwd(mut self, cwd: PathBuf) -> Self {
        self.cwd = Some(cwd);
        self
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        info!("Running command: {}", self.command);
        
        // Parse command and arguments properly
        let parts: Vec<&str> = self.command.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty command".into());
        }
        
        let (cmd, args) = parts.split_first().unwrap();
        
        let mut command = Command::new(cmd);
        command.args(args);
        
        // Set current directory if provided
        if let Some(ref cwd) = self.cwd {
            command.current_dir(cwd);
        }
        
        let output = command.output()?;
        
        // Check exit status
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Command failed with status {}: {}", output.status, stderr);
            return Err(format!("Command failed: {}", stderr).into());
        }
        
        // Log output only if non-empty
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            info!("Output: {}", stdout.trim());
        }
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            error!("Stderr: {}", stderr.trim());
        }
        
        Ok(())
    }
}