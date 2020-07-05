pub mod builder;
pub mod prelude;

mod css;
mod state;

use glib::clone;
use glib::source::Continue;
use gtk::Inhibit;
use std::time::{Duration, Instant, SystemTime};

use crate::config::Config;
use super::Msg;
use prelude::*;
use state::{Message, State};

use crate::x11::X11;

const XCB_EWMH_CLIENT_SOURCE_TYPE_OTHER: u32 = 2;

fn handle_msg_recv(
    state: &State,
    msg: Message,
    x11: &X11,
    root_win: xcb::Window,
    net_active_win_atom: xcb::Atom,
    option_old_active_win: Option<xcb::Window>,
) -> Continue {
    // enable(state);

    match msg {
        Message::Display => Continue(true),
        Message::End => {
            for window in state.get_app_wins() {
                window.hide();
                window.destroy();
            }
            state.notify_app_end();

            focus_previous_window(x11, root_win, net_active_win_atom, option_old_active_win);

            Continue(false)
        }
    }
}

fn focus_previous_window(
    x11: &X11,
    root_win: xcb::Window,
    net_active_win_atom: xcb::Atom,
    option_old_active_win: Option<xcb::Window>,
) {
    if let Some(old_active_win) = option_old_active_win {
        let message_data = xcb::ClientMessageData::from_data32([
            XCB_EWMH_CLIENT_SOURCE_TYPE_OTHER,
            xcb::CURRENT_TIME,
            xcb::WINDOW_NONE,
            0,
            0,
        ]);

        let message_event = xcb::ClientMessageEvent::new(
            // Data size (8-bit, 16-bit, or 32-bit).  This message is 32-bit.
            32,
            old_active_win,
            net_active_win_atom,
            message_data,
        );

        let res = xcb::send_event(
            &x11.conn,
            false,
            root_win,
            xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY
                | xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT,
            &message_event,
        )
        .request_check();

        match res {
            Ok(()) => (),
            Err(err) => {
                println!("Could not focus old focused window: {}", err)
            }
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

fn connect_events(config: &Config, state: &State) {
    for window in state.get_app_wins() {
        window.connect_key_press_event(
            clone!(@strong state => move |_, event_key| {
                if event_key.get_keyval() == gdk::enums::key::space {
                    decrement_presses_remaining(&state);
                    redisplay(&state);
                }
                Inhibit(false)
            }),
        );
    }

    // the full time we want to wait for
    let full_time = Duration::new(config.settings.break_duration_seconds.into(), 0);

    gtk::timeout_add(
        200,
        clone!(@strong state => move || update_time_remaining(&state, full_time))
    );
}

fn update_time_remaining(state: &State, full_time: Duration) -> Continue {
    let system_time_now = SystemTime::now();
    let option_system_time_diff = system_time_now.duration_since(state.start_time).ok();
    let option_system_time_remaining = option_system_time_diff.and_then(|system_time_diff| full_time.checked_sub(system_time_diff));

    match option_system_time_remaining {
        None => {
            end_break(&state);
            Continue(false)
        }
        Some(system_time_remaining) => {
            for label in state.get_time_remaining_labels() {
                let total_secs_remaining = system_time_remaining.as_secs();
                let mins: u64 = total_secs_remaining / 60;
                let secs: u64 = total_secs_remaining % 60;
                label.set_text(&format!("{:02}:{:02}", mins, secs));
            }
            Continue(true)
        }
    }
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
        window.set_default_size(monitor_rect.width, monitor_rect.height);
        window.resize(monitor_rect.width, monitor_rect.height);
        window.move_(monitor_rect.x, monitor_rect.y);

        let gdk_window: gdk::Window = window.get_window().expect(
            "Gtk::Window should always be able to be converted to Gdk::Window",
        );

        // Grab the mouse and keyboard on the first Window.
        if i == 0 {
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
                            if seat_grab_check_times > 1 {
                                "tries"
                            } else {
                                "try"
                            }
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

pub fn start_break(config: &Config, app_sender: glib::Sender<Msg>) {
    let x11 = X11::connect();

    let net_active_win_atom = x11.create_atom("_NET_ACTIVE_WINDOW").expect(
        "Could not get the _NET_ACTIVE_WINDOW value from the X server.",
    );
    let root_win = x11
        .get_root_win()
        .expect("Could not get the root window from the X server.");
    let old_active_win = x11.get_win_prop(root_win, net_active_win_atom);

    println!("previous active_win: {:?}", old_active_win);

    let (sender, receiver) =
        glib::MainContext::channel(glib::source::PRIORITY_DEFAULT);

    let state = State::new(config, app_sender, sender);

    setup(&state);

    connect_events(&config, &state);

    redisplay(&state);

    setup_windows(&state);

    receiver.attach(
        None,
        clone!(@strong state => move |msg|
            handle_msg_recv(
                &state,
                msg,
                &x11,
                root_win,
                net_active_win_atom,
                old_active_win
            )
        ),
    );
}
