use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Atom, AtomEnum, ConnectionExt, Window};
use x11rb::rust_connection::RustConnection;

pub struct X11 {
    pub conn: RustConnection,
    pub preferred_screen: usize,
}

impl X11 {
    pub fn connect() -> Self {
        let (conn, preferred_screen) =
            x11rb::connect(None).expect("Could not connect to X server");

        Self {
            conn,
            preferred_screen,
        }
    }

    pub fn create_atom(&self, atom_name: &str) -> Option<Atom> {
        self.conn
            .intern_atom(false, atom_name.as_bytes())
            .ok()?
            .reply()
            .ok()
            .map(|rep| rep.atom)
    }

    pub fn get_root_win(&self) -> Option<Window> {
        let setup = self.conn.setup();
        setup
            .roots
            .get(self.preferred_screen)
            .map(|screen| screen.root)
    }

    pub fn get_win_prop(&self, win: Window, atom: Atom) -> Option<Window> {
        let reply = self
            .conn
            .get_property(false, win, atom, AtomEnum::WINDOW, 0, 1)
            .ok()?
            .reply()
            .ok()?;

        // No value available, or the value is more than 1 (which is unexpected).
        if reply.value_len != 1 {
            return None;
        }

        // Window properties are expected to be 32-bit values.
        let window = reply.value32()?.next()?;

        if window == x11rb::NONE {
            None
        } else {
            Some(window)
        }
    }
}
