use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Force yourself to take regular breaks")]
pub struct Opts {
    /// This is the directory to hold the break-time configuration file.  Defaults to
    /// $XDG_CONFIG_HOME/break-time/ (or ~/.config/break-time/ if $XDG_CONFIG_HOME is not set).
    #[structopt(long, name = "CONFIG_DIR_PATH", parse(from_os_str))]
    pub conf_dir: Option<PathBuf>,

    /// This is the directory to hold the break-time cache data.  Defaults to
    /// $XDG_CACHE_HOME/break-time/ (or ~/.cache/break-time/ if $XDG_CACHE_HOME is not set).
    #[structopt(long, name = "CACHE_DIR_PATH", parse(from_os_str))]
    pub cache_dir: Option<PathBuf>,

    #[structopt(subcommand)]
    pub cmd: Option<Command>
}

impl Opts {
    pub fn parse_from_args() -> Self {
        Self::from_args()
    }
}

#[derive(Debug, StructOpt)]
pub enum Command {
    GoogleCalendar(GoogleCalendar),
}

#[derive(Debug, StructOpt)]
pub enum GoogleCalendar {
    ListEvents,
}
