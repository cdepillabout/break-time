mod builder;
mod css;
mod prelude;
mod state;

use glib::clone;
use gtk::Inhibit;
use std::time::{Duration, Instant};

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

fn setup(state: &State) {
    let window: gtk::ApplicationWindow = state.get_app_win();
    window.set_application(Some(&state.app));
    window.fullscreen();

    css::setup(window.upcast_ref());

}

fn connect_events(state: &State) {
    let window: gtk::ApplicationWindow = state.get_app_win();
    window.connect_key_press_event(clone!(@strong state => move |_, event_key| {
        if event_key.get_keyval() == gdk::enums::key::space {
            decrement_presses_remaining(&state);
            redisplay(&state);
            Inhibit(true)
        } else {
            Inhibit(false)
        }
    }));

    gtk::timeout_add(200, clone!(@strong state => move || {
        let time_remaining_label = state.get_time_remaining_label();

        let now = Instant::now();
        let time_diff = now.saturating_duration_since(state.start_instant);

        // the full time we want to wait for
        let full_time = Duration::new(20, 0);

        let option_time_remaining = full_time.checked_sub(time_diff);

        match option_time_remaining {
            None => {
                state.app.quit();
            }
            Some(time_remaining) => {
                time_remaining_label.set_text(&format!("{:?}", time_remaining));
            }
        }

        glib::source::Continue(true)
    }));
}

fn redisplay(state: &State) {
    let presses_remaining_label = state.get_presses_remaining_label();

    let presses_remaining = state.read_presses_remaining();
    presses_remaining_label.set_text(&format!("{}", presses_remaining));
}

fn app_activate(app: gtk::Application) {
    let (sender, receiver) =
        glib::MainContext::channel(glib::source::PRIORITY_DEFAULT);

    let state = State::new(app, sender);

    setup(&state);

    connect_events(&state);

    redisplay(&state);

    let window: gtk::ApplicationWindow = state.get_app_win();
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
