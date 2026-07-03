//! Route diagnostic output to a file when break-time is launched without a
//! terminal.
//!
//! When launched from Spotlight, Finder, or as a login item, macOS
//! (LaunchServices/launchd) wires the app's stdout and stderr to `/dev/null`, so
//! every `println!`/`eprintln!` — and any panic message — is silently discarded.
//! [`init`] detects that case and repoints file descriptors 1 and 2 at a
//! per-launch `~/Library/Logs/break-time.log`.
//!
//! When a terminal *is* attached (e.g. `cargo run`, or running the binary from a
//! shell) this is a no-op and output stays on stdout exactly as before.

#![allow(unsafe_code)]

use std::fs::{self, OpenOptions};
use std::io::IsTerminal;
use std::os::fd::AsRawFd;

use directories::BaseDirs;

/// Redirect stdout/stderr to `~/Library/Logs/break-time.log` (truncated on each
/// launch) unless a terminal is attached.
///
/// Best-effort: if the home directory can't be resolved or the file can't be
/// opened, the original file descriptors are left untouched. There is no usable
/// terminal to report such a failure to, and a missing debug log must never take
/// the app down.
pub fn init() {
    // A terminal is attached (e.g. `cargo run`): leave stdout/stderr alone.
    if std::io::stdout().is_terminal() {
        return;
    }

    let Some(base_dirs) = BaseDirs::new() else {
        return;
    };
    let log_path = base_dirs.home_dir().join("Library/Logs/break-time.log");

    // `~/Library/Logs` normally exists, but create it defensively.
    if let Some(parent) = log_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    // `truncate(true)` overwrites the previous session's log, so the file never
    // grows without bound across launches.
    let Ok(file) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&log_path)
    else {
        return;
    };

    // Point both stdout (fd 1) and stderr (fd 2) at the log file. `dup2` makes
    // them reference the file's open description, so `file`'s own descriptor can
    // drop at the end of this function without closing the log.
    let fd = file.as_raw_fd();
    unsafe {
        let _ = libc::dup2(fd, libc::STDOUT_FILENO);
        let _ = libc::dup2(fd, libc::STDERR_FILENO);
    }
}
