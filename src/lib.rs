#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]

mod scheduler;
mod prelude;
pub mod ui;

use scheduler::Scheduler;
use prelude::*;

const APPLICATION_ID: Option<&str> = Some("com.github.cdepillabout.break-time");

fn app_activate(app: gtk::Application) {

    let scheduler = Scheduler::new();

    // let (sender, receiver) =
    //     glib::MainContext::channel(glib::source::PRIORITY_DEFAULT);

    // let state = State::new(app, sender);

    // setup(&state);

    // connect_events(&state);

    // redisplay(&state);

    // for (window, monitor) in state.get_app_wins_with_monitors() {
    //     window.show_all();

    //     let monitor_rect = monitor.get_geometry();
    //     let gdk_window: gdk::Window = window.get_window().expect(
    //         "Gtk::Window should always be able to be converted to Gdk::Window",
    //     );
    //     gdk_window.fullscreen_on_monitor(monitor.id);
    //     // gdk_window.resize(monitor_rect.width, monitor_rect.height);
    // }

    // receiver.attach(
    //     None,
    //     clone!(@strong state => move |msg| {
    //         handle_msg_recv(&state, msg);
    //         glib::source::Continue(true)
    //     }),
    // );
}

pub fn default_main() {

    let uiapp = gtk::Application::new(APPLICATION_ID, gio::ApplicationFlags::FLAGS_NONE)
    .expect("Application::new failed");

    uiapp.connect_activate(|app| app_activate(app.clone()));

    // uiapp.run(&env::args().collect::<Vec<_>>());
    uiapp.run(&[]);
}
