use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: String,
}
