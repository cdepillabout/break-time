
struct Config;


impl Config {
    pub fn load() -> Result<Self, ()> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("break-time")
            .map_err(|xdg_base_dir_err| ())?;
        let google_oauth_token_path = xdg_dirs
            .place_config_file("google-oauth-token")
            .map_err(|io_err| ())?;
        let google_oauth_token_path_string =
            google_oauth_token_path.to_string_lossy().into_owned();

        Ok(Config)
    }
}
