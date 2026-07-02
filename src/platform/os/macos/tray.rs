//! macOS system tray: an `NSStatusItem` in the menu bar. The native counterpart
//! to the Linux `GtkStatusIcon` tray.
//!
//! The status-bar button shows a clock glyph, switching to a pause glyph while
//! paused and to an explicit countdown ("⏱ 4m") once the next break is less
//! than five minutes out — the text analog of the Linux tray drawing the
//! remaining time onto its icon. The tooltip always carries the exact time
//! until the next break. The menu (Pause/Resume, Enable/Disable Idle Detector,
//! Quit) sends the same `Msg`s as the Linux tray; menu items deliver their
//! action selector to a small Objective-C target object (`BTTrayTarget`)
//! holding the `AppSender`. Like the Linux tray reconnecting its popup handler,
//! the menu is rebuilt whenever the paused/idle-detector state flips.

#![allow(unsafe_code)]

use std::time::Duration;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{
    define_class, msg_send, sel, DefinedClass, MainThreadMarker, MainThreadOnly,
};
use objc2_app_kit::{
    NSMenu, NSMenuItem, NSStatusBar, NSStatusItem, NSVariableStatusItemLength,
};
use objc2_foundation::{NSObject, NSString};

use crate::config::Config;
use crate::platform::AppSender;
use crate::Msg;

/// Menu-bar text while counting down normally.
const TITLE_NORMAL: &str = "⏱";

/// Menu-bar text while paused.
const TITLE_PAUSED: &str = "⏸";

/// Show the remaining time in the menu bar once the next break is this close
/// (matches the Linux tray's countdown-on-icon threshold).
const COUNTDOWN_SHOW_THRESHOLD: Duration = Duration::from_mins(5);

#[derive(Copy, Clone, Debug)]
pub enum IsIdleDetectorEnabled {
    Yes,
    No,
}

#[derive(Copy, Clone, Debug)]
enum IsPaused {
    Yes,
    No,
}

struct TargetIvars {
    sender: AppSender,
}

define_class!(
    // The target object menu items deliver their action selectors to; each
    // action forwards the corresponding `Msg` to the main loop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "BTTrayTarget"]
    #[ivars = TargetIvars]
    struct TrayTarget;

    impl TrayTarget {
        #[unsafe(method(onPause:))]
        fn on_pause(&self, _sender: Option<&AnyObject>) {
            self.send(Msg::Pause);
        }

        #[unsafe(method(onResume:))]
        fn on_resume(&self, _sender: Option<&AnyObject>) {
            self.send(Msg::Resume);
        }

        #[unsafe(method(onEnableIdleDetector:))]
        fn on_enable_idle_detector(&self, _sender: Option<&AnyObject>) {
            self.send(Msg::EnableIdleDetector);
        }

        #[unsafe(method(onDisableIdleDetector:))]
        fn on_disable_idle_detector(&self, _sender: Option<&AnyObject>) {
            self.send(Msg::DisableIdleDetector);
        }

        #[unsafe(method(onQuit:))]
        fn on_quit(&self, _sender: Option<&AnyObject>) {
            self.send(Msg::Quit);
        }
    }
);

impl TrayTarget {
    fn new(mtm: MainThreadMarker, sender: AppSender) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(TargetIvars { sender });
        unsafe { msg_send![super(this), init] }
    }

    fn send(&self, msg: Msg) {
        self.ivars()
            .sender
            .send(msg)
            .expect("could not send a tray menu message to the main loop");
    }
}

pub struct Tray {
    mtm: MainThreadMarker,
    status_item: Retained<NSStatusItem>,
    target: Retained<TrayTarget>,
    is_idle_detector_enabled: IsIdleDetectorEnabled,
    is_paused: IsPaused,
}

impl Tray {
    #[must_use]
    pub fn run(config: &Config, sender: AppSender) -> Self {
        let mtm = MainThreadMarker::new()
            .expect("the tray must be created on the main thread");

        let status_item = NSStatusBar::systemStatusBar()
            .statusItemWithLength(NSVariableStatusItemLength);
        let target = TrayTarget::new(mtm, sender);

        let is_idle_detector_enabled = if config.settings.idle_detection_enabled
        {
            IsIdleDetectorEnabled::Yes
        } else {
            IsIdleDetectorEnabled::No
        };

        let tray = Self {
            mtm,
            status_item,
            target,
            is_idle_detector_enabled,
            is_paused: IsPaused::No,
        };

        tray.render_normal_icon();
        tray.set_tooltip_text("break-time");
        tray.rebuild_menu();

        tray
    }

    pub fn render_break_starting(&self) {
        self.render_normal_icon();
    }

    pub fn render_normal_icon(&self) {
        self.set_title(TITLE_NORMAL);
    }

    fn render_pause_icon(&self) {
        self.set_title(TITLE_PAUSED);
    }

    pub fn break_end(&self) {
        self.render_normal_icon();
    }

    pub fn pause(&mut self) {
        self.render_pause_icon();
        self.is_paused = IsPaused::Yes;
        self.rebuild_menu();
    }

    pub fn resume(&mut self) {
        self.render_normal_icon();
        self.is_paused = IsPaused::No;
        self.rebuild_menu();
    }

    pub fn set_is_idle_detector_enabled(
        &mut self,
        is_idle_detector_enabled: IsIdleDetectorEnabled,
    ) {
        self.is_idle_detector_enabled = is_idle_detector_enabled;
        self.rebuild_menu();
    }

    pub fn update_time_remaining(&self, remaining_time: Duration) {
        if remaining_time <= COUNTDOWN_SHOW_THRESHOLD {
            self.set_title(&format!(
                "{TITLE_NORMAL} {}",
                duration_to_text(remaining_time)
            ));
        }

        self.set_tooltip_text(&format!(
            "break-time: {} until next break",
            remaining_duration_to_text(remaining_time)
        ));
    }

    fn set_title(&self, title: &str) {
        if let Some(button) = self.status_item.button(self.mtm) {
            button.setTitle(&NSString::from_str(title));
        }
    }

    fn set_tooltip_text(&self, tooltip_text: &str) {
        if let Some(button) = self.status_item.button(self.mtm) {
            button.setToolTip(Some(&NSString::from_str(tooltip_text)));
        }
    }

    /// (Re)build the menu to match the current paused/idle-detector state —
    /// each state shows only the action that would change it, mirroring the
    /// Linux tray.
    fn rebuild_menu(&self) {
        let menu = NSMenu::new(self.mtm);

        match self.is_paused {
            IsPaused::No => {
                menu.addItem(&self.make_menu_item("Pause", sel!(onPause:)));
            }
            IsPaused::Yes => {
                menu.addItem(&self.make_menu_item("Resume", sel!(onResume:)));
            }
        }

        match self.is_idle_detector_enabled {
            IsIdleDetectorEnabled::No => {
                menu.addItem(&self.make_menu_item(
                    "Enable Idle Detector",
                    sel!(onEnableIdleDetector:),
                ));
            }
            IsIdleDetectorEnabled::Yes => {
                menu.addItem(&self.make_menu_item(
                    "Disable Idle Detector",
                    sel!(onDisableIdleDetector:),
                ));
            }
        }

        menu.addItem(&self.make_menu_item("Quit", sel!(onQuit:)));

        self.status_item.setMenu(Some(&menu));
    }

    fn make_menu_item(
        &self,
        title: &str,
        action: objc2::runtime::Sel,
    ) -> Retained<NSMenuItem> {
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(self.mtm),
                &NSString::from_str(title),
                Some(action),
                &NSString::from_str(""),
            )
        };
        unsafe { item.setTarget(Some(&self.target)) };
        item
    }
}

fn duration_to_text(duration: Duration) -> String {
    if duration > Duration::from_mins(1) {
        format!("{}m", duration.as_secs() / 60)
    } else {
        format!("{}s", duration.as_secs())
    }
}

fn remaining_duration_to_text(duration: Duration) -> String {
    let duration_secs = duration.as_secs();
    if duration_secs > 60 {
        format!(
            "{} minute{}",
            duration_secs / 60,
            if duration_secs == 60 { "" } else { "s" }
        )
    } else {
        format!(
            "{} second{}",
            duration_secs,
            if duration_secs == 1 { "" } else { "s" }
        )
    }
}
