// TEMPORARY STUB (Phase 1e in progress).
//
// The old synchronous google-calendar3 1.0 / hyper 0.10 / yup-oauth2 1.0 stack does not compile on
// modern Rust, so it has been replaced in Cargo.toml by google-calendar3 7.0 + yup-oauth2 12 +
// hyper-util + tokio (async).  This module still needs to be rewritten against that async API
// (driven from the synchronous `Plugin` via a `tokio` runtime + `block_on`).  Until then this stub
// keeps the crate compiling and behaves as "always allow a break" (i.e. the calendar never blocks
// a break).
//
// TODO(phase-1e): reimplement against google-calendar3 7.0 / yup-oauth2 12 (read the extracted crate
// source under ~/.cargo/registry for the exact API), restoring:
//   - per-account CalFetcher with an InstalledFlow authenticator + on-disk token cache
//   - has_events() over the [-10min, +20min] window
//   - filter_event() (ignore "ignore break-time" / OOO / cancelled / declined events)
//   - the `google-calendar list-events` and `google-calendar ignore-event` CLI subcommands

use super::{CanBreak, Plugin};

use crate::config::Config;

pub struct GoogleCalendar;

impl GoogleCalendar {
    // The real (async) `new` is fallible (auth / network / token cache), so it returns `Result`.
    // This stub can't fail, which trips `unnecessary_wraps`; keep the fallible signature so the
    // call site (`Plugins::new`) is unaffected when the real plugin lands (Phase 1e).
    #[allow(clippy::unnecessary_wraps)]
    pub const fn new(_config: &Config) -> Result<Self, ()> {
        Ok(Self)
    }
}

impl Plugin for GoogleCalendar {
    fn can_break_now(&self) -> Result<CanBreak, Box<dyn std::error::Error>> {
        Ok(CanBreak::Yes)
    }
}

pub fn list_events(_config: &Config) {
    println!(
        "`google-calendar list-events` is temporarily unavailable while the \
         Google Calendar plugin is being migrated to the async API (Phase 1e)."
    );
}

pub fn ignore_event(_config: &Config, _event_id: &str) {
    println!(
        "`google-calendar ignore-event` is temporarily unavailable while the \
         Google Calendar plugin is being migrated to the async API (Phase 1e)."
    );
}
