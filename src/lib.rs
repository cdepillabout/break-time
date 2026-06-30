#![deny(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(
    // Explicit `&` borrows are kept for readability; the removals this lint
    // suggests are stylistic and not worth the churn.
    clippy::needless_borrow,
    // We prefer the readable `.expect(&format!(...))` form over
    // `unwrap_or_else(|| panic!(...))`. None of these are hot paths, so
    // eagerly building the message even on the success path is fine.
    clippy::expect_fun_call,
    // A `match` with one real arm plus `_` often reads more clearly than the
    // `if let ... else` this lint wants.
    clippy::single_match_else,
    // `Settings.all_plugin_settings` ends with its type name (`PluginSettings`),
    // but the field name is clear and renaming it gains nothing.
    clippy::struct_field_names,
    // break-time is a binary, not a published library: `run` / `default_main` /
    // `start_break` are internal entry points, so `# Panics` doc sections would
    // be pure noise.
    clippy::missing_panics_doc,
    // We intentionally `{:?}`-format `PathBuf`s in panic/error messages — the
    // quoting and escaping is useful when diagnosing a bad path, and these are
    // developer-facing panics, not user-facing output.
    clippy::unnecessary_debug_formatting,
    // This nursery lint rewrites `if let { ... } else { ... }` into
    // `map_or_else` with two side-effecting closures, which is less readable
    // than the explicit form in the places it fires here.
    clippy::option_if_let_else
)]

mod config;
mod opts;
mod platform;
mod prelude;
mod scheduler;
mod x11;

use std::sync::mpsc::Sender;
use std::time::Duration;

use config::Config;
use platform::{IsIdleDetectorEnabled, TrayImpl as Tray};
use scheduler::Scheduler;

#[derive(Clone, Copy, Debug)]
pub enum Msg {
    EndBreak,
    Pause,
    Quit,
    ResetSysTrayIcon,
    Resume,
    StartBreak,
    TimeRemainingBeforeBreak(Duration),
    EnableIdleDetector,
    DisableIdleDetector,
}

fn handle_msg_recv(
    config: &Config,
    sender: glib::Sender<Msg>,
    scheduler_outer_sender: &Sender<scheduler::Msg>,
    scheduler_inner_sender: &Sender<scheduler::InnerMsg>,
    tray: &mut Tray,
    msg: Msg,
) {
    match msg {
        Msg::EndBreak => {
            println!("break ended");
            tray.break_end();
            scheduler_outer_sender.send(scheduler::Msg::Start).expect("TODO: figure out what to do about channels potentially failing");
        }
        Msg::Pause => {
            tray.pause();
            scheduler_inner_sender.send(scheduler::InnerMsg::Pause).expect("TODO: figure out what to do about channels potentially failing");
        }
        Msg::Quit => {
            gtk::main_quit();
        }
        Msg::StartBreak => {
            println!("starting break");
            tray.render_break_starting();
            platform::start_break(config, sender);
        }
        Msg::ResetSysTrayIcon => {
            tray.render_normal_icon();
        }
        Msg::Resume => {
            tray.resume();
            scheduler_outer_sender.send(scheduler::Msg::Start).expect("TODO: figure out what to do about channels potentially failing");
        }
        Msg::TimeRemainingBeforeBreak(remaining_time) => {
            tray.update_time_remaining(remaining_time);
        }
        Msg::EnableIdleDetector => {
            tray.set_is_idle_detector_enabled(IsIdleDetectorEnabled::Yes);
            scheduler_inner_sender.send(scheduler::InnerMsg::EnableIdleDetector).expect("TODO: figure out what to do about channels potentially failing");
        }
        Msg::DisableIdleDetector => {
            tray.set_is_idle_detector_enabled(IsIdleDetectorEnabled::No);
            scheduler_inner_sender.send(scheduler::InnerMsg::DisableIdleDetector).expect("TODO: figure out what to do about channels potentially failing");
        }
    }
}

pub fn run(config: Config) {
    gtk::init().expect("Could not initialize GTK");

    // `MainContext::channel` is deprecated in glib 0.18 (removed in 0.20) in favour of
    // async-channel + `spawn_future_local`.  Migrating the synchronous `Msg`/`Message` plumbing to
    // async channels is deferred to a later glib/GTK upgrade; suppress the deprecation for now.
    #[allow(deprecated)]
    let (sender, receiver) =
        glib::MainContext::channel(glib::Priority::DEFAULT);

    let mut tray = Tray::run(&config, sender.clone());

    println!("Starting the scheduler...");
    let (scheduler_outer_sender, scheduler_inner_sender) =
        Scheduler::run(&config, sender.clone());

    receiver.attach(None, move |msg| {
        handle_msg_recv(
            &config,
            sender.clone(),
            &scheduler_outer_sender,
            &scheduler_inner_sender,
            &mut tray,
            msg,
        );
        glib::ControlFlow::Continue
    });

    gtk::main();
}

pub fn run_google_calendar_command(
    config: &Config,
    google_calendar_command: opts::GoogleCalendar,
) {
    match google_calendar_command {
        opts::GoogleCalendar::ListEvents => {
            scheduler::plugins::google_calendar::list_events(&config);
        }
        opts::GoogleCalendar::IgnoreEvent(opts::IgnoreEvent { event_id }) => {
            scheduler::plugins::google_calendar::ignore_event(
                &config, &event_id,
            );
        }
    }
}

pub fn default_main() {
    let opts = opts::Opts::parse_from_args();

    let config = Config::load(&opts).expect("Could not load config file.");

    match opts.cmd {
        None => run(config),
        Some(opts::Command::GoogleCalendar(google_calendar_command)) => {
            run_google_calendar_command(&config, google_calendar_command);
        }
    }
}
