use super::{CanBreak, Plugin};

use crate::config::Config;
use crate::prelude::*;

use crate::x11::X11;

pub struct WindowTitles {
    x11: X11,
    net_wm_name_atom: xcb::Atom,
    utf8_string_atom: xcb::Atom,
}

impl WindowTitles {
    pub fn new(_config: &Config) -> Result<Self, ()> {
        let x11 = X11::connect();

        let net_wm_name_atom = x11.create_atom("_NET_WM_NAME").ok_or(())?;
        let utf8_string_atom = x11.create_atom("UTF8_STRING").ok_or(())?;

        Ok(Self {
            x11,
            net_wm_name_atom,
            utf8_string_atom,
        })
    }

    fn request_win_props(&'_ self, win: xcb::Window) -> WinPropCookies<'_> {
        let wm_name_cookie = xcb::xproto::get_property(
            &self.x11.conn,
            false,
            win,
            xcb::xproto::ATOM_WM_NAME,
            xcb::xproto::ATOM_STRING,
            PROP_STARTING_OFFSET,
            PROP_LENGTH_TO_GET,
        );

        let net_wm_name_cookie = xcb::xproto::get_property(
            &self.x11.conn,
            false,
            win,
            self.net_wm_name_atom,
            self.utf8_string_atom,
            PROP_STARTING_OFFSET,
            PROP_LENGTH_TO_GET,
        );

        let wm_class_cookie = xcb::xproto::get_property(
            &self.x11.conn,
            false,
            win,
            xcb::xproto::ATOM_WM_CLASS,
            xcb::xproto::ATOM_STRING,
            PROP_STARTING_OFFSET,
            PROP_LENGTH_TO_GET,
        );

        let wm_transient_for_cookie = xcb::xproto::get_property(
            &self.x11.conn,
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

    fn request_all_win_props<'a>(
        &'a self,
        wins: &[xcb::Window],
    ) -> Vec<WinPropCookies<'a>> {
        wins.iter()
            .map(|win| self.request_win_props(*win))
            .collect()
    }

    fn get_all_win_props_from_wins(
        &self,
        wins: &[xcb::Window],
    ) -> Vec<WinProps> {
        let win_prop_cookies: Vec<WinPropCookies> =
            self.request_all_win_props(wins);
        WinProps::get_all(win_prop_cookies)
    }

    fn get_all_win_props(&self) -> Result<Vec<WinProps>, ()> {
        let wins = self.get_all_wins()?;
        Ok(self.get_all_win_props_from_wins(&wins))
    }

    fn get_root_win(&self) -> Result<xcb::Window, ()> {
        let setup: xcb::Setup = self.x11.conn.get_setup();
        let mut roots: xcb::ScreenIterator = setup.roots();
        let preferred_screen_pos = usize::try_from(self.x11.preferred_screen)
            .expect("x11 preferred_screen is not positive");
        let screen: xcb::Screen = roots.nth(preferred_screen_pos).ok_or(())?;
        Ok(screen.root())
    }

    fn get_all_wins(&self) -> Result<Vec<xcb::Window>, ()> {
        let root_win = self.get_root_win()?;

        let query_tree_reply: xcb::QueryTreeReply =
            xcb::xproto::query_tree(&self.x11.conn, root_win)
                .get_reply()
                .map_err(|_| ())?;

        Ok(query_tree_reply.children().to_vec())
    }

    fn can_break(&self) -> Result<CanBreak, ()> {
        let all_win_props: Vec<WinProps> = self.get_all_win_props()?;
        let all_can_break_preds = CanBreakPreds::all();
        let can_break_bool = all_win_props.iter().all(|win_props| {
            all_can_break_preds.can_break(win_props).into_bool()
        });
        let can_break_res = CanBreak::from_bool(can_break_bool);
        Ok(can_break_res)
    }
}

struct CanBreakPred<F>(F);

impl CanBreakPred<Box<dyn Fn(&WinProps) -> CanBreak>> {
    fn from_name_class<G>(g: G) -> Self
    where
        G: 'static + Fn(&str, &str, &str) -> CanBreak,
    {
        Self(Box::new(move |win_props: &WinProps| {
            match (
                &win_props.net_wm_name,
                &win_props.class_name,
                &win_props.class,
            ) {
                (Ok(net_wm_name), Ok(class_name), Ok(class)) => {
                    g(&net_wm_name, &class_name, &class)
                }
                _ => CanBreak::Yes,
            }
        }))
    }

    fn can_break(&self, win_props: &WinProps) -> CanBreak {
        self.0(win_props)
    }
}

struct CanBreakPreds<F>(Vec<CanBreakPred<F>>);

impl CanBreakPreds<Box<dyn Fn(&WinProps) -> CanBreak>> {
    fn all() -> Self {
        Self(vec![
            // BigBlueButton in browser
            CanBreakPred::from_name_class(
                |net_wm_name: &str,
                 class_name: &str,
                 class: &str|
                 -> CanBreak {
                    browser_title_starts_with(
                        class,
                        class_name,
                        net_wm_name,
                        "BigBlueButton",
                    )
                },
            ),
            // Google Meet in browser
            CanBreakPred::from_name_class(
                |net_wm_name: &str,
                 class_name: &str,
                 class: &str|
                 -> CanBreak {
                    browser_title_starts_with(
                        class,
                        class_name,
                        net_wm_name,
                        "Meet",
                    )
                },
            ),
            // Jitsi in browser
            CanBreakPred::from_name_class(
                |net_wm_name: &str,
                 class_name: &str,
                 class: &str|
                 -> CanBreak {
                    browser_title_contains(
                        class,
                        class_name,
                        net_wm_name,
                        "Jitsi Meet",
                    )
                },
            ),
            // Slack: Initiating a Slack call in browser
            CanBreakPred::from_name_class(
                |net_wm_name: &str,
                 class_name: &str,
                 class: &str|
                 -> CanBreak {
                    browser_title_starts_with(
                        class,
                        class_name,
                        net_wm_name,
                        "Slack | Calling ",
                    )
                },
            ),
            // Slack: In a Slack call in browser
            CanBreakPred::from_name_class(
                |net_wm_name: &str,
                 class_name: &str,
                 class: &str|
                 -> CanBreak {
                    browser_title_starts_with(
                        class,
                        class_name,
                        net_wm_name,
                        "Slack | Slack call ",
                    )
                },
            ),
            // Skype
            CanBreakPred::from_name_class(
                |net_wm_name: &str,
                 class_name: &str,
                 class: &str|
                 -> CanBreak {
                    if class == "Skype"
                        && class_name == "skype"
                        && net_wm_name == "Skype"
                    {
                        CanBreak::No
                    } else {
                        CanBreak::Yes
                    }
                },
            ),
            // Zoom
            CanBreakPred::from_name_class(
                |net_wm_name: &str,
                 class_name: &str,
                 class: &str|
                 -> CanBreak {
                    if class == "zoom"
                        && class_name == "zoom"
                        && net_wm_name == "Zoom"
                    {
                        CanBreak::No
                    } else {
                        CanBreak::Yes
                    }
                },
            ),
        ])
    }

    fn can_break(&self, win_props: &WinProps) -> CanBreak {
        CanBreak::from_bool(self.0.iter().all(|can_break_pred| {
            can_break_pred.can_break(win_props).into_bool()
        }))
    }
}

fn is_browser(class: &str, class_name: &str) -> bool {
    (class == "Chromium-browser" && class_name == "chromium-browser")
        || (class == "Firefox" && class_name == "Navigator")
}

fn browser_title_starts_with_raw(
    class: &str,
    class_name: &str,
    net_wm_name: &str,
    title_starts_with: &str,
) -> bool {
    is_browser(class, class_name) && net_wm_name.starts_with(title_starts_with)
}

fn browser_title_starts_with(
    class: &str,
    class_name: &str,
    net_wm_name: &str,
    title_starts_with: &str,
) -> CanBreak {
    if browser_title_starts_with_raw(
        class,
        class_name,
        net_wm_name,
        title_starts_with,
    ) {
        CanBreak::No
    } else {
        CanBreak::Yes
    }
}

fn browser_title_contains_raw(
    class: &str,
    class_name: &str,
    net_wm_name: &str,
    title_contains: &str,
) -> bool {
    is_browser(class, class_name) && net_wm_name.contains(title_contains)
}

fn browser_title_contains(
    class: &str,
    class_name: &str,
    net_wm_name: &str,
    title_contains: &str,
) -> CanBreak {
    if browser_title_contains_raw(
        class,
        class_name,
        net_wm_name,
        title_contains,
    ) {
        CanBreak::No
    } else {
        CanBreak::Yes
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

#[derive(Clone, Debug)]
struct WinProps {
    #[allow(dead_code)]
    wm_name: Result<String, ()>,
    net_wm_name: Result<String, ()>,
    #[allow(dead_code)]
    transient_for_wins: Result<Vec<xcb::Window>, ()>,
    class_name: Result<String, ()>,
    class: Result<String, ()>,
}

impl WinProps {
    fn get_all(all_win_prop_cookies: Vec<WinPropCookies>) -> Vec<Self> {
        all_win_prop_cookies.into_iter().map(Self::get).collect()
    }

    fn get(win_prop_cookies: WinPropCookies) -> Self {
        let wm_name = win_prop_cookies
            .wm_name
            .get_reply()
            .map_err(|_generic_err| ())
            .and_then(|wm_name_reply| {
                let wm_name_vec = wm_name_reply.value().to_vec();
                String::from_utf8(wm_name_vec).map_err(|_from_utf8_err| ())
            });

        let net_wm_name = win_prop_cookies
            .net_wm_name
            .get_reply()
            .map_err(|_generic_err| ())
            .and_then(|net_wm_name_reply| {
                let net_wm_name_vec = net_wm_name_reply.value().to_vec();
                String::from_utf8(net_wm_name_vec).map_err(|_from_utf8_err| ())
            });

        let transient_for_wins = win_prop_cookies
            .wm_transient_for
            .get_reply()
            .map_err(|_generic_err| ())
            .map(|trans_reply| trans_reply.value().to_vec());

        let ClassInfo {
            name: class_name,
            class,
        } = ClassInfo::from_raw(
            win_prop_cookies
                .wm_class
                .get_reply()
                .map_err(|_generic_error| ()),
            (),
            |_from_utf8_err| (),
        );

        Self {
            wm_name,
            net_wm_name,
            transient_for_wins,
            class_name,
            class,
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

    fn from_raw<F: Fn(std::string::FromUtf8Error) -> T>(
        res_raw: Result<xcb::GetPropertyReply, T>,
        no_index_err: T,
        utf8_err_mapper: F,
    ) -> Self {
        match res_raw {
            Err(t) => Self::err(t),
            Ok(raw) => {
                let all = raw.value::<u8>();
                Self::from_raw_data(all, no_index_err, utf8_err_mapper)
            }
        }
    }
}

impl Plugin for WindowTitles {
    fn can_break_now(&self) -> Result<CanBreak, Box<dyn std::error::Error>> {
        let custom_error = std::io::Error::new(
            std::io::ErrorKind::Other,
            "TODO: change this to an actual error",
        );
        self.can_break()
            .map_err(|()| Box::new(custom_error) as Box<dyn std::error::Error>)
    }

    fn name(&self) -> String {
        String::from("window_titles")
    }
}
