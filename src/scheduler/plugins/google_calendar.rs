use super::{CanBreak, Plugin};

use crate::config::{Config, PluginSettings};

use std::collections::HashMap;
use std::net::TcpListener;
use std::path::Path;

use google_calendar3::{CalendarHub, CalendarListEntry, Events, Scope};
use yup_oauth2::{
    ApplicationSecret, Authenticator, DefaultAuthenticatorDelegate,
    DiskTokenStorage,
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

pub struct CalFetcher {
    email: String,
    hub: CalHub,
    calendar_ids: Vec<String>,
}

impl CalFetcher {
    pub fn new(break_time_cache_dir: &Path, email: String) -> Result<Self, ()> {
        let google_cal_dir_name = Path::new("google-calendar");
        let google_cal_dir_path =
            break_time_cache_dir.join(google_cal_dir_name);

        std::fs::create_dir_all(&google_cal_dir_path).map_err(|_io_err| ())?;

        let token_path = google_cal_dir_path.join(&email);

        let token_path_string = token_path.to_string_lossy().into_owned();
        let disk_token_storage: DiskTokenStorage = DiskTokenStorage::new(
            &token_path_string,
        )
        .expect("Couldn't create a file to hold the google oauth token");

        let hub: CalHub = create_hub(disk_token_storage)?;

        let calendar_ids = get_all_calendar_ids(&hub);

        Ok(Self {
            email,
            hub,
            calendar_ids,
        })
    }

    pub fn can_break(&self) -> Result<CanBreak, GoogleCalErr> {
        let now: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
        let ten_minutes_ago: chrono::DateTime<chrono::Utc> =
            now - chrono::Duration::minutes(10);
        let in_twenty_mins: chrono::DateTime<chrono::Utc> =
            now + chrono::Duration::minutes(20);

        let res = has_events(
            &self.email,
            &self.hub,
            &self.calendar_ids,
            ten_minutes_ago,
            in_twenty_mins,
        );

        match res {
            Err(err) => Err(err),
            Ok(HasEvent::Yes) => Ok(CanBreak::No),
            Ok(HasEvent::No) => Ok(CanBreak::Yes),
        }
    }
}

const GOOGLE_CLIENT_ID: &str =
    "728095687622-mpib9rmdtck7e8ln9egelnns6na0me08.apps.googleusercontent.com";

// It is weird embedding something called a "client_secret" directly in the source
// code, but it doesn't seem like this needs to be something that is actually kept
// secret:
// https://stackoverflow.com/questions/59416326/safely-distribute-oauth-2-0-client-secret-in-desktop-applications-in-python
const GOOGLE_CLIENT_SECRET: &str = "mI7MmEnboy8jdYEBjK9rZ2M2";

// TODO: Create a datatype to hold all the settings for the GoogleCalendar plugin.
// Don't try parsing it out manually here.
fn get_emails(plugin_settings: &PluginSettings) -> Result<Vec<String>, ()> {
    let google_cal_settings: &toml::Value =
        match plugin_settings.0.get("google_calendar") {
            // If the "google_calendar" key doesn't exist, then just skip.
            None => return Ok(vec![]),
            Some(val) => val,
        };
    let google_cal_settings_table: &toml::value::Table =
        google_cal_settings.as_table().ok_or(
            // If the "google_calendar" key exists, but it doesn't contain a table, then throw an
            // error.
            (),
        )?;
    let all_accounts: &toml::Value =
        match google_cal_settings_table.get("accounts") {
            // If the "google_calendar" key exists, but it doesn't have an accounts field, then
            // just skip.
            None => return Ok(vec![]),
            Some(all_accounts) => all_accounts,
        };

    let all_emails = all_accounts.clone().try_into().map_err(|_err| ());

    // println!("All emails: {:?}", all_emails);

    all_emails
}

fn collect_first_err<T, E>(v: Vec<Result<T, E>>) -> Result<Vec<T>, E> {
    let mut ok_vec = vec![];

    for res in v {
        match res {
            Err(err) => return Err(err),
            Ok(i) => ok_vec.push(i),
        }
    }

    Ok(ok_vec)
}

pub struct GoogleCalendar {
    fetchers: Vec<CalFetcher>,
}

impl GoogleCalendar {
    pub fn new(config: &Config) -> Result<Self, ()> {
        let break_time_cache_dir: &Path = &config.cache_dir;
        let emails = get_emails(&config.settings.all_plugin_settings)?;

        let fetchers_res = emails
            .into_iter()
            .map(|email| CalFetcher::new(break_time_cache_dir, email))
            .collect();

        let fetchers = collect_first_err(fetchers_res)?;

        Ok(Self { fetchers })
    }

    fn can_break(&self) -> Result<CanBreak, GoogleCalErr> {
        // println!("now: {}, after twenty: {}", now.to_rfc3339(), in_twenty_mins.to_rfc3339());

        self.fetchers.iter().map(CalFetcher::can_break).fold(
            Ok(CanBreak::Yes),
            |accum, can_break_res| match (accum, can_break_res) {
                (Err(err), _) => Err(err),
                (_, Err(err)) => Err(err),
                (Ok(CanBreak::No), _) => Ok(CanBreak::No),
                (_, can_break) => can_break,
            },
        )
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
        ..ApplicationSecret::default()
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
        .add_scope(Scope::Event)
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

/// Check whether or not any events occur during the `start_time` to `end_time`.
fn has_events(
    email: &str,
    hub: &CalHub,
    calendar_ids: &[String],
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
) -> Result<HasEvent, GoogleCalErr> {
    let all_has_events: Vec<Result<HasEvent, GoogleCalErr>> = calendar_ids
        .iter()
        .map(|calendar_id| {
            // We only check calendar_ids that are equal to the email address we are looking for.
            //
            // TODO: Eventually, we probably want to let the user configure what email addresses
            // they want to look for events on.
            if email == calendar_id {
                has_event(hub, calendar_id, start_time, end_time)
            } else {
                Ok(HasEvent::No)
            }
        })
        .collect();

    all_has_events
        .into_iter()
        .fold(Ok(HasEvent::No), |accum, res| match (accum, res) {
            (Err(err), _) => Err(err),
            (_, Err(err)) => Err(err),
            (Ok(HasEvent::No), new) => new,
            (Ok(HasEvent::Yes), _) => Ok(HasEvent::Yes),
        })
}

enum HasEvent {
    No,
    Yes,
}

#[derive(Debug)]
pub enum GoogleCalErr {
    FetchingEvents {
        calendar_id: String,
        google_cal_err: google_calendar3::Error,
    },
}

impl std::error::Error for GoogleCalErr {}

impl std::fmt::Display for GoogleCalErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FetchingEvents {
                calendar_id,
                google_cal_err,
            } => write!(
                f,
                "Google Calendard Plugin: Error fetching calendar_id {}: {}",
                calendar_id, google_cal_err
            ),
        }
    }
}

fn get_event(
    hub: &CalHub,
    calendar_id: &str,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
    // This is super hacky...
    log: bool,
) -> Result<Vec<google_calendar3::Event>, GoogleCalErr> {
    let result: google_calendar3::Result<(_, Events)> = hub
        .events()
        .list(calendar_id)
        .add_scope(Scope::Readonly)
        .add_scope(Scope::Event)
        // all events that occur over the next 20 minutes
        .time_min(&start_time.to_rfc3339())
        .time_max(&end_time.to_rfc3339())
        // Expand recurring events into single events.
        .single_events(true)
        .doit();

    // println!("\n\nevents for {}: {:?}", calendar_id, result);

    match result {
        Err(err) => Err(GoogleCalErr::FetchingEvents {
            calendar_id: String::from(calendar_id),
            google_cal_err: err,
        }),
        Ok((_, events)) => match events.items {
            None => Ok(vec![]),
            Some(event_items) => {
                let filtered_events: Vec<google_calendar3::Event> =
                    filter_cal_events(event_items);
                if filtered_events.is_empty() {
                    Ok(filtered_events)
                } else {
                    if log {
                        println!("There were some event items from calendar id {}: {:?}", calendar_id, filtered_events);
                    }
                    Ok(filtered_events)
                }
            }
        },
    }
}

fn has_event(
    hub: &CalHub,
    calendar_id: &str,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
) -> Result<HasEvent, GoogleCalErr> {
    let event_res = get_event(hub, calendar_id, start_time, end_time, true);
    event_res.map(|filtered_events| {
        if filtered_events.is_empty() {
            HasEvent::No
        } else {
            HasEvent::Yes
        }
    })
}

fn filter_cal_events(
    events: Vec<google_calendar3::Event>,
) -> Vec<google_calendar3::Event> {
    events.into_iter().filter(filter_event).collect()
}

fn filter_event(event: &google_calendar3::Event) -> bool {
    if let Some(desc) = &event.description {
        // Ignore events where the description contains the magic string
        // "ignore break-time"
        if desc.to_lowercase().contains("ignore break-time") {
            return false;
        }

        // Ignore events where the description talks about being an out-of-office event.
        // Even if we are out-of-office, we still may be on our personal computer, and
        // want break-time to occassionally break.
        if desc.to_lowercase().contains("out-of-office event") {
            return false;
        }
    }

    // Ignore events where the `ignore-break-time` extended property is set.
    if let Some(extended_props) = &event.extended_properties {
        if let Some(props) = &extended_props.private {
            if let Some(ignore_break_time) = props.get("ignore-break-time") {
                if ignore_break_time == "true" {
                    return false;
                }
            }
        }
    }

    // Ignore events where the status is "cancelled".
    //
    // For some reason, sometimes Google Calendar will not set the event status as "cancelled" even
    // though you have cancelled the event.  It keeps the event status as "needsAction".  Check for
    // "needsAction" and ignore events with this status as well.
    if let Some(status) = &event.status {
        if status == "cancelled" || status == "needsAction" {
            return false;
        }
    }

    // Ignore events where there are attendees, and you are marked as not attending.
    if let Some(attendees) = &event.attendees {
        if let Some(me) = attendees
            .iter()
            .find(|attendee| attendee.self_ == Some(true))
        {
            if me.response_status == Some(String::from("declined")) {
                return false;
            }
        }
    }

    true
}

impl Plugin for GoogleCalendar {
    fn can_break_now(&self) -> Result<CanBreak, Box<dyn std::error::Error>> {
        self.can_break().map_err(|google_cal_err| {
            Box::new(google_cal_err) as Box<dyn std::error::Error>
        })
    }

    fn name(&self) -> String {
        String::from("google_calendar")
    }
}

pub fn list_events(config: Config) {
    let google_calendar = GoogleCalendar::new(&config)
        .expect("Could not initialize Google Calendar.");

    let event_calendar_lists: Vec<_> = google_calendar
        .fetchers
        .iter()
        .flat_map(get_events)
        .collect();

    for (email, res_event_list) in event_calendar_lists {
        println!("{}:", email);
        match res_event_list {
            Err(err) => {
                println!("ERROR with Google Calendar: {}", err);
            }
            Ok(event_list) => {
                for event in event_list {
                    println!(
                        "    - id: {:?}, summary: {:?}",
                        event.id, event.summary
                    );
                }
            }
        }
    }
}

fn get_events(
    cal_fetcher: &CalFetcher,
) -> Vec<(String, Result<Vec<google_calendar3::Event>, GoogleCalErr>)> {
    let now: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
    let ten_minutes_ago: chrono::DateTime<chrono::Utc> =
        now - chrono::Duration::minutes(10);
    let in_twenty_mins: chrono::DateTime<chrono::Utc> =
        now + chrono::Duration::minutes(20);
    let start_time = ten_minutes_ago;
    let end_time = in_twenty_mins;

    let events_list: Vec<(
        String,
        Result<Vec<google_calendar3::Event>, GoogleCalErr>,
    )> = cal_fetcher
        .calendar_ids
        .iter()
        .filter_map(|calendar_id| {
            // We only check calendar_ids that are equal to the email address we are looking for.
            //
            // TODO: Eventually, we probably want to let the user configure what email addresses
            // they want to look for events on.
            if &cal_fetcher.email == calendar_id {
                Some((
                    String::from(calendar_id),
                    get_event(
                        &cal_fetcher.hub,
                        calendar_id,
                        start_time,
                        end_time,
                        false,
                    ),
                ))
            } else {
                None
            }
        })
        .collect();

    events_list
}

pub fn ignore_event(config: Config, event_id: String) {
    let google_calendar = GoogleCalendar::new(&config)
        .expect("Could not initialize Google Calendar.");

    // let event_calendar_lists: Vec<_> = google_calendar.fetchers.iter().flat_map(get_events).collect();
    for fetcher in google_calendar.fetchers {
        for calendar_id in fetcher.calendar_ids {
            // We only check calendar_ids that are equal to the email address we are looking for.
            //
            // TODO: Eventually, we probably want to let the user configure what email addresses
            // they want to look for events on.
            if calendar_id == fetcher.email {
                let mut props = HashMap::new();
                props.insert(
                    "ignore-break-time".to_string(),
                    "true".to_string(),
                );
                let extended_props =
                    google_calendar3::EventExtendedProperties {
                        private: Some(props),
                        ..google_calendar3::EventExtendedProperties::default()
                    };
                let req = google_calendar3::Event {
                    extended_properties: Some(extended_props),
                    ..google_calendar3::Event::default()
                };
                let _res = fetcher
                    .hub
                    .events()
                    .patch(req, &calendar_id, &event_id)
                    .add_scope(Scope::Readonly)
                    .add_scope(Scope::Event)
                    .doit();
                // println!("event_id: {}, calendar_id: {}, res: {:?}", event_id, calendar_id, res);
            }
        }
    }
}
