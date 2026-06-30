#![allow(unsafe_code)]

pub use glib::translate::*;
use std::time::Duration;

use crate::config::Config;
use crate::prelude::*;
use crate::Msg;

static IMG: &[u8] = include_bytes!("../imgs/clock.png");
static IMG_STOPPED: &[u8] = include_bytes!("../imgs/clock-stopped.png");
// static IMG2: &'static [u8] = include_bytes!("../imgs/clock-2.png");

fn connect_activate<F>(
    status_icon: *mut gtk::ffi::GtkStatusIcon,
    f: F,
) -> glib::signal::SignalHandlerId
where
    F: Fn(*mut gtk::ffi::GtkStatusIcon) + 'static,
{
    unsafe extern "C" fn trampoline<G>(
        this: *mut gtk::ffi::GtkStatusIcon,
        g: glib::ffi::gpointer,
    ) where
        G: Fn(*mut gtk::ffi::GtkStatusIcon) + 'static,
    {
        let g: &G = &*(g as *const G);
        g(this);
    }

    let f: Box<F> = Box::new(f);
    let raw_f: *mut F = Box::into_raw(f);
    let signal_name = c"activate".as_ptr();

    unsafe {
        let trampoline_fn: unsafe extern "C" fn(
            *mut gtk::ffi::GtkStatusIcon,
            glib::ffi::gpointer,
        ) = trampoline::<F>;
        let raw_trampoline: unsafe extern "C" fn() =
            std::mem::transmute(trampoline_fn);

        glib::signal::connect_raw(
            status_icon.cast::<glib::gobject_ffi::GObject>(),
            signal_name,
            Some(raw_trampoline),
            raw_f,
        )
    }
}

fn connect_popup_menu<F>(
    status_icon: *mut gtk::ffi::GtkStatusIcon,
    f: F,
) -> glib::signal::SignalHandlerId
where
    F: Fn(*mut gtk::ffi::GtkStatusIcon, u32, u32) + 'static,
{
    unsafe extern "C" fn trampoline<G>(
        this: *mut gtk::ffi::GtkStatusIcon,
        button: u32,
        activate_time: u32,
        g: glib::ffi::gpointer,
    ) where
        G: Fn(*mut gtk::ffi::GtkStatusIcon, u32, u32) + 'static,
    {
        let g: &G = &*(g as *const G);
        g(this, button, activate_time);
    }

    let f: Box<F> = Box::new(f);
    let raw_f: *mut F = Box::into_raw(f);
    let signal_name = c"popup-menu".as_ptr();

    unsafe {
        let trampoline_fn: unsafe extern "C" fn(
            *mut gtk::ffi::GtkStatusIcon,
            u32,
            u32,
            glib::ffi::gpointer,
        ) = trampoline::<F>;
        let raw_trampoline: unsafe extern "C" fn() =
            std::mem::transmute(trampoline_fn);

        glib::signal::connect_raw(
            status_icon.cast::<glib::gobject_ffi::GObject>(),
            signal_name,
            Some(raw_trampoline),
            raw_f,
        )
    }
}

pub fn signal_handler_disconnect(
    status_icon: *mut gtk::ffi::GtkStatusIcon,
    handler_id: &glib::signal::SignalHandlerId,
) {
    unsafe {
        glib::gobject_ffi::g_signal_handler_disconnect(
            status_icon.cast::<glib::gobject_ffi::GObject>(),
            handler_id.as_raw(),
        );
    }
}

pub struct Tray {
    status_icon: *mut gtk::ffi::GtkStatusIcon,
    pixbuf: gdk_pixbuf::Pixbuf,
    pixbuf_stopped: gdk_pixbuf::Pixbuf,
    sender: glib::Sender<Msg>,
    menu_right_click_signal_handler_id: Option<glib::signal::SignalHandlerId>,
    is_idle_detector_enabled: IsIdleDetectorEnabled,
    is_paused: IsPaused,
}

fn load_pixbuf(image_bytes: &[u8]) -> gdk_pixbuf::Pixbuf {
    let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();
    pixbuf_loader
        .write(image_bytes)
        .expect("could not write image to pixbufloader");
    let pixbuf = pixbuf_loader
        .pixbuf()
        .expect("could not get a pixbuf from the loaded image");
    pixbuf_loader
        .close()
        .expect("could not close pixbuf loader");

    pixbuf
}

impl Tray {
    pub fn new(config: &Config, sender: glib::Sender<Msg>) -> Self {
        let pixbuf = load_pixbuf(IMG);
        let pixbuf_stopped = load_pixbuf(IMG_STOPPED);

        let pixbuf_sys: *mut gdk_pixbuf::ffi::GdkPixbuf =
            pixbuf.to_glib_none().0;
        let status_icon: *mut gtk::ffi::GtkStatusIcon;

        unsafe {
            status_icon = gtk::ffi::gtk_status_icon_new();

            gtk::ffi::gtk_status_icon_set_from_pixbuf(status_icon, pixbuf_sys);

            gtk::ffi::gtk_status_icon_set_visible(status_icon, 1);
        }

        let menu_right_click_signal_handler_id = None;

        let is_idle_detector_enabled = if config.settings.idle_detection_enabled
        {
            IsIdleDetectorEnabled::Yes
        } else {
            IsIdleDetectorEnabled::No
        };

        let tray = Self {
            status_icon,
            pixbuf,
            pixbuf_stopped,
            sender,
            menu_right_click_signal_handler_id,
            is_idle_detector_enabled,
            is_paused: IsPaused::No,
        };

        tray.render_normal_icon();

        tray
    }

    fn set_tooltip_text(&self, tooltip_text: &str) {
        unsafe {
            gtk::ffi::gtk_status_icon_set_tooltip_text(
                self.status_icon,
                tooltip_text.to_glib_none().0,
            );
        }
    }

    pub fn render_break_starting(&self) {
        self.render_normal_icon();
    }

    fn render_pause_icon(&self) {
        self.render_pixbuf(&self.pixbuf_stopped);
    }

    pub fn render_normal_icon(&self) {
        self.render_pixbuf(&self.pixbuf);
    }

    fn render_pixbuf(&self, pixbuf: &gdk_pixbuf::Pixbuf) {
        let pixbuf_sys: *mut gdk_pixbuf::ffi::GdkPixbuf =
            pixbuf.to_glib_none().0;
        unsafe {
            gtk::ffi::gtk_status_icon_set_from_pixbuf(
                self.status_icon,
                pixbuf_sys,
            );
        }
    }

    pub fn render_time_remaining_before_break(&self, remaining_time: Duration) {
        // println!("Called render time remaining before break, remaining_time: {:?}...", remaining_time);
        let mut img: &[u8] = <&[u8]>::clone(&IMG);

        let image_surface = cairo::ImageSurface::create_from_png(&mut img)
            .expect("should create png from mem");

        let remaining_time_text = duration_to_text(remaining_time);
        let remaining_time_text_len = remaining_time_text.len();

        let cr = cairo::Context::new(&image_surface)
            .expect("should create cairo context");
        cr.select_font_face(
            "monospace",
            cairo::FontSlant::Normal,
            cairo::FontWeight::Bold,
        );
        cr.set_font_size(800.0);
        cr.set_source_rgb(1.0, 0.0, 0.0);

        if remaining_time_text_len <= 1 {
            cr.move_to(250.0, 750.0);
        } else {
            cr.move_to(0.0, 750.0);
        }

        cr.show_text(&remaining_time_text)
            .expect("should draw text onto the icon");

        let new_pixbuf =
            gdk::pixbuf_get_from_surface(&image_surface, 0, 0, 1000, 1000)
                .expect("Should always return surface.");
        self.render_pixbuf(&new_pixbuf);
    }

    pub fn run(config: &Config, sender: glib::Sender<Msg>) -> Self {
        let mut tray = Self::new(config, sender);
        tray.set_tooltip_text("break-time");

        connect_activate(
            tray.status_icon,
            move |_status_icon: *mut gtk::ffi::GtkStatusIcon| {
                println!("clicked!!!");
            },
        );

        tray.conn_popup_menu();

        tray
    }

    pub fn resume(&mut self) {
        self.render_normal_icon();
        self.is_paused = IsPaused::No;
        self.conn_popup_menu();
    }

    pub fn pause(&mut self) {
        self.render_pause_icon();
        self.is_paused = IsPaused::Yes;
        self.conn_popup_menu();
    }

    pub fn break_end(&self) {
        self.render_normal_icon();
    }

    pub fn set_is_idle_detector_enabled(
        &mut self,
        is_idle_detector_enabled: IsIdleDetectorEnabled,
    ) {
        self.is_idle_detector_enabled = is_idle_detector_enabled;
        self.conn_popup_menu();
    }

    fn conn_popup_menu(&mut self) {
        if let Some(prev_signal_handler_id) =
            &self.menu_right_click_signal_handler_id
        {
            signal_handler_disconnect(self.status_icon, prev_signal_handler_id);
        }

        let is_idle_detector_enabled = self.is_idle_detector_enabled;
        let is_paused = self.is_paused;

        let sender = self.sender.clone();
        let signal_handler_id = connect_popup_menu(
            self.status_icon,
            move |_status_icon: *mut gtk::ffi::GtkStatusIcon,
                  button,
                  activate_time| {
                let menu = gtk::Menu::new();

                match is_paused {
                    IsPaused::No => {
                        let pause_item = gtk::MenuItem::with_label("Pause");
                        let sender_clone = sender.clone();
                        pause_item.connect_activate(move |_| {
                            sender_clone
                                .send(Msg::Pause)
                                .expect("Could not send Msg::Pause");
                        });
                        menu.append(&pause_item);
                    }
                    IsPaused::Yes => {
                        let resume_item = gtk::MenuItem::with_label("Resume");
                        let sender_clone = sender.clone();
                        resume_item.connect_activate(move |_| {
                            sender_clone
                                .send(Msg::Resume)
                                .expect("Could not send Msg::Resume");
                        });
                        menu.append(&resume_item);
                    }
                }

                match is_idle_detector_enabled {
                    IsIdleDetectorEnabled::No => {
                        let enable_idle_detector_item =
                            gtk::MenuItem::with_label("Enable Idle Detector");
                        let sender_clone = sender.clone();
                        enable_idle_detector_item.connect_activate(move |_| {
                            sender_clone.send(Msg::EnableIdleDetector).expect(
                                "Could not send Msg::EnableIdleDetector",
                            );
                        });
                        menu.append(&enable_idle_detector_item);
                    }
                    IsIdleDetectorEnabled::Yes => {
                        let disable_idle_detector_item =
                            gtk::MenuItem::with_label("Disable Idle Detector");
                        let sender_clone = sender.clone();
                        disable_idle_detector_item.connect_activate(
                            move |_| {
                                sender_clone
                                    .send(Msg::DisableIdleDetector)
                                    .expect(
                                    "Could not send Msg::DisableIdleDetector",
                                );
                            },
                        );
                        menu.append(&disable_idle_detector_item);
                    }
                }

                let quit_item = gtk::MenuItem::with_label("Quit");
                let sender_clone = sender.clone();
                quit_item.connect_activate(move |_| {
                    sender_clone
                        .send(Msg::Quit)
                        .expect("Could not send Msg::Quit");
                });
                menu.append(&quit_item);

                menu.show_all();
                menu.popup_easy(button, activate_time);
            },
        );
        self.menu_right_click_signal_handler_id = Some(signal_handler_id);
    }

    fn set_time_remaining_tool_tip(&self, remaining_time: Duration) {
        self.set_tooltip_text(&format!(
            "break-time: {} until next break",
            remaining_duration_to_text(remaining_time)
        ));
    }

    pub fn update_time_remaining(&self, remaining_time: Duration) {
        if remaining_time <= Duration::from_mins(5) {
            self.render_time_remaining_before_break(remaining_time);
        }

        self.set_time_remaining_tool_tip(remaining_time);
    }
}

#[derive(Copy, Clone, Debug)]
pub enum IsPaused {
    Yes,
    No,
}

#[derive(Copy, Clone, Debug)]
pub enum IsIdleDetectorEnabled {
    Yes,
    No,
}

fn duration_to_text(duration: Duration) -> String {
    if duration > Duration::from_mins(1) {
        format!("{}m", duration.as_secs() / 60)
    } else {
        duration.as_secs().to_string()
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
