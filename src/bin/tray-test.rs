
use gdk_pixbuf::prelude::*;
use glib::translate::*;

static IMG: &'static [u8] = include_bytes!("../../imgs/clock.png");
static IMG2: &'static [u8] = include_bytes!("../../imgs/clock-2.png");

fn connect_activate<F>(status_icon: *mut gtk_sys::GtkStatusIcon, f: F) -> glib::signal::SignalHandlerId where
    F: Fn(*mut gtk_sys::GtkStatusIcon) + 'static,
{
    unsafe extern "C" fn activate_trampoline<G>(this: *mut gtk_sys::GtkStatusIcon, g: glib_sys::gpointer) where
        G: Fn(*mut gtk_sys::GtkStatusIcon) + 'static,
    {
        let g: &G = &*(g as *const G);
        g(this)
    }

    let f: Box<F> = Box::new(f);
    let raw_f: *mut F = Box::into_raw(f);
    let signal_name = b"activate\0".as_ptr() as *const std::os::raw::c_char;

    unsafe {
        let raw_trampoline: unsafe extern "C" fn() = std::mem::transmute(activate_trampoline::<F> as usize);

        glib::signal::connect_raw(
            status_icon as *mut gobject_sys::GObject,
            signal_name,
            Some(raw_trampoline),
            raw_f,
        )
    }
}

fn main() {
    gtk::init().unwrap();

    let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();
    pixbuf_loader.write(IMG).expect("could not write image to pixbufloader");
    let pixbuf = pixbuf_loader.get_pixbuf().expect("could not get a pixbuf from the loaded image");
    let whowho: *mut gdk_pixbuf_sys::GdkPixbuf = pixbuf.to_glib_none().0;

    let pixbuf_loader2 = gdk_pixbuf::PixbufLoader::new();
    pixbuf_loader2.write(IMG2).expect("could not write image to pixbufloader");
    let pixbuf2: gdk_pixbuf::Pixbuf = pixbuf_loader2.get_pixbuf().expect("could not get a pixbuf from the loaded image");
    let whatwhat: *mut gdk_pixbuf_sys::GdkPixbuf = pixbuf2.to_glib_none().0;

    let status_icon: *mut gtk_sys::GtkStatusIcon;
    unsafe {
        status_icon = gtk_sys::gtk_status_icon_new();
        gtk_sys::gtk_status_icon_set_from_pixbuf(status_icon, whowho);
        connect_activate(status_icon, move |status_icon: *mut gtk_sys::GtkStatusIcon| {
            gtk_sys::gtk_status_icon_set_from_pixbuf(status_icon, whatwhat);
            println!("clicked!!!");
        });
    }

    gtk::main();
}
