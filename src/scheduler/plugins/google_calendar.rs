use super::{CanBreak, Plugin};

use std::net::TcpListener;

use google_calendar3::{
    CalendarHub, CalendarListEntry, Channel, Error, Events, Scope,
};
use yup_oauth2::{
    ApplicationSecret, Authenticator, AuthenticatorDelegate,
    DefaultAuthenticatorDelegate, DiskTokenStorage, Retry,
};

fn get_available_port() -> Option<u16> {
    (8080..65535).find(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok())
}

type CalHub = CalendarHub<
    hyper::Client,
    Authenticator<
        DefaultAuthenticatorDelegate,
        DiskTokenStorage,
        hyper::Client,
    >,
>;

type Auth = Authenticator<
    DefaultAuthenticatorDelegate,
    DiskTokenStorage,
    hyper::Client,
>;

pub struct GoogleCalendar {
    hub: CalHub,
    calendar_ids: Vec<String>,
}

const GOOGLE_CLIENT_ID: &'static str =
    "728095687622-mpib9rmdtck7e8ln9egelnns6na0me08.apps.googleusercontent.com";

// It is weird embedding something called a "client_secret" directly in the source
// code, but it doesn't seem like this needs to be something that is actually kept
// secret:
// https://stackoverflow.com/questions/59416326/safely-distribute-oauth-2-0-client-secret-in-desktop-applications-in-python
const GOOGLE_CLIENT_SECRET: &'static str = "mI7MmEnboy8jdYEBjK9rZ2M2";

impl GoogleCalendar {
    pub fn new() -> Result<Self, ()> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("break-time")
            .map_err(|xdg_base_dir_err| ())?;
        let google_oauth_token_path = xdg_dirs
            .place_config_file("google-oauth-token")
            .map_err(|io_err| ())?;
        let google_oauth_token_path_string =
            google_oauth_token_path.to_string_lossy().into_owned();
        let disk_token_storage: DiskTokenStorage = DiskTokenStorage::new(
            &google_oauth_token_path_string,
        )
        .expect("Couldn't create a file to hold the google oauth token");

        let hub: CalHub = create_hub(disk_token_storage)?;

        let calendar_ids = get_all_calendar_ids(&hub);

        Ok(GoogleCalendar { hub, calendar_ids })
    }

    fn can_break(&self) -> Result<CanBreak, ()> {
        let now: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
        let ten_minutes_ago: chrono::DateTime<chrono::Utc> =
            now - chrono::Duration::minutes(10);
        let in_twenty_mins: chrono::DateTime<chrono::Utc> =
            now + chrono::Duration::minutes(20);
        // println!("now: {}, after twenty: {}", now.to_rfc3339(), in_twenty_mins.to_rfc3339());

        if has_events(
            &self.hub,
            &self.calendar_ids,
            ten_minutes_ago,
            in_twenty_mins,
        ) {
            Ok(CanBreak::No)
        } else {
            Ok(CanBreak::Yes)
        }
    }
}

fn create_auth(disk_token_storage: DiskTokenStorage) -> Result<Auth, ()> {
    let secret: ApplicationSecret = ApplicationSecret {
        client_id: String::from(GOOGLE_CLIENT_ID),
        client_secret: String::from(GOOGLE_CLIENT_SECRET),
        token_uri: "https://oauth2.googleapis.com/token".to_string(),
        auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
        redirect_uris: vec![
            "http://127.0.0.1".to_string(),
            "urn:ietf:wg:oauth:2.0:oob".to_string(),
        ],
        ..Default::default()
    };

    let http_client_for_auth: hyper::Client = hyper::Client::with_connector(
        hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new()),
    );
    let first_available_port = get_available_port().ok_or(())?;

    let auth: Auth = Authenticator::new(
        &secret,
        DefaultAuthenticatorDelegate,
        http_client_for_auth,
        disk_token_storage,
        Some(yup_oauth2::FlowType::InstalledRedirect(
            first_available_port.into(),
        )),
    );

    Ok(auth)
}

fn create_hub_from_auth(auth: Auth) -> CalHub {
    let http_client_for_cal: hyper::Client = hyper::Client::with_connector(
        hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new()),
    );

    CalendarHub::new(http_client_for_cal, auth)
}

fn create_hub(disk_token_storage: DiskTokenStorage) -> Result<CalHub, ()> {
    let auth: Auth = create_auth(disk_token_storage)?;

    let hub: CalHub = create_hub_from_auth(auth);

    Ok(hub)
}

fn get_all_calendar_ids(hub: &CalHub) -> Vec<String> {
    let (_, calendar_list_res) = hub
        .calendar_list()
        .list()
        .add_scope(Scope::Readonly)
        .add_scope(Scope::EventReadonly)
        .doit()
        .expect("couldn't get a response from calendar_list");

    let calendars = calendar_list_res
        .items
        .expect("There should be some calendars available");

    let calendar_ids: Vec<String> = calendars
        .into_iter()
        .map(|calendar: CalendarListEntry| {
            calendar.id.expect("Calendars should always have ids")
        })
        .collect();

    calendar_ids
}

fn has_events(
    hub: &CalHub,
    calendar_ids: &[String],
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
) -> bool {
    calendar_ids
        .iter()
        .any(|calendar_id| has_event(hub, calendar_id, start_time, end_time))
}

fn has_event(
    hub: &CalHub,
    calendar_id: &str,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
) -> bool {
    let result: google_calendar3::Result<(_, Events)> = hub
        .events()
        .list(calendar_id)
        .add_scope(Scope::Readonly)
        .add_scope(Scope::EventReadonly)
        // all events the occur over the next 20 minutes
        .time_min(&start_time.to_rfc3339())
        .time_max(&end_time.to_rfc3339())
        .doit();

    // println!("\n\nevents for {}: {:?}", calendar_id, result);

    match result {
        Err(_err) => {
            // TODO: Maybe I should warn about these errors?
            false
        }
        Ok((_, events)) => match events.items {
            None => false,
            Some(event_items) => {
                if event_items.len() >= 1 {
                    println!("There were some event items! {:?}", event_items);
                    true
                } else {
                    false
                }
            }
        },
    }
}

impl Plugin for GoogleCalendar {
    fn can_break_now(&self) -> Result<CanBreak, ()> {
        self.can_break()
    }
}
