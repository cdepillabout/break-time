use std::default::Default;
use std::fs::File;
use std::path::PathBuf;

use indoc::indoc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(transparent)]
pub struct PluginSettings(toml::value::Table);

// pub trait Deserialize<'de>: Sized {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>;
// }


impl Default for PluginSettings {
    fn default() -> Self {
        let mut google_cal: toml::value::Table = toml::map::Map::new();
        let google_cal_accounts: Vec<String> = vec![];
        let google_cal_accounts_val = toml::Value::try_from(google_cal_accounts).expect("Could not decode google_cal_accounts as toml Value, even though we should be able to.");
        google_cal.insert(String::from("accounts"), google_cal_accounts_val);

        let x11_window_title_checker: toml::value::Table = toml::map::Map::new();

        let mut plugin_settings_table: toml::value::Table =
            toml::map::Map::new();
        plugin_settings_table.insert(
            String::from("google_calendar"),
            toml::Value::Table(google_cal),
        );
        plugin_settings_table.insert(
            String::from("x11_window_title_checker"),
            toml::Value::Table(x11_window_title_checker),
        );

        PluginSettings(plugin_settings_table)
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Settings {
    break_duration_seconds: u32,
    #[serde(rename = "plugin")]
    all_plugin_settings: PluginSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            break_duration_seconds: 600,
            all_plugin_settings: Default::default(),
        }
    }
}

pub struct Config {
    base_dir: xdg::BaseDirectories,
    file_path: PathBuf,
    settings: Settings,
}

const DEFAULT_SETTINGS: &str = indoc!(
    "
    # The number of seconds in a break.
    break_duration_seconds = 600 # 10 minutes

    [plugin.google_calendar]
    # A list of strings, one for each Google account you want to authenticate with.
    accounts = []

    [plugin.x11_window_title_checker]
    "
);

//     "/nix/store/qy93dp4a3rqyn2mz63fbxjg228hffwyw-hello-2.10
//     +---/nix/store/pnd2kl27sag76h23wa5kl95a76n3k9i3-glibc-2.27
//     +---/nix/store/pnd2kl27sag76h23wa5kl95a76n3k9i3-glibc-2.27 [...]
//     +---/nix/store/qy93dp4a3rqyn2mz63fbxjg228hffwyw-hello-2.10 [...]
//     "
// );

impl Config {
    pub fn load() -> Result<Self, ()> {
        let base_dir = xdg::BaseDirectories::with_prefix("break-time")
            .map_err(|xdg_base_dir_err| ())?;

        let file_path = base_dir
            .place_config_file("config.toml")
            .map_err(|io_err| ())?;

        // Try opening the config file to see whether it exists or not.
        let res = File::open(file_path);
        match res {
            Err(_) => todo!(),
            Ok(existing_config_file) => {}
        }

        // let config = Config { base_dir, file_path };

        // Ok(config)

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_plugins() {
        // let plugin_settings_table: toml::value::Table = toml::map::Map::new();
        // let example_settings =
        //     Settings {
        //         all_plugin_settings: plugin_settings_table,
        //     };
        // let serialized = toml::to_string(&example_settings);

        // assert_eq!(serialized, Ok(String::from("hello")));

        // let raw_input = indoc!(
        //     "/nix/store/qy93dp4a3rqyn2mz63fbxjg228hffwyw-hello-2.10
        //     +---/nix/store/pnd2kl27sag76h23wa5kl95a76n3k9i3-glibc-2.27
        //     +---/nix/store/pnd2kl27sag76h23wa5kl95a76n3k9i3-glibc-2.27 [...]
        //     +---/nix/store/qy93dp4a3rqyn2mz63fbxjg228hffwyw-hello-2.10 [...]
        //     "
        // );
    }

    #[test]
    fn test_default_settings_constant_is_same_as_default_impl() {
        let settings_from_default_instance: Settings = Default::default();
        let settings_from_default_const: Settings = toml::from_str(DEFAULT_SETTINGS).unwrap();

        assert_eq!(settings_from_default_instance, settings_from_default_const);
    }
}
