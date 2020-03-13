
static IMG: &'static [u8] = include_bytes!("../../imgs/clock.png");

fn main() {
    gtk::init().unwrap();

    let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();

    let status_icon;
    unsafe {
        status_icon = gtk_sys::gtk_status_icon_new();


    }

    gtk::main();
}
