mod builder;
mod css;
mod prelude;
mod state;

use glib::clone;
use gtk::Inhibit;

use prelude::*;
use state::{Message, State};

fn handle_msg_recv(state: &State, msg: Message) {
    // enable(state);

    match msg {
        Message::Display => (),
    }
}

fn decrement_presses_remaining(state: &State) {
    let remaining = state.decrement_presses_remaining();

    if remaining == 0 {
        state.app.quit();
    }
}

fn app_activate(app: gtk::Application) {
    let (sender, receiver) =
        glib::MainContext::channel(glib::source::PRIORITY_DEFAULT);

    let state = State::new(app, sender);

    let window: gtk::ApplicationWindow = state.get_app_win();
    window.set_application(Some(&state.app));

    css::setup(window.upcast_ref());

    window.connect_key_press_event(clone!(@strong state => move |_, event_key| {
        if event_key.get_keyval() == gdk::enums::key::space {
            println!("Got key press event for space: {:?}", event_key);
            decrement_presses_remaining(&state);
            Inhibit(true)
        } else {
            Inhibit(false)
        }
    }));

    window.show_all();

    receiver.attach(
        None,
        clone!(@strong state => move |msg| {
            handle_msg_recv(&state, msg);
            glib::source::Continue(true)
        }),
    );

    // Do the initial search and display the results.
    // let opts = crate::opts::Opts::parse_from_args();
    // search_for(&state, &opts.nix_store_path);
}

pub fn run() {
    let uiapp = gtk::Application::new(
        Some("com.github.cdepillabout.break-time"),
        gio::ApplicationFlags::FLAGS_NONE,
    )
    .expect("Application::new failed");

    uiapp.connect_activate(|app| app_activate(app.clone()));

    // uiapp.run(&env::args().collect::<Vec<_>>());
    uiapp.run(&[]);
}
