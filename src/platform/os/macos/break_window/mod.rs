//! macOS break window: one borderless full-screen shield `NSWindow` per
//! `NSScreen`, with a centered countdown, driven by an `NSTimer`. This is the
//! Cocoa counterpart to the GTK break window under
//! `platform::os::linux::break_window`.
//!
//! MVP scope: the break ends when the countdown elapses. The `CGEventTap`-based
//! input lock and spacebar-to-end (both native, both requiring the Accessibility
//! permission) land in a follow-up commit; until then other apps' input is not
//! blocked — macOS has no user-space input grab, see the TODO plan — but the
//! screen is covered and the break runs to completion.

#![allow(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSBackingStoreType, NSColor, NSFont, NSScreen,
    NSTextAlignment, NSTextField, NSWindow, NSWindowCollectionBehavior,
    NSWindowStyleMask,
};
use objc2_core_graphics::CGShieldingWindowLevel;
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString, NSTimer};

use crate::config::Config;
use crate::platform::AppSender;
use crate::Msg;

/// How often the countdown label refreshes / the end condition is checked.
const TICK_INTERVAL: f64 = 0.2;

/// Point size of the countdown text.
const COUNTDOWN_FONT_SIZE: f64 = 140.0;

/// Height of the countdown label's frame within each screen.
const LABEL_HEIGHT: f64 = 180.0;

struct BreakState {
    windows: Vec<Retained<NSWindow>>,
    labels: Vec<Retained<NSTextField>>,
    start: Instant,
    duration: Duration,
    app_sender: AppSender,
    ended: Cell<bool>,
}

pub fn start_break(config: &Config, app_sender: AppSender) {
    let mtm = MainThreadMarker::new()
        .expect("start_break must run on the main thread");
    let app = NSApplication::sharedApplication(mtm);

    let mut windows = Vec::new();
    let mut labels = Vec::new();
    // `NSArray` is iterated via its `iter()`; there is no by-reference form.
    #[allow(clippy::explicit_iter_loop)]
    for screen in NSScreen::screens(mtm).iter() {
        let frame = screen.frame();
        let (window, label) = make_shield_window(mtm, frame);
        window.makeKeyAndOrderFront(None);
        windows.push(window);
        labels.push(label);
    }
    // Bring our (accessory) app to the front so the shield windows are on top.
    #[allow(deprecated)]
    app.activateIgnoringOtherApps(true);

    let state = Rc::new(BreakState {
        windows,
        labels,
        start: Instant::now(),
        duration: Duration::from_secs(
            config.settings.break_duration_seconds.into(),
        ),
        app_sender,
        ended: Cell::new(false),
    });

    // Paint the initial time immediately (the first tick is TICK_INTERVAL away).
    refresh(&state);

    let state_cb = Rc::clone(&state);
    let block = RcBlock::new(move |timer: std::ptr::NonNull<NSTimer>| {
        tick(&state_cb, timer);
    });
    // The run loop retains the scheduled timer; the timer owns the block, which
    // owns `state`. Invalidating the timer at break end tears all of it down.
    let _timer = unsafe {
        NSTimer::scheduledTimerWithTimeInterval_repeats_block(
            TICK_INTERVAL,
            true,
            &block,
        )
    };
}

fn tick(state: &BreakState, timer: std::ptr::NonNull<NSTimer>) {
    if state.ended.get() {
        return;
    }
    if state.start.elapsed() >= state.duration {
        end_break(state, timer);
    } else {
        refresh(state);
    }
}

fn refresh(state: &BreakState) {
    let remaining = state.duration.saturating_sub(state.start.elapsed());
    let total_secs = remaining.as_secs();
    let text = format!("{:02}:{:02}", total_secs / 60, total_secs % 60);
    let ns_text = NSString::from_str(&text);
    for label in &state.labels {
        label.setStringValue(&ns_text);
    }
}

fn end_break(state: &BreakState, timer: std::ptr::NonNull<NSTimer>) {
    state.ended.set(true);
    unsafe { timer.as_ref().invalidate() };
    for window in &state.windows {
        window.close();
    }
    state
        .app_sender
        .send(Msg::EndBreak)
        .expect("could not notify the main loop that the break ended");
}

fn make_shield_window(
    mtm: MainThreadMarker,
    frame: NSRect,
) -> (Retained<NSWindow>, Retained<NSTextField>) {
    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            frame,
            NSWindowStyleMask::Borderless,
            NSBackingStoreType::Buffered,
            false,
        )
    };
    // We own the window via `Retained`; don't let `-close` also release it.
    unsafe { window.setReleasedWhenClosed(false) };
    let level = CGShieldingWindowLevel();
    window.setLevel(level as isize);
    window.setBackgroundColor(Some(&NSColor::blackColor()));
    window.setOpaque(true);
    window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::FullScreenAuxiliary,
    );

    let label = NSTextField::labelWithString(&NSString::from_str("--:--"), mtm);
    label.setTextColor(Some(&NSColor::whiteColor()));
    label.setFont(Some(&NSFont::systemFontOfSize(COUNTDOWN_FONT_SIZE)));
    label.setAlignment(NSTextAlignment::Center);
    label.setFrame(NSRect::new(
        NSPoint::new(0.0, frame.size.height / 2.0 - LABEL_HEIGHT / 2.0),
        NSSize::new(frame.size.width, LABEL_HEIGHT),
    ));
    if let Some(content_view) = window.contentView() {
        content_view.addSubview(&label);
    }
    (window, label)
}
