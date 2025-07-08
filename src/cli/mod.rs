use std::path::PathBuf;
use clap::{Parser, Subcommand};
use tracing::{info, warn, error, debug, trace};
use crate::runner::pipeline::load_pipeline;
use crate::runner::executor::run_pipeline;

#[derive(Parser)]
#[command(name  = "gaia")]
#[command(about = "GaiaCI - Ligthweight Lua-based CI Runner", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        #[arg(default_value = ".gaiaci/pipeline.lua")]
        file: PathBuf,
    },
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => {
            info!("GaiaCI Pipeline loading from {:?}", file);
            match load_pipeline(&file) {
                Ok((lua, pipeline)) => {
                    info!("GaiaCI pipeline loaded successfully");
                    if let Err(e) = run_pipeline(&lua, pipeline) {
                        error!("GaiaCI pipeline execution failed: {}", e);
                        return Err(e);
                    }
                    info!("GaiaCI pipeline execution finished")
                }
                Err(e) => {
                    error!("GaiCI pipeline failed to load: {}", e)
                }
            }
        }
    }
    Ok(())
}