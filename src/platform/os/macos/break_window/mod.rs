//! macOS break window: one borderless full-screen shield `NSWindow` per
//! `NSScreen`, with a centered countdown, driven by an `NSTimer`. This is the
//! Cocoa counterpart to the GTK break window under
//! `platform::os::linux::break_window`.
//!
//! The break ends when the countdown elapses or the user presses space
//! `clicks_to_end_break_early` times. The lock is *permission-free*: the shield
//! windows sit at `CGShieldingWindowLevel`, and while the app is active it
//! enters kiosk mode via `NSApplicationPresentationOptions` (blocking Cmd-Tab,
//! the Dock, the menu bar, the Force-Quit panel, and logout/shutdown). Spacebar
//! is read via a local `NSEvent` keyDown monitor on a `canBecomeKeyWindow`
//! shield window.
//!
//! Known limitation (deferred — see the Phase 4/5 backlog in TODO.md): macOS
//! blocks a background app from programmatically stealing keyboard focus, so on
//! break start the shields appear but are not the key window until the user
//! clicks one. The kiosk options and spacebar-to-end only take effect once the
//! app is active — i.e. after that click. This was confirmed across `cargo
//! run`, a bare bundle, and an `open`-launched `.app` (neither
//! `setActivationPolicy(.regular)` nor `activateIgnoringOtherApps` helped); a
//! mouse click before spacebar is the accepted workaround for now. The break
//! still always ends on the timer regardless.
//!
//! macOS also has no user-space way to *swallow* all input without the
//! Accessibility permission (that would need a `CGEventTap`); Mission Control in
//! particular is not blocked by kiosk mode. An optional `CGEventTap` — active
//! only when Accessibility is granted — is a possible future enhancement, not
//! required here.

#![allow(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::ptr::NonNull;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{define_class, msg_send, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationOptions,
    NSApplicationActivationPolicy, NSApplicationPresentationOptions,
    NSBackingStoreType, NSColor, NSEvent, NSEventMask, NSFont,
    NSRunningApplication, NSScreen, NSTextAlignment, NSTextField, NSWindow,
    NSWindowCollectionBehavior, NSWindowStyleMask, NSWorkspace,
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
const COUNTDOWN_LABEL_HEIGHT: f64 = 180.0;

/// Point size of the "press space to end" hint.
const HINT_FONT_SIZE: f64 = 34.0;

/// Height of the hint label's frame.
const HINT_LABEL_HEIGHT: f64 = 60.0;

/// The `keyCode` for the space bar.
const SPACE_KEY_CODE: u16 = 49;

define_class!(
    // A borderless `NSWindow` returns NO for `canBecomeKeyWindow` by default,
    // which would stop it from receiving key events. Override it so the shield
    // can become key and the local `NSEvent` monitor sees spacebar presses —
    // without needing any permission.
    #[unsafe(super(NSWindow))]
    #[thread_kind = MainThreadOnly]
    #[name = "BTBreakWindow"]
    struct BreakWindow;

    impl BreakWindow {
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key_window(&self) -> bool {
            true
        }
    }
);

struct BreakState {
    windows: Vec<Retained<BreakWindow>>,
    time_labels: Vec<Retained<NSTextField>>,
    hint_labels: Vec<Retained<NSTextField>>,
    /// Break start in wall-clock time (like the Linux break window), NOT
    /// `Instant`: on macOS `Instant` freezes during system sleep, which would
    /// make a break suspended mid-way resume where it left off. Wall time lets
    /// the suspend run the break down — being away IS the break.
    start: SystemTime,
    duration: Duration,
    app_sender: AppSender,
    ended: Cell<bool>,
    /// Spacebar presses left before the break ends early.
    presses_remaining: Cell<u32>,
    /// The app that was frontmost before the break, reactivated on end so focus
    /// returns to it. macOS restores the *app*, not a specific window (which
    /// would need the Accessibility API).
    previous_app: Option<Retained<NSRunningApplication>>,
    /// The local keyDown monitor, removed on break end. Stored behind a
    /// `RefCell` so it can be installed after `BreakState` exists (its handler
    /// captures the shared state) and `take`-n out on end to break the
    /// state↔monitor reference cycle.
    key_monitor: RefCell<Option<Retained<AnyObject>>>,
}

pub fn start_break(config: &Config, app_sender: AppSender) {
    let mtm = MainThreadMarker::new()
        .expect("start_break must run on the main thread");
    let app = NSApplication::sharedApplication(mtm);

    // Remember what was frontmost before we cover the screen, so focus can
    // return to it on break end. Capture this before activating our own app.
    let previous_app = NSWorkspace::sharedWorkspace().frontmostApplication();

    // Become a regular, activatable app for the break. An accessory (menu-bar)
    // app cannot reliably become active or take keyboard focus, which would
    // leave spacebar going to whatever is underneath the shield. The Dock icon
    // this implies is hidden by the kiosk HideDock below, and we revert to
    // accessory in `end_break`. Activate before ordering the shields so they
    // can become key.
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
    #[allow(deprecated)]
    app.activateIgnoringOtherApps(true);

    let mut windows = Vec::new();
    let mut time_labels = Vec::new();
    let mut hint_labels = Vec::new();
    // `NSArray` is iterated via its `iter()`; there is no by-reference form.
    #[allow(clippy::explicit_iter_loop)]
    for screen in NSScreen::screens(mtm).iter() {
        let frame = screen.frame();
        let (window, time_label, hint_label) = make_shield_window(mtm, frame);
        window.makeKeyAndOrderFront(None);
        windows.push(window);
        time_labels.push(time_label);
        hint_labels.push(hint_label);
    }

    // Enter kiosk mode (blocks Cmd-Tab, Dock, menu bar, Force-Quit,
    // logout/shutdown). presentationOptions only apply while we are active, so
    // set them after activating.
    app.setPresentationOptions(kiosk_presentation_options());

    let state = Rc::new(BreakState {
        windows,
        time_labels,
        hint_labels,
        start: SystemTime::now(),
        duration: Duration::from_secs(
            config.settings.break_duration_seconds.into(),
        ),
        app_sender,
        ended: Cell::new(false),
        presses_remaining: Cell::new(config.settings.clicks_to_end_break_early),
        previous_app,
        key_monitor: RefCell::new(None),
    });

    // Paint the initial time + hint immediately (the first tick is
    // TICK_INTERVAL away).
    refresh_time(&state);
    refresh_hint(&state);

    // Count spacebar presses via a local keyDown monitor. It consumes every
    // keyDown (so stray keys neither beep nor reach anything) and, on space,
    // decrements the counter; the timer tick notices when it reaches zero.
    let state_key = Rc::clone(&state);
    let key_handler =
        RcBlock::new(move |event: NonNull<NSEvent>| -> *mut NSEvent {
            on_key_down(&state_key, event)
        });
    let monitor = unsafe {
        NSEvent::addLocalMonitorForEventsMatchingMask_handler(
            NSEventMask::KeyDown,
            &key_handler,
        )
    };
    *state.key_monitor.borrow_mut() = monitor;

    // Drive the countdown / end conditions. The run loop retains the scheduled
    // timer; the timer owns the block, which owns `state`. Invalidating the
    // timer at break end tears all of it down.
    let state_tick = Rc::clone(&state);
    let tick_handler = RcBlock::new(move |timer: NonNull<NSTimer>| {
        tick(&state_tick, timer);
    });
    let _timer = unsafe {
        NSTimer::scheduledTimerWithTimeInterval_repeats_block(
            TICK_INTERVAL,
            true,
            &tick_handler,
        )
    };
}

fn on_key_down(state: &BreakState, event: NonNull<NSEvent>) -> *mut NSEvent {
    if !state.ended.get()
        && unsafe { event.as_ref().keyCode() } == SPACE_KEY_CODE
    {
        let remaining = state.presses_remaining.get().saturating_sub(1);
        state.presses_remaining.set(remaining);
        refresh_hint(state);
    }
    // Consume the event (returning null); nothing behind the shield should see
    // keystrokes.
    std::ptr::null_mut()
}

/// Wall-clock time since the break started. A backwards clock jump makes
/// `SystemTime::elapsed` fail; report the full duration in that case so the
/// break simply ends (the Linux break window resolves the same anomaly the
/// same way).
fn elapsed(state: &BreakState) -> Duration {
    state.start.elapsed().unwrap_or(state.duration)
}

fn tick(state: &BreakState, timer: NonNull<NSTimer>) {
    if state.ended.get() {
        return;
    }
    if elapsed(state) >= state.duration || state.presses_remaining.get() == 0 {
        end_break(state, timer);
    } else {
        refresh_time(state);
    }
}

fn refresh_time(state: &BreakState) {
    let remaining = state.duration.saturating_sub(elapsed(state));
    let total_secs = remaining.as_secs();
    let text = format!("{:02}:{:02}", total_secs / 60, total_secs % 60);
    let ns_text = NSString::from_str(&text);
    for label in &state.time_labels {
        label.setStringValue(&ns_text);
    }
}

fn refresh_hint(state: &BreakState) {
    let remaining = state.presses_remaining.get();
    let text = if remaining == 0 {
        String::new()
    } else {
        format!("or press the space bar {remaining}× to end early")
    };
    let ns_text = NSString::from_str(&text);
    for label in &state.hint_labels {
        label.setStringValue(&ns_text);
    }
}

fn end_break(state: &BreakState, timer: NonNull<NSTimer>) {
    state.ended.set(true);
    unsafe { timer.as_ref().invalidate() };

    // Remove the key monitor (taking it out of `state` also breaks the
    // state↔monitor cycle) and leave kiosk mode.
    if let Some(monitor) = state.key_monitor.borrow_mut().take() {
        unsafe { NSEvent::removeMonitor(&monitor) };
    }
    if let Some(mtm) = MainThreadMarker::new() {
        let app = NSApplication::sharedApplication(mtm);
        app.setPresentationOptions(NSApplicationPresentationOptions::empty());
        // Revert to a menu-bar (accessory) app now the break is over.
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    }

    for window in &state.windows {
        window.close();
    }

    // Return focus to whatever was frontmost before the break.
    if let Some(previous_app) = &state.previous_app {
        // Deprecated in macOS 14 (superseded by `activate`) but widely
        // compatible and sufficient here.
        #[allow(deprecated)]
        previous_app
            .activateWithOptions(NSApplicationActivationOptions::empty());
    }

    state
        .app_sender
        .send(Msg::EndBreak)
        .expect("could not notify the main loop that the break ended");
}

/// Kiosk-mode flags. Every `Disable*` flag must be accompanied by `HideDock`, so
/// that is included. These block the common escape routes without any
/// permission; Mission Control is the notable exception (see the module docs).
fn kiosk_presentation_options() -> NSApplicationPresentationOptions {
    NSApplicationPresentationOptions::HideDock
        | NSApplicationPresentationOptions::HideMenuBar
        | NSApplicationPresentationOptions::DisableProcessSwitching
        | NSApplicationPresentationOptions::DisableForceQuit
        | NSApplicationPresentationOptions::DisableSessionTermination
        | NSApplicationPresentationOptions::DisableHideApplication
        | NSApplicationPresentationOptions::DisableAppleMenu
}

fn make_shield_window(
    mtm: MainThreadMarker,
    frame: NSRect,
) -> (
    Retained<BreakWindow>,
    Retained<NSTextField>,
    Retained<NSTextField>,
) {
    let window: Retained<BreakWindow> = unsafe {
        msg_send![
            BreakWindow::alloc(mtm),
            initWithContentRect: frame,
            styleMask: NSWindowStyleMask::Borderless,
            backing: NSBackingStoreType::Buffered,
            defer: false,
        ]
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

    let center_y = frame.size.height / 2.0;
    let time_label = make_label(mtm, COUNTDOWN_FONT_SIZE);
    time_label.setFrame(NSRect::new(
        NSPoint::new(0.0, center_y - COUNTDOWN_LABEL_HEIGHT / 2.0),
        NSSize::new(frame.size.width, COUNTDOWN_LABEL_HEIGHT),
    ));

    let hint_label = make_label(mtm, HINT_FONT_SIZE);
    hint_label.setFrame(NSRect::new(
        NSPoint::new(
            0.0,
            center_y - COUNTDOWN_LABEL_HEIGHT / 2.0 - HINT_LABEL_HEIGHT,
        ),
        NSSize::new(frame.size.width, HINT_LABEL_HEIGHT),
    ));

    if let Some(content_view) = window.contentView() {
        content_view.addSubview(&time_label);
        content_view.addSubview(&hint_label);
    }
    (window, time_label, hint_label)
}

fn make_label(mtm: MainThreadMarker, font_size: f64) -> Retained<NSTextField> {
    let label = NSTextField::labelWithString(&NSString::from_str(""), mtm);
    label.setTextColor(Some(&NSColor::whiteColor()));
    label.setFont(Some(&NSFont::systemFontOfSize(font_size)));
    label.setAlignment(NSTextAlignment::Center);
    label
}
