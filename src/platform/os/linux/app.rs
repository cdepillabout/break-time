//! Linux app runtime: the GTK main loop plus the glib channel that delivers
//! `Msg`s to it. This is the concrete backing for the cross-platform
//! app-runtime seam (`platform::{channel, run_main_loop, quit, AppSender,
//! AppReceiver}`); the macOS counterpart lives in
//! `platform::os::macos::app`.

use crate::Msg;

/// Sender half of the app message channel. On Linux this is a glib channel
/// serviced by the GTK main loop, so a send wakes the loop. Cloneable and
/// `Send`, so the tray, scheduler thread, and break window can each hold one.
pub type AppSender = glib::Sender<Msg>;

/// Receiver half, consumed by [`run_main_loop`].
pub type AppReceiver = glib::Receiver<Msg>;

/// Initialize the GUI toolkit and create the app message channel. Must be called
/// before any tray/window construction.
#[must_use]
pub fn channel() -> (AppSender, AppReceiver) {
    gtk::init().expect("Could not initialize GTK");

    // `MainContext::channel` is deprecated in glib 0.18 (removed in 0.20) in
    // favour of async-channel + `spawn_future_local`. Migrating the synchronous
    // `Msg` plumbing to async channels is deferred to a later glib/GTK upgrade;
    // suppress the deprecation for now.
    #[allow(deprecated)]
    glib::MainContext::channel(glib::Priority::DEFAULT)
}

/// Attach `handler` to the main loop (invoked on the main thread for each `Msg`)
/// and run the loop until [`quit`] is called.
pub fn run_main_loop(
    receiver: AppReceiver,
    mut handler: impl FnMut(Msg) + 'static,
) {
    receiver.attach(None, move |msg| {
        handler(msg);
        glib::ControlFlow::Continue
    });

    gtk::main();
}

/// Stop the main loop; causes [`run_main_loop`] to return.
pub fn quit() {
    gtk::main_quit();
}
