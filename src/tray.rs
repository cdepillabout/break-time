#![allow(unsafe_code)]

pub use glib::translate::*;

use crate::prelude::*;
use crate::Msg;

static IMG: &'static [u8] = include_bytes!("../imgs/clock.png");
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


pub struct Tray {
    status_icon: *mut gtk_sys::GtkStatusIcon,
    pixbuf: gdk_pixbuf::Pixbuf,
    sender: glib::Sender<Msg>,
}

impl Tray {
    pub fn new(sender: glib::Sender<Msg>) -> Self {

        let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();
        pixbuf_loader
            .write(IMG)
            .expect("could not write image to pixbufloader");
        let pixbuf = pixbuf_loader
            .get_pixbuf()
            .expect("could not get a pixbuf from the loaded image");
        pixbuf_loader
            .close()
            .expect("could not close pixbuf loader");

        let pixbuf_sys: *mut gdk_pixbuf_sys::GdkPixbuf = pixbuf.to_glib_none().0;
        let status_icon: *mut gtk_sys::GtkStatusIcon;

        unsafe {
            status_icon = gtk_sys::gtk_status_icon_new();

            gtk_sys::gtk_status_icon_set_from_pixbuf(status_icon, pixbuf_sys);

            gtk_sys::gtk_status_icon_set_visible(status_icon, 1);
        }

        Tray {
            status_icon,
            pixbuf,
            sender,
        }
    }

    pub fn set_tooltip_text(&self, tooltip_text: &str) {
        unsafe {
            gtk_sys::gtk_status_icon_set_tooltip_text(
                self.status_icon,
                tooltip_text.to_glib_none().0,
            );
        }
    }

    pub fn run(sender: glib::Sender<Msg>) -> Self {
        let tray = Self::new(sender);
        tray.set_tooltip_text("break-time");

        connect_activate(
            tray.status_icon,
            move |_status_icon: *mut gtk_sys::GtkStatusIcon| {
                // unsafe {
                //     gtk_sys::gtk_status_icon_set_from_pixbuf(status_icon, whatwhat);
                // }
                println!("clicked!!!");
            },
        );

        let sender = tray.sender.clone();

        connect_popup_menu(
            tray.status_icon,
            move |_status_icon: *mut gtk_sys::GtkStatusIcon,
                button,
                activate_time| {
                let menu = gtk::Menu::new();
                let quit_item = gtk::MenuItem::new_with_label("Quit");
                let sender_clone = sender.clone();
                quit_item.connect_activate(move |_| sender_clone.send(Msg::Quit).expect("Could not send Msg::Quit"));
                menu.append(&quit_item);
                menu.show_all();
                menu.popup_easy(button, activate_time);
            },
        );

        tray
    }
}
