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

        let net_wm_name_atom_cookie =
            xcb::intern_atom(&conn, false, "_NET_WM_NAME");
        let utf8_string_atom_cookie =
            xcb::intern_atom(&conn, false, "UTF8_STRING");

        let net_wm_name_atom =
            net_wm_name_atom_cookie.get_reply().map_err(|_| ())?.atom();
        let utf8_string_atom =
            utf8_string_atom_cookie.get_reply().map_err(|_| ())?.atom();

        Ok(WindowTitles {
            conn,
            screen_num,
            net_wm_name_atom,
            utf8_string_atom,
        })
    }

    fn request_win_props<'a>(&'a self, win: xcb::Window) -> WinPropCookies<'a> {
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

        WinPropCookies {
            wm_name: wm_name_cookie,
            net_wm_name: net_wm_name_cookie,
            wm_class: wm_class_cookie,
            wm_transient_for: wm_transient_for_cookie,
        }
    }
}

const PROP_STARTING_OFFSET: u32 = 0;
const PROP_LENGTH_TO_GET: u32 = 2048;

struct WinPropCookies<'a> {
    wm_name: xcb::xproto::GetPropertyCookie<'a>,
    net_wm_name: xcb::xproto::GetPropertyCookie<'a>,
    wm_class: xcb::xproto::GetPropertyCookie<'a>,
    wm_transient_for: xcb::xproto::GetPropertyCookie<'a>,
}

struct WinProps {
    wm_name: Result<String, ()>,
    net_wm_name: Result<String, ()>,
    transient_for_wins: Result<Vec<xcb::Window>, ()>,
}

fn get_win_props(win_prop_cookies: WinPropCookies) -> WinProps {
    let wm_name = win_prop_cookies
        .wm_name
        .get_reply()
        .map_err(|generic_err| ())
        .and_then(|wm_name_reply| {
            let wm_name_vec = wm_name_reply.value().to_vec();
            String::from_utf8(wm_name_vec).map_err(|from_utf8_err| ())
        });

    let net_wm_name = win_prop_cookies
        .net_wm_name
        .get_reply()
        .map_err(|generic_err| ())
        .and_then(|net_wm_name_reply| {
            let net_wm_name_vec = net_wm_name_reply.value().to_vec();
            String::from_utf8(net_wm_name_vec).map_err(|from_utf8_err| ())
        });

    let transient_for_wins = win_prop_cookies
        .wm_transient_for
        .get_reply()
        .map_err(|generic_err| ())
        .map(|trans_reply| trans_reply.value().to_vec());

    let class = win_prop_cookies
        .wm_class
        .get_reply()
        .map_err(|generic_err| ())
        .map(|class_reply| {
            // TODO: Continue writing this...
            class_reply
        });

    WinProps { wm_name, net_wm_name, transient_for_wins }
}

impl Plugin for WindowTitles {
    fn can_break_now(&self) -> Result<bool, ()> {
        let setup: xcb::Setup = self.conn.get_setup();
        let mut roots: xcb::ScreenIterator = setup.roots();
        let screen: xcb::Screen =
            roots.nth(self.screen_num as usize).ok_or(())?;
        let root_window: xcb::Window = screen.root();

        let query_tree_reply: xcb::QueryTreeReply =
            xcb::xproto::query_tree(&self.conn, root_window)
                .get_reply()
                .map_err(|_| ())?;

        let wins: &[u32] = query_tree_reply.children();

        let cookies: Vec<WinPropCookies> = wins
            .iter()
            .map(|win| self.request_win_props(*win))
            .collect();

        let window_props = cookies
            .into_iter()
            .map(|win_prop_cookies| get_win_props(win_prop_cookies));

        // TODO: Finish writing this.

        Ok(true)
    }
}
