
use super::Plugin;

pub struct WindowTitles {
    conn: xcb::Connection,
    screen_num: i32,
    net_wm_name_atom: xcb::Atom,
    utf8_string_atom: xcb::Atom,
}

impl WindowTitles {
    pub fn new() -> Result<Self, ()> {
        let (conn, screen_num) = xcb::Connection::connect(None).unwrap();

        let net_wm_name_atom_cookie = xcb::intern_atom(&conn, false, "_NET_WM_NAME");
        let utf8_string_atom_cookie = xcb::intern_atom(&conn, false, "UTF8_STRING");

        let net_wm_name_atom = net_wm_name_atom_cookie.get_reply().map_err(|_| ())?.atom();
        let utf8_string_atom = utf8_string_atom_cookie.get_reply().map_err(|_| ())?.atom();

        Ok(
            WindowTitles {
                conn,
                screen_num,
                net_wm_name_atom,
                utf8_string_atom,
            }
        )
    }

    fn request_window_props<'a>(&'a self, win: xcb::Window) -> WindowPropCookies<'a> {
        let wm_name_cookie = xcb::xproto::get_property(
            &self.conn,
            false,
            win,
            xcb::xproto::ATOM_WM_NAME,
            xcb::xproto::ATOM_STRING,
            PROP_STARTING_OFFSET,
            PROP_LENGTH_TO_GET,
        );

        let net_wm_name_cookie = xcb::xproto::get_property(
            &self.conn,
            false,
            win,
            self.net_wm_name_atom,
            self.utf8_string_atom,
            PROP_STARTING_OFFSET,
            PROP_LENGTH_TO_GET,
        );

        let wm_class_cookie = xcb::xproto::get_property(
            &self.conn,
            false,
            win,
            xcb::xproto::ATOM_WM_CLASS,
            xcb::xproto::ATOM_STRING,
            PROP_STARTING_OFFSET,
            PROP_LENGTH_TO_GET,
        );

        let wm_transient_for_cookie = xcb::xproto::get_property(
            &self.conn,
            false,
            win,
            xcb::xproto::ATOM_WM_TRANSIENT_FOR,
            xcb::xproto::ATOM_WINDOW,
            PROP_STARTING_OFFSET,
            PROP_LENGTH_TO_GET,
        );

        WindowPropCookies {
            wm_name: wm_name_cookie,
            net_wm_name: net_wm_name_cookie,
            wm_class: wm_class_cookie,
            wm_transient_for: wm_transient_for_cookie,
        }
    }
}

const PROP_STARTING_OFFSET: u32 = 0;
const PROP_LENGTH_TO_GET: u32 = 2048;

struct WindowPropCookies<'a> {
    wm_name: xcb::xproto::GetPropertyCookie<'a>,
    net_wm_name: xcb::xproto::GetPropertyCookie<'a>,
    wm_class: xcb::xproto::GetPropertyCookie<'a>,
    wm_transient_for: xcb::xproto::GetPropertyCookie<'a>,
}

struct WindowProp {
}

impl Plugin for WindowTitles {
    fn can_break_now(&self) -> Result<bool, ()> {
        let setup: xcb::Setup = self.conn.get_setup();
        let mut roots: xcb::ScreenIterator = setup.roots();
        let screen: xcb::Screen = roots.nth(self.screen_num as usize).ok_or(())?;
        let root_window: xcb::Window = screen.root();

        let query_tree_reply: xcb::QueryTreeReply =
            xcb::xproto::query_tree(&self.conn, root_window)
                .get_reply()
                .map_err(|_| ())?;

        let windows: &[u32] = query_tree_reply.children();

        let cookies: Vec<WindowPropCookies> = windows.iter().map(|win| self.request_window_props(*win)).collect();

        // cookies.iter().map(

        // TODO: Finish writing this.

        Ok(true)
    }
}
