#![allow(unsafe_code)]

pub use glib::translate::*;
use std::time::Duration;

use crate::prelude::*;
use crate::Msg;

static IMG: &[u8] = include_bytes!("../imgs/clock.png");
static IMG_STOPPED: &[u8] = include_bytes!("../imgs/clock-stopped.png");
// static IMG2: &'static [u8] = include_bytes!("../imgs/clock-2.png");

fn connect_activate<F>(
    status_icon: *mut gtk_sys::GtkStatusIcon,
    f: F,
) -> glib::signal::SignalHandlerId
where
    F: Fn(*mut gtk_sys::GtkStatusIcon) + 'static,
{
    unsafe extern "C" fn trampoline<G>(
        this: *mut gtk_sys::GtkStatusIcon,
        g: glib_sys::gpointer,
    ) where
        G: Fn(*mut gtk_sys::GtkStatusIcon) + 'static,
    {
        let g: &G = &*(g as *const G);
        g(this)
    }

    let f: Box<F> = Box::new(f);
    let raw_f: *mut F = Box::into_raw(f);
    let signal_name = b"activate\0".as_ptr() as *const std::os::raw::c_char;

    unsafe {
        let raw_trampoline: unsafe extern "C" fn() =
            std::mem::transmute(trampoline::<F> as usize);

        glib::signal::connect_raw(
            status_icon as *mut gobject_sys::GObject,
            signal_name,
            Some(raw_trampoline),
            raw_f,
        )
    }
}

fn connect_popup_menu<F>(
    status_icon: *mut gtk_sys::GtkStatusIcon,
    f: F,
) -> glib::signal::SignalHandlerId
where
    F: Fn(*mut gtk_sys::GtkStatusIcon, u32, u32) + 'static,
{
    unsafe extern "C" fn trampoline<G>(
        this: *mut gtk_sys::GtkStatusIcon,
        button: u32,
        activate_time: u32,
        g: glib_sys::gpointer,
    ) where
        G: Fn(*mut gtk_sys::GtkStatusIcon, u32, u32) + 'static,
    {
        let g: &G = &*(g as *const G);
        g(this, button, activate_time)
    }

    let f: Box<F> = Box::new(f);
    let raw_f: *mut F = Box::into_raw(f);
    let signal_name = b"popup-menu\0".as_ptr() as *const std::os::raw::c_char;

    unsafe {
        let raw_trampoline: unsafe extern "C" fn() =
            std::mem::transmute(trampoline::<F> as usize);

        glib::signal::connect_raw(
            status_icon as *mut gobject_sys::GObject,
            signal_name,
            Some(raw_trampoline),
            raw_f,
        )
    }
}

pub fn signal_handler_disconnect(
    status_icon: *mut gtk_sys::GtkStatusIcon,
    handler_id: &glib::signal::SignalHandlerId,
) {
    unsafe {
        gobject_sys::g_signal_handler_disconnect(
            status_icon as *mut gobject_sys::GObject,
            handler_id.to_glib(),
        );
    }
}

pub struct Tray {
    status_icon: *mut gtk_sys::GtkStatusIcon,
    pixbuf: gdk_pixbuf::Pixbuf,
    pixbuf_stopped: gdk_pixbuf::Pixbuf,
    sender: glib::Sender<Msg>,
    menu_right_click_signal_handler_id: Option<glib::signal::SignalHandlerId>,
}

fn load_pixbuf(image_bytes: &[u8]) -> gdk_pixbuf::Pixbuf {
    let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();
    pixbuf_loader
        .write(image_bytes)
        .expect("could not write image to pixbufloader");
    let pixbuf = pixbuf_loader
        .get_pixbuf()
        .expect("could not get a pixbuf from the loaded image");
    pixbuf_loader
        .close()
        .expect("could not close pixbuf loader");

    pixbuf
}

impl Tray {
    pub fn new(sender: glib::Sender<Msg>) -> Self {
        let pixbuf = load_pixbuf(IMG);
        let pixbuf_stopped = load_pixbuf(IMG_STOPPED);

        let pixbuf_sys: *mut gdk_pixbuf_sys::GdkPixbuf =
            pixbuf.to_glib_none().0;
        let status_icon: *mut gtk_sys::GtkStatusIcon;

        unsafe {
            status_icon = gtk_sys::gtk_status_icon_new();

            gtk_sys::gtk_status_icon_set_from_pixbuf(status_icon, pixbuf_sys);

            gtk_sys::gtk_status_icon_set_visible(status_icon, 1);
        }

        let menu_right_click_signal_handler_id = None;

        let tray = Tray {
            status_icon,
            pixbuf,
            pixbuf_stopped,
            sender,
            menu_right_click_signal_handler_id,
        };

        tray.render_normal_icon();

        tray
    }

    fn set_tooltip_text(&self, tooltip_text: &str) {
        unsafe {
            gtk_sys::gtk_status_icon_set_tooltip_text(
                self.status_icon,
                tooltip_text.to_glib_none().0,
            );
        }
    }

    pub fn render_break_starting(&self) {
        self.render_normal_icon()
    }

    fn render_pause_icon(&self) {
        self.render_pixbuf(&self.pixbuf_stopped);
    }

    pub fn render_normal_icon(&self) {
        self.render_pixbuf(&self.pixbuf);
    }

    fn render_pixbuf(&self, pixbuf: &gdk_pixbuf::Pixbuf) {
        let pixbuf_sys: *mut gdk_pixbuf_sys::GdkPixbuf =
            pixbuf.to_glib_none().0;
        unsafe {
            gtk_sys::gtk_status_icon_set_from_pixbuf(
                self.status_icon,
                pixbuf_sys,
            );
        }
    }

    pub fn render_time_remaining_before_break(&self, remaining_time: Duration) {
        // println!("Called render time remaining before break, remaining_time: {:?}...", remaining_time);
        let img: &mut &[u8] = &mut IMG.clone();

        let image_surface = cairo::ImageSurface::create_from_png(img)
            .expect("should create png from mem");

        let remaining_time_text = duration_to_text(remaining_time);
        let remaining_time_text_len = remaining_time_text.len();

        let cr = cairo::Context::new(&image_surface);
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

        cr.show_text(&remaining_time_text);

        let new_pixbuf =
            gdk::pixbuf_get_from_surface(&image_surface, 0, 0, 1000, 1000)
                .expect("Should always return surface.");
        self.render_pixbuf(&new_pixbuf);
    }

    pub fn run(sender: glib::Sender<Msg>) -> Self {
        let mut tray = Self::new(sender);
        tray.set_tooltip_text("break-time");

        connect_activate(
            tray.status_icon,
            move |_status_icon: *mut gtk_sys::GtkStatusIcon| {
                println!("clicked!!!");
            },
        );

        tray.conn_popup_menu(IsPaused::No);

        tray
    }

    pub fn resume(&mut self) {
        self.render_normal_icon();
        self.conn_popup_menu(IsPaused::No);
    }

    pub fn pause(&mut self) {
        self.render_pause_icon();
        self.conn_popup_menu(IsPaused::Yes);
    }

    pub fn break_end(&self) {
        self.render_normal_icon();
    }

    fn conn_popup_menu(&mut self, is_paused: IsPaused) {
        if let Some(prev_signal_handler_id) =
            &self.menu_right_click_signal_handler_id
        {
            signal_handler_disconnect(self.status_icon, prev_signal_handler_id);
        }

        let sender = self.sender.clone();
        let signal_handler_id = connect_popup_menu(
            self.status_icon,
            move |_status_icon: *mut gtk_sys::GtkStatusIcon,
                  button,
                  activate_time| {
                let menu = gtk::Menu::new();

                match is_paused {
                    IsPaused::No => {
                        let pause_item = gtk::MenuItem::new_with_label("Pause");
                        let sender_clone = sender.clone();
                        pause_item.connect_activate(move |_| {
                            sender_clone
                                .send(Msg::Pause)
                                .expect("Could not send Msg::Pause")
                        });
                        menu.append(&pause_item);
                    }
                    IsPaused::Yes => {
                        let resume_item =
                            gtk::MenuItem::new_with_label("Resume");
                        let sender_clone = sender.clone();
                        resume_item.connect_activate(move |_| {
                            sender_clone
                                .send(Msg::Resume)
                                .expect("Could not send Msg::Resume")
                        });
                        menu.append(&resume_item);
                    }
                }

                let quit_item = gtk::MenuItem::new_with_label("Quit");
                let sender_clone = sender.clone();
                quit_item.connect_activate(move |_| {
                    sender_clone
                        .send(Msg::Quit)
                        .expect("Could not send Msg::Quit")
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
        if remaining_time <= Duration::from_secs(5 * 60) {
            self.render_time_remaining_before_break(remaining_time);
        }

        self.set_time_remaining_tool_tip(remaining_time);
    }
}

pub enum IsPaused {
    Yes,
    No,
}

fn duration_to_text(duration: Duration) -> String {
    if duration > Duration::from_secs(60) {
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
