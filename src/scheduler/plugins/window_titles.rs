use super::{CanBreak, Plugin};

use crate::config::Config;

use crate::x11::X11;

pub struct WindowTitles {
    x11: X11,
    net_wm_name_atom: xcb::Atom,
    utf8_string_atom: xcb::Atom,
}

impl WindowTitles {
    pub fn new(config: &Config) -> Result<Self, ()> {
        let x11 = X11::connect();

        let net_wm_name_atom = x11.create_atom("_NET_WM_NAME").ok_or(())?;
        let utf8_string_atom = x11.create_atom("UTF8_STRING").ok_or(())?;

        Ok(WindowTitles {
            x11,
            net_wm_name_atom,
            utf8_string_atom,
        })
    }

    fn request_win_props<'a>(&'a self, win: xcb::Window) -> WinPropCookies<'a> {
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
        let screen: xcb::Screen =
            roots.nth(self.x11.preferred_screen as usize).ok_or(())?;
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

    fn can_break_win_prop(&self, win_props: &WinProps) -> CanBreak {
        dbg!(win_props);
        if let Ok(class) = &win_props.class {
            if let Ok(class_name) = &win_props.class_name {
                if let Ok(net_wm_name) = &win_props.net_wm_name {
                    if class == "Firefox" && class_name == "Navigator" {
                        if net_wm_name.starts_with("Meet") {
                            return CanBreak::No;
                        }
                    }
                }
            }
        }

        CanBreak::Yes
    }

    fn can_break(&self) -> Result<CanBreak, ()> {
        let all_win_props: Vec<WinProps> = self.get_all_win_props()?;
        let can_break_bool = all_win_props
            .iter()
            .all(|win_props| self.can_break_win_prop(win_props).into_bool());
        let can_break_res = CanBreak::from_bool(can_break_bool);
        Ok(can_break_res)
    }
}

struct CanBreakPred(Box<dyn Fn(&WinProps) -> CanBreak>);

// impl<F> CanBreakPred<F>
// where
//     F: Fn(&WinProps) -> CanBreak,
// {
//     // pub fn new(f: F) -> CanBreakPred<impl Fn(&WinProps) -> CanBreak> {
//     //     Self(f)
//     // }

    fn from_name_class<G>(g: G) -> CanBreakPred
    where
        G: 'static + Fn(&str, &str, &str) -> CanBreak,
    {
        CanBreakPred(Box::new(move |win_props: &WinProps| {
            match (&win_props.net_wm_name, &win_props.class_name, &win_props.class)
            {
                (Ok(net_wm_name), Ok(class_name), Ok(class)) => {
                    g(&net_wm_name, &class_name, &class)
                }
                _ => CanBreak::Yes,
            }
        }))
    }
// }

struct CanBreakPreds(Vec<CanBreakPred>);

fn all() -> CanBreakPreds {
    let fff: CanBreakPred = from_name_class(
        |net_wm_name: &str, class_name: &str, class: &str| -> CanBreak {
            if class == "Firefox" && class_name == "Navigator" {
                if net_wm_name.starts_with("Meet") {
                    return CanBreak::No;
                }
            }
            CanBreak::Yes
        }
    );
    CanBreakPreds(vec![fff])
}

// struct CanBreakPred<F>(F);

// impl<F> CanBreakPred<F>
// where
//     F: Fn(&WinProps) -> CanBreak,
// {
//     // pub fn new(f: F) -> CanBreakPred<impl Fn(&WinProps) -> CanBreak> {
//     //     Self(f)
//     // }

//     pub fn from_name_class<G>(g: G) -> CanBreakPred<Box<dyn Fn(&WinProps) -> CanBreak>>
//     where
//         G: 'static + Fn(&str, &str, &str) -> CanBreak,
//     {
//         Self(Box::new(move |win_props: &WinProps| {
//             match (&win_props.net_wm_name, &win_props.class_name, &win_props.class)
//             {
//                 (Ok(net_wm_name), Ok(class_name), Ok(class)) => {
//                     g(&net_wm_name, &class_name, &class)
//                 }
//                 _ => CanBreak::Yes,
//             }
//         }))
//     }
// }

// struct CanBreakPreds<F>(Vec<CanBreakPred<F>>);

// pub fn all() -> CanBreakPreds<Box<dyn Fn(&WinProps) -> CanBreak>> {
//     let fff: CanBreakPred<Box<dyn Fn(&WinProps) -> CanBreak>> = CanBreakPred::from_name_class(
//         |net_wm_name: &str, class_name: &str, class: &str| -> CanBreak {
//             if class == "Firefox" && class_name == "Navigator" {
//                 if net_wm_name.starts_with("Meet") {
//                     return CanBreak::No;
//                 }
//             }
//             CanBreak::Yes
//         }
//     );
//     CanBreakPreds(vec![fff])
// }

const PROP_STARTING_OFFSET: u32 = 0;
const PROP_LENGTH_TO_GET: u32 = 2048;

struct WinPropCookies<'a> {
    wm_name: xcb::xproto::GetPropertyCookie<'a>,
    net_wm_name: xcb::xproto::GetPropertyCookie<'a>,
    wm_class: xcb::xproto::GetPropertyCookie<'a>,
    wm_transient_for: xcb::xproto::GetPropertyCookie<'a>,
}

#[derive(Debug)]
struct WinProps {
    wm_name: Result<String, ()>,
    net_wm_name: Result<String, ()>,
    transient_for_wins: Result<Vec<xcb::Window>, ()>,
    class_name: Result<String, ()>,
    class: Result<String, ()>,
}

impl WinProps {
    fn get_all(all_win_prop_cookies: Vec<WinPropCookies>) -> Vec<Self> {
        all_win_prop_cookies
            .into_iter()
            .map(|win_prop_cookies| WinProps::get(win_prop_cookies))
            .collect()
    }

    fn get(win_prop_cookies: WinPropCookies) -> Self {
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

        let ClassInfo {
            name: class_name,
            class,
        } = ClassInfo::from_raw(
            win_prop_cookies
                .wm_class
                .get_reply()
                .map_err(|generic_error| ()),
            (),
            |from_utf8_err| (),
        );

        WinProps {
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
    fn err(t: T) -> ClassInfo<T> {
        ClassInfo {
            name: Err(t.clone()),
            class: Err(t),
        }
    }

    fn from_raw_data_with_index<F: Fn(std::string::FromUtf8Error) -> T>(
        raw: &[u8],
        index: usize,
        utf8_err_mapper: F,
    ) -> ClassInfo<T> {
        ClassInfo {
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
    ) -> ClassInfo<T> {
        let option_index = raw.iter().position(|&b| b == 0);
        match option_index {
            None => ClassInfo::err(no_index_err),
            Some(index) => {
                ClassInfo::from_raw_data_with_index(raw, index, utf8_err_mapper)
            }
        }
    }

    fn from_raw<F: Fn(std::string::FromUtf8Error) -> T>(
        res_raw: Result<xcb::GetPropertyReply, T>,
        no_index_err: T,
        utf8_err_mapper: F,
    ) -> ClassInfo<T> {
        match res_raw {
            Err(t) => ClassInfo::err(t),
            Ok(raw) => {
                let all = raw.value::<u8>();
                ClassInfo::from_raw_data(all, no_index_err, utf8_err_mapper)
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
}
