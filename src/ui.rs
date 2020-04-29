pub mod builder;
pub mod prelude;

mod css;
mod state;

use glib::clone;
use glib::source::Continue;
use gtk::Inhibit;
use std::time::{Duration, Instant};

use super::Msg;
use prelude::*;
use state::{Message, State};

use crate::x11::X11;

fn handle_msg_recv(state: &State, msg: Message /*, x_conn: &xcb::Connection, original_focused_win: &xcb::GetInputFocusReply*/ ) -> Continue {
    // enable(state);

    match msg {
        Message::Display => Continue(true),
        Message::End => {
            for window in state.get_app_wins() {
                window.hide();
                window.destroy();
            }
            state.notify_app_end();

            // xcb::set_input_focus(x_conn, original_focused_win.revert_to(), original_focused_win.focus(), xcb::CURRENT_TIME);

            Continue(false)
        }
    }
}

fn end_break(state: &State) {
    state.end();
}

fn decrement_presses_remaining(state: &State) {
    let remaining = state.decrement_presses_remaining();

    if remaining == 0 {
        end_break(state);
    }
}

fn setup(state: &State) {
    for window in state.get_app_wins() {
        css::setup(window.upcast_ref());
    }
}

fn connect_events(state: &State) {
    for window in state.get_app_wins() {
        window.connect_key_press_event(
            clone!(@strong state => move |_, event_key| {
                if event_key.get_keyval() == gdk::enums::key::space {
                    decrement_presses_remaining(&state);
                    redisplay(&state);
                    Inhibit(true)
                } else {
                    Inhibit(false)
                }
            }),
        );
    }

    gtk::timeout_add(
        200,
        clone!(@strong state => move || {

            let now = Instant::now();
            let time_diff = now.saturating_duration_since(state.start_instant);

            // the full time we want to wait for
            let full_time = Duration::new(20, 0);

            let option_time_remaining = full_time.checked_sub(time_diff);

            match option_time_remaining {
                None => {
                    end_break(&state);
                    Continue(false)
                }
                Some(time_remaining) => {
                    for label in state.get_time_remaining_labels() {
                        label.set_text(&format!("{:?}", time_remaining));
                    }
                    Continue(true)
                }
            }

        }),
    );
}

fn redisplay(state: &State) {
    let presses_remaining = state.read_presses_remaining();

    for label in state.get_presses_remaining_labels() {
        label.set_text(&format!("{}", presses_remaining));
    }
}

fn setup_windows(state: &State) {
    let app_wins_with_monitors = state.get_app_wins_with_monitors();

    for (i, (window, monitor)) in app_wins_with_monitors.into_iter().enumerate()
    {
        window.show_all();

        let monitor_rect = monitor.get_geometry();
        let gdk_window: gdk::Window = window.get_window().expect(
            "Gtk::Window should always be able to be converted to Gdk::Window",
        );
        gdk_window.fullscreen_on_monitor(monitor.id);
        gdk_window.resize(monitor_rect.width, monitor_rect.height);

        // Grab the mouse and keyboard on the first Window.
        if i == 1 {
            let mut seat_grab_check_times = 0;
            // For some reason, grab() fails unless we wait for a while until the window is fully
            // shown.
            gtk::idle_add(move || {
                seat_grab_check_times += 1;
                let ten_millis = std::time::Duration::from_millis(200);
                std::thread::sleep(ten_millis);

                let default_display = gdk::Display::get_default()
                    .expect("gdk should always find a Display when it runs");

                let default_seat = default_display
                    .get_default_seat()
                    .expect("gdk Display should always have a deafult Seat");

                let grab_status = default_seat.grab(
                    &gdk_window,
                    gdk::SeatCapabilities::ALL,
                    false,
                    None,
                    None,
                    None,
                );

                match grab_status {
                    gdk::GrabStatus::Success => {
                        println!(
                            "Successfully grabbed screen after {} {}.",
                            seat_grab_check_times,
                            if seat_grab_check_times > 1 { "tries" } else { "try" }
                        );
                        Continue(false)
                    }
                    _ => {
                        if seat_grab_check_times >= 20 {
                            println!("Tried grabbing keyboard/mouse {} times, but never succeeded.", seat_grab_check_times);
                            Continue(false)
                        } else {
                            Continue(true)
                        }
                    }
                }
            });
        }
    }
}

pub fn start_break(app_sender: glib::Sender<Msg>) {

    let x11 = X11::connect();

    let net_active_window_atom = x11.create_atom("_NET_ACTIVE_WINDOW").expect("Could not get the _NET_ACTIVE_WINDOW value from the X server.");
    let root_win = x11.get_root_win().expect("Could not get the root window from the X server.");
    let active_win = x11.get_win_prop(root_win, net_active_window_atom);

    println!("active_win: {:?}", active_win);

    let (sender, receiver) =
        glib::MainContext::channel(glib::source::PRIORITY_DEFAULT);

    let state = State::new(app_sender, sender);

    setup(&state);

    connect_events(&state);

    redisplay(&state);

    setup_windows(&state);

    receiver.attach(
        None,
        clone!(@strong state => move |msg| handle_msg_recv(&state, msg /*, &x_conn, &original_focused_win*/)),
    );
}
