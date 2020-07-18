use crate::prelude::*;

use byteorder::{LittleEndian, ReadBytesExt};

pub struct X11 {
    pub conn: xcb::Connection,
    pub preferred_screen: i32,
}

impl X11 {
    pub fn connect() -> Self {
        let (conn, preferred_screen) = xcb::Connection::connect(None)
            .expect("Could not connect to X server");

        Self {
            conn,
            preferred_screen,
        }
    }

    pub fn create_atom(&self, atom_name: &str) -> Option<xcb::Atom> {
        let net_wm_name_atom_cookie =
            xcb::intern_atom(&self.conn, false, atom_name);

        net_wm_name_atom_cookie
            .get_reply()
            .ok()
            .map(|rep| rep.atom())
    }

    pub fn get_root_win(&self) -> Option<xcb::Window> {
        let setup: xcb::Setup = self.conn.get_setup();
        let mut roots: xcb::ScreenIterator = setup.roots();
        let preferred_screen_pos = usize::try_from(self.preferred_screen).expect("x11 preferred_screen is not positive");
        roots
            .nth(preferred_screen_pos)
            .map(|screen| screen.root())
    }

    pub fn get_win_prop(
        &self,
        win: xcb::Window,
        atom: xcb::Atom,
    ) -> Option<xcb::Window> {
        let reply = xcb::get_property(
            &self.conn,
            false,
            win,
            atom,
            xcb::ATOM_WINDOW,
            0,
            1,
        )
        .get_reply()
        .ok()?;

        // No value available, or the value is more than 1 (which is unexpected).
        if reply.value_len() != 1 {
            return None;
        }

        let mut raw = reply.value();

        // Window properties are expected to be 4 bytes.
        if raw.len() != 4 {
            return None;
        }

        let window = raw.read_u32::<LittleEndian>().unwrap() as xcb::Window;

        if window == xcb::WINDOW_NONE {
            None
        } else {
            Some(window)
        }
    }
}
