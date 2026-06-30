//! X11 display backend, built on the pure-Rust `x11rb` protocol implementation.
//! Wraps the low-level [`X11`] connection helper and owns the X11-specific window
//! enumeration that feeds the cross-platform meeting-detection predicates.

mod conn;

use std::time::Duration;

use x11rb::protocol::screensaver::ConnectionExt as _;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ClientMessageEvent, ConnectionExt as _, EventMask, Window,
};

use super::{DisplayBackend, WindowInfo, WindowRef};
use conn::X11;

const PROP_STARTING_OFFSET: u32 = 0;
const PROP_LENGTH_TO_GET: u32 = 2048;

// EWMH source indication for the _NET_ACTIVE_WINDOW client message: 2 == "other".
const XCB_EWMH_CLIENT_SOURCE_TYPE_OTHER: u32 = 2;

pub struct X11Backend {
    x11: X11,
    root: Window,
    net_active_window: Atom,
    net_wm_name: Atom,
    utf8_string: Atom,
}

impl X11Backend {
    pub fn connect() -> Self {
        let x11 = X11::connect();
        let root = x11
            .get_root_win()
            .expect("X11: could not determine the root window");
        // Interning these core atoms can only realistically fail on a broken X
        // server; match the previous behaviour and take the process down at
        // startup if so.
        let net_active_window = x11
            .create_atom("_NET_ACTIVE_WINDOW")
            .expect("X11: could not intern the _NET_ACTIVE_WINDOW atom");
        let net_wm_name = x11
            .create_atom("_NET_WM_NAME")
            .expect("X11: could not intern the _NET_WM_NAME atom");
        let utf8_string = x11
            .create_atom("UTF8_STRING")
            .expect("X11: could not intern the UTF8_STRING atom");
        Self {
            x11,
            root,
            net_active_window,
            net_wm_name,
            utf8_string,
        }
    }

    fn get_string_prop(
        &self,
        win: Window,
        property: Atom,
        type_: Atom,
    ) -> Result<String, ()> {
        let reply = self
            .x11
            .conn
            .get_property(
                false,
                win,
                property,
                type_,
                PROP_STARTING_OFFSET,
                PROP_LENGTH_TO_GET,
            )
            .map_err(|_| ())?
            .reply()
            .map_err(|_| ())?;
        String::from_utf8(reply.value).map_err(|_| ())
    }

    fn get_class_info(&self, win: Window) -> ClassInfo<()> {
        let res_value = self
            .x11
            .conn
            .get_property(
                false,
                win,
                AtomEnum::WM_CLASS,
                AtomEnum::STRING,
                PROP_STARTING_OFFSET,
                PROP_LENGTH_TO_GET,
            )
            .map_err(|_| ())
            .and_then(|cookie| cookie.reply().map_err(|_| ()));

        match res_value {
            Err(()) => ClassInfo::err(()),
            Ok(reply) => ClassInfo::from_raw_data(&reply.value, (), |_| ()),
        }
    }

    fn get_transient_for(&self, win: Window) -> Result<Vec<WindowRef>, ()> {
        let reply = self
            .x11
            .conn
            .get_property(
                false,
                win,
                AtomEnum::WM_TRANSIENT_FOR,
                AtomEnum::WINDOW,
                PROP_STARTING_OFFSET,
                PROP_LENGTH_TO_GET,
            )
            .map_err(|_| ())?
            .reply()
            .map_err(|_| ())?;
        Ok(reply
            .value32()
            .map(|iter| iter.map(WindowRef).collect())
            .unwrap_or_default())
    }

    fn get_win_props(&self, win: Window) -> WindowInfo {
        let wm_name = self.get_string_prop(
            win,
            AtomEnum::WM_NAME.into(),
            AtomEnum::STRING.into(),
        );
        let net_wm_name =
            self.get_string_prop(win, self.net_wm_name, self.utf8_string);
        let transient_for = self.get_transient_for(win);
        let ClassInfo {
            name: class_name,
            class,
        } = self.get_class_info(win);

        WindowInfo {
            wm_name,
            net_wm_name,
            class_name,
            class,
            transient_for,
        }
    }
}

impl DisplayBackend for X11Backend {
    fn idle_time(&self) -> Duration {
        // Matches the previous behaviour of the idle detector: a failed query
        // panics (rather than silently reporting zero idle time, which would
        // suppress idle detection).
        let info = self
            .x11
            .conn
            .screensaver_query_info(self.root)
            .expect("X11: ScreenSaverQueryInfo request failed")
            .reply()
            .expect("X11: ScreenSaverQueryInfo reply failed");
        Duration::from_millis(u64::from(info.ms_since_user_input))
    }

    fn list_windows(&self) -> Result<Vec<WindowInfo>, ()> {
        let query_tree_reply = self
            .x11
            .conn
            .query_tree(self.root)
            .map_err(|_| ())?
            .reply()
            .map_err(|_| ())?;

        Ok(query_tree_reply
            .children
            .iter()
            .map(|win| self.get_win_props(*win))
            .collect())
    }

    fn save_active_window(&self) -> Option<WindowRef> {
        self.x11
            .get_win_prop(self.root, self.net_active_window)
            .map(WindowRef)
    }

    fn restore_active_window(&self, win: WindowRef) {
        let message_data: [u32; 5] = [
            XCB_EWMH_CLIENT_SOURCE_TYPE_OTHER,
            x11rb::CURRENT_TIME,
            x11rb::NONE,
            0,
            0,
        ];

        let message_event = ClientMessageEvent::new(
            // Data size (8-bit, 16-bit, or 32-bit). This message is 32-bit.
            32,
            win.0,
            self.net_active_window,
            message_data,
        );

        let res = self.x11.conn.send_event(
            false,
            self.root,
            EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT,
            message_event,
        );

        match res {
            Ok(cookie) => {
                if let Err(err) = cookie.check() {
                    println!("Could not focus old focused window: {err}");
                }
            }
            Err(err) => {
                println!("Could not focus old focused window: {err}");
            }
        }
    }
}

struct ClassInfo<T> {
    name: Result<String, T>,
    class: Result<String, T>,
}

impl<T: Clone> ClassInfo<T> {
    fn err(t: T) -> Self {
        Self {
            name: Err(t.clone()),
            class: Err(t),
        }
    }

    fn from_raw_data_with_index<F: Fn(std::string::FromUtf8Error) -> T>(
        raw: &[u8],
        index: usize,
        utf8_err_mapper: F,
    ) -> Self {
        Self {
            name: String::from_utf8(raw[0..index].to_vec())
                .map_err(&utf8_err_mapper),
            class: String::from_utf8(raw[index + 1..raw.len() - 1].to_vec())
                .map_err(utf8_err_mapper),
        }
    }

    fn from_raw_data<F: Fn(std::string::FromUtf8Error) -> T>(
        raw: &[u8],
        no_index_err: T,
        utf8_err_mapper: F,
    ) -> Self {
        let option_index = raw.iter().position(|&b| b == 0);
        match option_index {
            None => Self::err(no_index_err),
            Some(index) => {
                Self::from_raw_data_with_index(raw, index, utf8_err_mapper)
            }
        }
    }
}
