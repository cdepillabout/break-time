
use gdk_pixbuf::prelude::*;
use glib::translate::*;

static IMG: &'static [u8] = include_bytes!("../../imgs/clock.png");

fn main() {
    gtk::init().unwrap();

    let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();
    pixbuf_loader.write(IMG).expect("could not write image to pixbufloader");
    let pixbuf = pixbuf_loader.get_pixbuf().expect("could not get a pixbuf from the loaded image");

    let status_icon;
    unsafe {
        status_icon = gtk_sys::gtk_status_icon_new();
        let whowho = pixbuf.to_glib_none().0;
        gtk_sys::gtk_status_icon_set_from_pixbuf(status_icon, whowho);
    }

    gtk::main();
}
