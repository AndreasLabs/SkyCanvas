use clap::Parser;
use log::info;
mod shell;
use crate::shell::Shell;
#[derive(Parser)]
pub enum DockerCommand {
    Build,
    Up,
    Down,
}

#[derive(Parser)]
pub enum RepoCliCommand {
    Docker {
        #[clap(subcommand)]
        command: DockerCommand,
    },
}

#[derive(Parser)]
pub struct RepoCliArgs {
    #[clap(subcommand)]
    command: RepoCliCommand,
}

fn main() {
    pretty_env_logger::init();
    info!("Starting Repo CLI...");
    let args = RepoCliArgs::parse();
    
    // Get the crate via CARGO_MANIFEST_DIR
    let crate_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let repo_root = std::path::Path::new(&crate_root).parent().unwrap();
    let repo_root = repo_root.to_path_buf();
    match args.command {
        RepoCliCommand::Docker { command } => match command {
            DockerCommand::Build => {
                // run 
                info!("Building Docker container...");
                Shell::new("docker build -t ardupilot-sil -f ardupilot_sil/Dockerfile .")
                    .with_cwd(repo_root.clone())
                    .run()
                    .expect("Failed to build Docker container");
    
            }
            DockerCommand::Up => {
                // run 'docker compose -f docker/compose.sil.yml up -d'
                info!("Starting Docker container...");
                Shell::new("docker compose -f docker/compose.sil.yml up -d")
                    .with_cwd(repo_root.clone())
                    .run()
                    .expect("Failed to start Docker container");

            }
            DockerCommand::Down => {
                info!("Stopping Docker container...");
                Shell::new("docker compose -f docker/compose.sil.yml down")
                    .with_cwd(repo_root.clone())
                    .run()
                    .expect("Failed to stop Docker container");
            }
        },
    }
}