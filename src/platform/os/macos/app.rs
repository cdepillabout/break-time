//! macOS app runtime: an `NSApplication` run loop plus a plain `mpsc` channel
//! whose receiver is drained on the main thread by a repeating `NSTimer`. This
//! backs the cross-platform app-runtime seam (`platform::{channel,
//! run_main_loop, quit, AppSender, AppReceiver}`); the Linux counterpart is
//! `platform::os::linux::app`.
//!
//! Unlike the Linux glib channel (a send wakes the loop), this polls: a
//! low-frequency `NSTimer` drains whatever `Msg`s have arrived. The latency is
//! imperceptible for break-time's traffic (a tray redraw ~1×/s, break
//! start/end), and it keeps `AppSender` a trivially `Send + Clone` wrapper with
//! no run-loop pointers to marshal across threads. The timer runs a few times a
//! second with a generous tolerance so macOS coalesces the wakeups; a fully
//! event-driven wakeup (a `CFRunLoopSource` the sender signals) is tracked as a
//! follow-up in the Phase 4 backlog in TODO.md.

#![allow(unsafe_code)]

use std::cell::RefCell;
use std::ptr::NonNull;
use std::sync::mpsc::{Receiver, SendError, Sender};

use block2::RcBlock;
use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSEvent,
    NSEventModifierFlags, NSEventType,
};
use objc2_foundation::{NSPoint, NSTimer};

use crate::Msg;

/// How often the main thread drains pending `Msg`s. Deliberately low-frequency
/// (not a tight poll) and paired with a tolerance so macOS can coalesce the
/// wakeups; the latency is imperceptible for this app's message traffic.
const DRAIN_INTERVAL: f64 = 0.25;

/// Slack allowed on [`DRAIN_INTERVAL`] so macOS can batch the wakeup with other
/// timers (a power optimization).
const DRAIN_TOLERANCE: f64 = 0.1;

/// Sender half of the app message channel. Cloneable and `Send`, so the tray,
/// scheduler thread, and break window can all hold one.
#[derive(Clone)]
pub struct AppSender(Sender<Msg>);

impl AppSender {
    /// Queue a `Msg` for the main thread. Mirrors `glib::Sender::send` (and its
    /// `Result`), so call sites are identical across platforms.
    pub fn send(&self, msg: Msg) -> Result<(), SendError<Msg>> {
        self.0.send(msg)
    }
}

/// Receiver half, consumed by [`run_main_loop`] on the main thread.
pub struct AppReceiver(Receiver<Msg>);

/// Create the app message channel. (Unlike Linux there is no GUI toolkit to
/// initialize here; `NSApplication` is set up in [`run_main_loop`].)
#[must_use]
pub fn channel() -> (AppSender, AppReceiver) {
    let (tx, rx) = std::sync::mpsc::channel();
    (AppSender(tx), AppReceiver(rx))
}

/// Run the `NSApplication` loop, draining `receiver` and invoking `handler` on
/// the main thread for each `Msg`, until [`quit`] is called.
pub fn run_main_loop(
    receiver: AppReceiver,
    handler: impl FnMut(Msg) + 'static,
) {
    let mtm = MainThreadMarker::new()
        .expect("run_main_loop must be called on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    // A background / menu-bar app: no Dock icon, no menu bar of its own.
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    // Interior mutability so the `Fn` block can drive an `FnMut` handler.
    let handler = RefCell::new(handler);
    let block = RcBlock::new(move |_timer: NonNull<NSTimer>| {
        while let Ok(msg) = receiver.0.try_recv() {
            (&mut *handler.borrow_mut())(msg);
        }
    });
    // A low-frequency timer with tolerance (rather than a tight poll) so macOS
    // coalesces the wakeups; the run loop retains the timer for its lifetime.
    let timer = unsafe {
        NSTimer::scheduledTimerWithTimeInterval_repeats_block(
            DRAIN_INTERVAL,
            true,
            &block,
        )
    };
    timer.setTolerance(DRAIN_TOLERANCE);

    app.run();
}

/// Stop the run loop; causes [`run_main_loop`] to return. Must be called on the
/// main thread — it is, from the drain handler.
pub fn quit() {
    let mtm = MainThreadMarker::new()
        .expect("quit must be called on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.stop(None);
    // `-stop:` only takes effect after the run loop finishes dispatching an
    // *event* — and we are called from a timer callback, which is not one. Post
    // a do-nothing application-defined event so the loop wakes, dispatches it,
    // notices the stop flag, and actually exits.
    let wake_event =
        NSEvent::otherEventWithType_location_modifierFlags_timestamp_windowNumber_context_subtype_data1_data2(
            NSEventType::ApplicationDefined,
            NSPoint::ZERO,
            NSEventModifierFlags::empty(),
            0.0,
            0,
            None,
            0,
            0,
            0,
        )
        .expect("could not create the wake-up event");
    app.postEvent_atStart(&wake_event, true);
}
