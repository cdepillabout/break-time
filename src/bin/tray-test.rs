use gdk_pixbuf::prelude::*;
use glib::translate::*;
use gtk::prelude::*;
use std::io::prelude::*;

static IMG: &'static [u8] = include_bytes!("../../imgs/clock.png");
static IMG2: &'static [u8] = include_bytes!("../../imgs/clock-2.png");

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

// fn connect_popup_menu<F: Fn(&Self) -> bool + 'static>(&self, f: F) -> SignalHandlerId {
//     unsafe extern "C" fn popup_menu_trampoline<P, F: Fn(&P) -> bool + 'static>(
//         this: *mut gtk_sys::GtkWidget,
//         f: glib_sys::gpointer,
//     ) -> glib_sys::gboolean
//     where
//         P: IsA<Widget>,
//     {
//         let f: &F = &*(f as *const F);
//         f(&Widget::from_glib_borrow(this).unsafe_cast()).to_glib()
//     }
//     unsafe {
//         let f: Box_<F> = Box_::new(f);
//         connect_raw(
//             self.as_ptr() as *mut _,
//             b"popup-menu\0".as_ptr() as *const _,
//             Some(transmute(popup_menu_trampoline::<Self, F> as usize)),
//             Box_::into_raw(f),
//         )
//     }
// }

fn main() {
    gtk::init().unwrap();

    let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();
    pixbuf_loader
        .write(IMG)
        .expect("could not write image to pixbufloader");
    let pixbuf = pixbuf_loader
        .get_pixbuf()
        .expect("could not get a pixbuf from the loaded image");
    let whowho: *mut gdk_pixbuf_sys::GdkPixbuf = pixbuf.to_glib_none().0;

    let pixbuf_loader2 = gdk_pixbuf::PixbufLoader::new();
    pixbuf_loader2
        .write(IMG2)
        .expect("could not write image to pixbufloader");
    let pixbuf2: gdk_pixbuf::Pixbuf = pixbuf_loader2
        .get_pixbuf()
        .expect("could not get a pixbuf from the loaded image");
    let whatwhat: *mut gdk_pixbuf_sys::GdkPixbuf = pixbuf2.to_glib_none().0;

    let status_icon: *mut gtk_sys::GtkStatusIcon;
    unsafe {
        status_icon = gtk_sys::gtk_status_icon_new();

        gtk_sys::gtk_status_icon_set_from_pixbuf(status_icon, whowho);

        gtk_sys::gtk_status_icon_set_tooltip_text(status_icon, "hello".to_glib_none().0);
    }

    connect_activate(
        status_icon,
        move |status_icon: *mut gtk_sys::GtkStatusIcon| {
            unsafe {
                gtk_sys::gtk_status_icon_set_from_pixbuf(status_icon, whatwhat);
            }
            println!("clicked!!!");
        },
    );

    connect_popup_menu(
        status_icon,
        move |status_icon: *mut gtk_sys::GtkStatusIcon, button, activate_time| {
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

    let mut whowhowho: &[u8] = IMG;
    let whowhowho1: &mut &[u8] = &mut whowhowho;

    let image_surface = cairo::ImageSurface::create_from_png(whowhowho1).expect("should create png from mem");

    let cr = cairo::Context::new(&image_surface);
    // cairo_select_font_face (cr, "serif", CAIRO_FONT_SLANT_NORMAL, CAIRO_FONT_WEIGHT_BOLD);
    // cairo_set_font_size (cr, 32.0);
    // cairo_set_source_rgb (cr, 0.0, 0.0, 1.0);
    // cairo_move_to (cr, 10.0, 50.0);
    // cairo_show_text (cr, "Hello, world");
    cr.select_font_face("serif", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
    cr.set_font_size(40.0);
    cr.set_source_rgb(1.0,0.0,0.0);
    cr.move_to(10.0, 50.0);
    cr.show_text("Hello, world");

    let mut file = std::fs::File::create("test.png").expect("no open");
    image_surface.write_to_png(&mut file).expect("fefefefefe");

    // pub fn create_from_png<R: Read>(stream: &mut R) -> Result<ImageSurface, IoError>
    // impl<'_> Read for &'_ [u8]
    // impl<'_, R: Read + ?Sized> Read for &'_ mut R
    // impl<R: Read + ?Sized> Read for Box<R>

    // pub fn create_from_png(stream: &mut (&'_ [u8])) -> Result<ImageSurface, IoError>
    
    // impl Write for File

    gtk::main();
}
