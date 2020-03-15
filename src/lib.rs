#![warn(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]

mod scheduler;
mod prelude;
mod tray;
pub mod ui;

use scheduler::Scheduler;
use prelude::*;

// fn app_activate(app: gtk::Application) {

//     let (sender, receiver) =
//         glib::MainContext::channel(glib::source::PRIORITY_DEFAULT);

//     let state = State::new(app, sender);

//     setup(&state);

//     connect_events(&state);

//     redisplay(&state);

//     for (window, monitor) in state.get_app_wins_with_monitors() {
//         window.show_all();

//         let monitor_rect = monitor.get_geometry();
//         let gdk_window: gdk::Window = window.get_window().expect(
//             "Gtk::Window should always be able to be converted to Gdk::Window",
//         );
//         gdk_window.fullscreen_on_monitor(monitor.id);
//         // gdk_window.resize(monitor_rect.width, monitor_rect.height);
//     }

//     receiver.attach(
//         None,
//         clone!(@strong state => move |msg| {
//             handle_msg_recv(&state, msg);
//             glib::source::Continue(true)
//         }),
//     );
    

// }

pub fn default_main() {
    gtk::init().expect("Could not initialize GTK");

    let scheduler = Scheduler::new();

    let (status_icon, pixbuf) = tray::show();

    let is_vis;
    let is_embed;

    unsafe {
        is_vis = gtk_sys::gtk_status_icon_get_visible(status_icon);
        is_embed = gtk_sys::gtk_status_icon_is_embedded(status_icon);
    }

    println!("is vis: {}, is_embed: {}", is_vis, is_embed);

    gtk::main();
}
