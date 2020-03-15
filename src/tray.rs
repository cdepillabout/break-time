#![allow(unsafe_code)]

pub use glib::translate::*;

use crate::prelude::*;


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
    )
    where
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

pub fn show() -> (*mut gtk_sys::GtkStatusIcon, gdk_pixbuf::Pixbuf) {
    let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();
    pixbuf_loader
        .write(IMG)
        .expect("could not write image to pixbufloader");
    let pixbuf = pixbuf_loader
        .get_pixbuf()
        .expect("could not get a pixbuf from the loaded image");
    pixbuf_loader.close().expect("could not close pixbuf loader");

    let pixbuf_sys: *mut gdk_pixbuf_sys::GdkPixbuf = pixbuf.to_glib_none().0;

    let status_icon: *mut gtk_sys::GtkStatusIcon;

    unsafe {
        status_icon = gtk_sys::gtk_status_icon_new();

        gtk_sys::gtk_status_icon_set_from_pixbuf(status_icon, pixbuf_sys);

        gtk_sys::gtk_status_icon_set_tooltip_text(status_icon, "hello".to_glib_none().0);

        gtk_sys::gtk_status_icon_set_visible(status_icon, 1);
    }

    connect_activate(
        status_icon,
        move |_status_icon: *mut gtk_sys::GtkStatusIcon| {
            // unsafe {
            //     gtk_sys::gtk_status_icon_set_from_pixbuf(status_icon, whatwhat);
            // }
            println!("clicked!!!");
        },
    );

    connect_popup_menu(
        status_icon,
        move |_status_icon: *mut gtk_sys::GtkStatusIcon, button, activate_time| {
            let menu = gtk::Menu::new();
            let test_item = gtk::MenuItem::new_with_label("test test test");
            test_item.connect_activate(|_| println!("yo from menu item"));
            menu.append(&test_item);
            println!("before popup_menu!!!");
            menu.show_all();
            menu.popup_easy(button, activate_time);
            println!("after popup_menu!!!");
        }
    );

    (status_icon, pixbuf)
}
