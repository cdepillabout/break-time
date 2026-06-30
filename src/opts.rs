use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(about = "Force yourself to take regular breaks")]
pub struct Opts {
    /// This is the directory to hold the break-time configuration file.  Defaults to
    /// $XDG_CONFIG_HOME/break-time/ (or ~/.config/break-time/ if $`XDG_CONFIG_HOME` is not set).
    #[arg(long, value_name = "CONFIG_DIR_PATH")]
    pub conf_dir: Option<PathBuf>,

    /// This is the directory to hold the break-time cache data.  Defaults to
    /// $XDG_CACHE_HOME/break-time/ (or ~/.cache/break-time/ if $`XDG_CACHE_HOME` is not set).
    #[arg(long, value_name = "CACHE_DIR_PATH")]
    pub cache_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub cmd: Option<Command>,
}

impl Opts {
    pub fn parse_from_args() -> Self {
        Self::parse()
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(subcommand)]
    GoogleCalendar(GoogleCalendar),
}

#[derive(Debug, Subcommand)]
pub enum GoogleCalendar {
    ListEvents,
    IgnoreEvent(IgnoreEvent),
}

#[derive(Debug, Args)]
pub struct IgnoreEvent {
    /// Event ID.  You can get this with `break-time google-calendar list-events`.
    pub event_id: String,
}
