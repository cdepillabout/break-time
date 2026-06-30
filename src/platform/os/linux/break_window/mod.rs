pub mod builder;
pub mod prelude;

mod css;
mod state;

use glib::clone;
use glib::ControlFlow;
use std::time::{Duration, SystemTime};

use crate::config::Config;
use crate::platform::display::{DisplayBackend, WindowRef};
use crate::Msg;
use prelude::*;
use state::{Message, State};

fn handle_msg_recv(
    state: &State,
    msg: Message,
    display: &dyn DisplayBackend,
    saved_active_win: Option<WindowRef>,
) -> ControlFlow {
    match msg {
        Message::End => {
            for window in state.get_app_wins() {
                window.hide();
                window.close();
            }
            state.notify_app_end();

            if let Some(win) = saved_active_win {
                display.restore_active_window(win);
            }

            ControlFlow::Break
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
        window.connect_key_release_event(
            clone!(@strong state => move |_, event_key| {
                if event_key.keyval() == gdk::keys::constants::space {
                    decrement_presses_remaining(&state);
                    redisplay(&state);
                }
                glib::Propagation::Proceed
            }),
        );
    }

    // the full time we want to wait for
    let full_time =
        Duration::new(config.settings.break_duration_seconds.into(), 0);

    glib::timeout_add_local(
        Duration::from_millis(200),
        clone!(@strong state => move || update_time_remaining(&state, full_time)),
    );
}

fn update_time_remaining(state: &State, full_time: Duration) -> ControlFlow {
    // If the break was already ended (e.g. via spacebar), stop this timer.
    // Without this check, the timer keeps running after the break windows are
    // destroyed and eventually calls end_break(), which tries to send on a
    // channel whose receiver has already been dropped, causing a panic.
    if state.has_break_ended() {
        return ControlFlow::Break;
    }

    let system_time_now = SystemTime::now();
    let option_system_time_diff =
        system_time_now.duration_since(state.start_time).ok();
    let option_system_time_remaining = option_system_time_diff
        .and_then(|system_time_diff| full_time.checked_sub(system_time_diff));

    match option_system_time_remaining {
        None => {
            end_break(&state);
            ControlFlow::Break
        }
        Some(system_time_remaining) => {
            for label in state.get_time_remaining_labels() {
                let total_secs_remaining = system_time_remaining.as_secs();
                let mins: u64 = total_secs_remaining / 60;
                let secs: u64 = total_secs_remaining % 60;
                label.set_text(&format!("{mins:02}:{secs:02}"));
            }
            ControlFlow::Continue
        }
    }
}

fn redisplay(state: &State) {
    let presses_remaining = *state.read_presses_remaining();

    for label in state.get_presses_remaining_labels() {
        label.set_text(&format!("{presses_remaining}"));
    }
}

fn setup_windows(state: &State) {
    let app_wins_with_monitors = state.get_app_wins_with_monitors();

    for (i, (window, monitor)) in app_wins_with_monitors.into_iter().enumerate()
    {
        window.show_all();

        let monitor_rect = monitor.geometry();
        window.set_default_size(monitor_rect.width(), monitor_rect.height());
        window.resize(monitor_rect.width(), monitor_rect.height());
        window.move_(monitor_rect.x(), monitor_rect.y());

        let gdk_window: gdk::Window = window.window().expect(
            "Gtk::Window should always be able to be converted to Gdk::Window",
        );

        // Grab the mouse and keyboard on the first Window.
        if i == 0 {
            let mut seat_grab_check_times = 0;
            // For some reason, grab() fails unless we wait for a while until the window is fully
            // shown.
            glib::idle_add_local(move || {
                seat_grab_check_times += 1;
                let ten_millis = std::time::Duration::from_millis(200);
                std::thread::sleep(ten_millis);

                let default_display = gdk::Display::default()
                    .expect("gdk should always find a Display when it runs");

                let default_seat = default_display
                    .default_seat()
                    .expect("gdk Display should always have a default Seat");

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
                        ControlFlow::Break
                    }
                    _ => {
                        if seat_grab_check_times >= 20 {
                            println!("Tried grabbing keyboard/mouse {seat_grab_check_times} times, but never succeeded.");
                            ControlFlow::Break
                        } else {
                            ControlFlow::Continue
                        }
                    }
                }
            });
        }
    }
}

pub fn start_break(config: &Config, app_sender: glib::Sender<Msg>) {
    let display = crate::platform::create_display();
    let saved_active_win = display.save_active_window();

    println!("previous active_win: {saved_active_win:?}");

    // `MainContext::channel` is deprecated in glib 0.18 (removed in 0.20) in favour of
    // async-channel + `spawn_future_local`.  Migrating the synchronous `Msg`/`Message` plumbing to
    // async channels is deferred to a later glib/GTK upgrade; suppress the deprecation for now.
    #[allow(deprecated)]
    let (sender, receiver) =
        glib::MainContext::channel(glib::Priority::DEFAULT);

    let state = State::new(config, app_sender, sender);

    setup(&state);

    connect_events(config, &state);

    redisplay(&state);

    setup_windows(&state);

    // `display` (the backend) and the saved active window are moved into this
    // callback so the active window can be restored when the break ends.
    receiver.attach(
        None,
        clone!(@strong state => move |msg|
            handle_msg_recv(&state, msg, display.as_ref(), saved_active_win)
        ),
    );
}
