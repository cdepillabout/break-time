use std::default::Default;
use std::fs::File;
use std::path::PathBuf;

use indoc::indoc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(transparent)]
pub struct PluginSettings(toml::value::Table);

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


impl Config {
    // TODO: Change some of the panics in this function to returning errors.
    pub fn load() -> Result<Self, ()> {
        let base_dir = xdg::BaseDirectories::with_prefix("break-time")
            .map_err(|xdg_base_dir_err| ())?;

        let config_file_path = base_dir
            .place_config_file("config.toml")
            .map_err(|io_err| ())?;

        // Try reading the config file to see whether it exists or not.
        let res_config_file = std::fs::read_to_string(&config_file_path);
        let settings = match res_config_file {
            // TODO: I should probably check here the reason we are getting an
            // error.  If there is a bad permission on the file, then I
            // should probably just error out fast instead of trying to
            // create a new config file.
            Err(_) => {
                // If we couldn't read the config file, then create a new one
                // from the default.
                let write_res = std::fs::write(&config_file_path, DEFAULT_SETTINGS);
                match write_res {
                    Ok(()) => (),
                    Err(err) => panic!("Couldn't write a new config file at {:?} because of the following error: {}", config_file_path, err),
                }
                Settings::default()
            }
            Ok(config_file) => {
                let res_settings = toml::from_str(&config_file);
                match res_settings {
                    Err(err) => {
                        panic!("Can't parse config file at {:?} because of the following error: {}", config_file_path, err)
                    }
                    Ok(settings) => settings,
                }
            }
        };

        let config = Config { base_dir, file_path: config_file_path, settings };

        Ok(config)
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
