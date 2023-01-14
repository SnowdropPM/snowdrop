use clap::{Parser, Subcommand};
use log::debug;
use miette::Result;

use crate::commands::*;
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Install a package.
    Install {
        /// The name of the package you want to install
        package: String,

        /// Whether to run a "dry-run". If this flag is set, then no
        /// files will be written
        #[clap(long)]
        dry_run: bool,
    },
}

impl Command {
    pub async fn execute(&self) -> Result<()> {
        debug!("Command invoked: {:#?}", self);
        match self {
            Self::Install { dry_run, package } => install::Install::execute(package, dry_run).await,
        }
    }
}
