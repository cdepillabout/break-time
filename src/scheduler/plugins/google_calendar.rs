use super::{CanBreak, Plugin};

use crate::config::{Config, PluginSettings};

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use google_calendar3::api::{Event, EventExtendedProperties, Scope};
use google_calendar3::{hyper_rustls, hyper_util, CalendarHub};
use yup_oauth2::{
    ApplicationSecret, InstalledFlowAuthenticator, InstalledFlowReturnMethod,
};

// google-calendar3 7.0 is async (the network calls are `Future`s driven by a
// `tokio` runtime), but break-time's scheduler and the `Plugin` trait are
// synchronous.  We bridge the two by giving the plugin its own `tokio` runtime
// and calling `runtime.block_on(...)` from the synchronous methods — the async
// "island" stays confined to this file.  `block_on` runs a future to completion
// on the calling thread and returns its value, exactly like the old synchronous
// `.doit()` did.  This is only safe because the scheduler calls us from a plain
// `std::thread` (never from inside a tokio worker, where `block_on` panics).
type HttpsConnector = hyper_rustls::HttpsConnector<
    hyper_util::client::legacy::connect::HttpConnector,
>;
type CalHub = CalendarHub<HttpsConnector>;

pub struct CalFetcher {
    email: String,
    hub: CalHub,
    calendar_ids: Vec<String>,
}

impl CalFetcher {
    async fn new(
        break_time_cache_dir: &Path,
        email: String,
    ) -> Result<Self, InitError> {
        let google_cal_dir_name = Path::new("google-calendar");
        let google_cal_dir_path =
            break_time_cache_dir.join(google_cal_dir_name);

        std::fs::create_dir_all(&google_cal_dir_path).map_err(|source| {
            InitError::CreateCacheDir {
                path: google_cal_dir_path.clone(),
                source,
            }
        })?;

        let token_path = google_cal_dir_path.join(&email);

        println!("Trying to set up Google Calendar OAuth for {email}.");

        let hub: CalHub = create_hub(&token_path).await?;

        let calendar_ids = get_all_calendar_ids(&email, &hub).await?;

        Ok(Self {
            email,
            hub,
            calendar_ids,
        })
    }

    async fn can_break(&self) -> Result<CanBreak, GoogleCalErr> {
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
        )
        .await;

        match res {
            Err(err) => Err(err),
            Ok(HasEvent::Yes) => Ok(CanBreak::No),
            Ok(HasEvent::No) => Ok(CanBreak::Yes),
        }
    }
}

#[derive(Debug)]
pub enum InitError {
    InvalidSettings {
        message: String,
    },
    CreateRuntime {
        source: std::io::Error,
    },
    CreateCacheDir {
        path: PathBuf,
        source: std::io::Error,
    },
    CreateAuthenticator {
        token_path: PathBuf,
        source: std::io::Error,
    },
    CreateHttpsConnector {
        source: std::io::Error,
    },
    FetchCalendarList {
        email: String,
        source: Box<google_calendar3::Error>,
    },
    CalendarListMissingItems {
        email: String,
    },
    CalendarListEntryMissingId {
        email: String,
    },
}

impl std::error::Error for InitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CreateRuntime { source }
            | Self::CreateCacheDir { source, .. }
            | Self::CreateAuthenticator { source, .. }
            | Self::CreateHttpsConnector { source } => Some(source),
            Self::FetchCalendarList { source, .. } => Some(source.as_ref()),
            Self::InvalidSettings { .. }
            | Self::CalendarListMissingItems { .. }
            | Self::CalendarListEntryMissingId { .. } => None,
        }
    }
}

impl std::fmt::Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSettings { message } => {
                write!(f, "Google Calendar Plugin: invalid config: {message}")
            }
            Self::CreateRuntime { source } => write!(
                f,
                "Google Calendar Plugin: could not create async runtime: {source}"
            ),
            Self::CreateCacheDir { path, source } => write!(
                f,
                "Google Calendar Plugin: could not create cache directory \
                 {path:?}: {source}"
            ),
            Self::CreateAuthenticator { token_path, source } => {
                write!(
                    f,
                    "Google Calendar Plugin: could not initialize OAuth token \
                     storage at {token_path:?}: {source}"
                )?;
                if source.kind() == std::io::ErrorKind::InvalidData {
                    write!(
                        f,
                        ". This can happen when the token file was written by \
                         an older break-time/yup-oauth2 version; move it aside \
                         or delete it and restart break-time to authenticate \
                         again."
                    )?;
                }
                Ok(())
            }
            Self::CreateHttpsConnector { source } => write!(
                f,
                "Google Calendar Plugin: could not load native TLS roots: \
                 {source}"
            ),
            Self::FetchCalendarList { email, source } => write!(
                f,
                "Google Calendar Plugin: could not fetch the calendar list for \
                 {email}: {source}"
            ),
            Self::CalendarListMissingItems { email } => write!(
                f,
                "Google Calendar Plugin: Google returned no calendar-list \
                 items for {email}"
            ),
            Self::CalendarListEntryMissingId { email } => write!(
                f,
                "Google Calendar Plugin: Google returned a calendar-list entry \
                 without an id for {email}"
            ),
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
#[allow(clippy::let_and_return)]
fn get_emails(
    plugin_settings: &PluginSettings,
) -> Result<Vec<String>, InitError> {
    let google_cal_settings: &toml::Value =
        match plugin_settings.0.get("google_calendar") {
            // If the "google_calendar" key doesn't exist, then just skip.
            None => return Ok(vec![]),
            Some(val) => val,
        };
    let google_cal_settings_table: &toml::value::Table =
        google_cal_settings.as_table().ok_or_else(|| {
            // If the "google_calendar" key exists, but it doesn't contain a table, then throw an
            // error.
            InitError::InvalidSettings {
                message: String::from(
                    "plugin.google_calendar must be a TOML table",
                ),
            }
        })?;
    let all_accounts: &toml::Value =
        match google_cal_settings_table.get("accounts") {
            // If the "google_calendar" key exists, but it doesn't have an accounts field, then
            // just skip.
            None => return Ok(vec![]),
            Some(all_accounts) => all_accounts,
        };

    let all_emails = all_accounts.clone().try_into().map_err(|err| {
        InitError::InvalidSettings {
            message: format!(
                "plugin.google_calendar.accounts must be a list of strings: \
                 {err}"
            ),
        }
    });

    // println!("All emails: {:?}", all_emails);

    all_emails
}

pub struct GoogleCalendar {
    runtime: tokio::runtime::Runtime,
    fetchers: Vec<CalFetcher>,
}

impl GoogleCalendar {
    pub fn new(config: &Config) -> Result<Self, InitError> {
        let break_time_cache_dir = config.cache_dir.clone();
        let emails = get_emails(&config.settings.all_plugin_settings)?;

        let runtime = tokio::runtime::Runtime::new()
            .map_err(|source| InitError::CreateRuntime { source })?;

        // Building each fetcher requires `await`ing the OAuth flow and an
        // initial calendar-list fetch, so it has to happen inside the runtime.
        let fetchers = runtime.block_on(async {
            let mut fetchers = Vec::with_capacity(emails.len());
            for email in emails {
                fetchers
                    .push(CalFetcher::new(&break_time_cache_dir, email).await?);
            }
            Ok::<Vec<CalFetcher>, InitError>(fetchers)
        })?;

        Ok(Self { runtime, fetchers })
    }

    fn can_break(&self) -> Result<CanBreak, GoogleCalErr> {
        self.runtime.block_on(async {
            let mut accum = CanBreak::Yes;
            for fetcher in &self.fetchers {
                match fetcher.can_break().await? {
                    CanBreak::No => accum = CanBreak::No,
                    CanBreak::Yes => {}
                }
            }
            Ok(accum)
        })
    }
}

fn application_secret() -> ApplicationSecret {
    ApplicationSecret {
        client_id: String::from(GOOGLE_CLIENT_ID),
        client_secret: String::from(GOOGLE_CLIENT_SECRET),
        token_uri: "https://oauth2.googleapis.com/token".to_string(),
        auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
        redirect_uris: vec![
            "http://127.0.0.1".to_string(),
            "urn:ietf:wg:oauth:2.0:oob".to_string(),
        ],
        ..ApplicationSecret::default()
    }
}

async fn create_hub(token_path: &Path) -> Result<CalHub, InitError> {
    let auth = InstalledFlowAuthenticator::builder(
        application_secret(),
        InstalledFlowReturnMethod::HTTPRedirect,
    )
    .persist_tokens_to_disk(token_path)
    .build()
    .await
    .map_err(|source| InitError::CreateAuthenticator {
        token_path: token_path.to_path_buf(),
        source,
    })?;

    // Pass the `ring` crypto provider explicitly rather than going through
    // `with_native_roots()`, which relies on a process-wide default rustls
    // provider being installed and panics at connect time if none is.  This
    // matches how yup-oauth2 builds its own client internally.
    let connector =
        hyper_rustls::HttpsConnectorBuilder::new()
            .with_provider_and_native_roots(
                rustls::crypto::ring::default_provider(),
            )
            .map_err(|source| InitError::CreateHttpsConnector { source })?
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();
    let client = hyper_util::client::legacy::Client::builder(
        hyper_util::rt::TokioExecutor::new(),
    )
    .build(connector);

    Ok(CalendarHub::new(client, auth))
}

async fn get_all_calendar_ids(
    email: &str,
    hub: &CalHub,
) -> Result<Vec<String>, InitError> {
    let (_, calendar_list) = hub
        .calendar_list()
        .list()
        .add_scope(Scope::Readonly)
        .add_scope(Scope::Event)
        .doit()
        .await
        .map_err(|source| InitError::FetchCalendarList {
            email: email.to_string(),
            source: Box::new(source),
        })?;

    let calendars = calendar_list.items.ok_or_else(|| {
        InitError::CalendarListMissingItems {
            email: email.to_string(),
        }
    })?;

    calendars
        .into_iter()
        .map(|calendar| {
            calendar
                .id
                .ok_or_else(|| InitError::CalendarListEntryMissingId {
                    email: email.to_string(),
                })
        })
        .collect()
}

/// Check whether or not any events occur during the `start_time` to `end_time`.
async fn has_events(
    email: &str,
    hub: &CalHub,
    calendar_ids: &[String],
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
) -> Result<HasEvent, GoogleCalErr> {
    for calendar_id in calendar_ids {
        // We only check calendar_ids that are equal to the email address we are looking for.
        //
        // TODO: Eventually, we probably want to let the user configure what email addresses
        // they want to look for events on.
        if email == calendar_id
            && matches!(
                has_event(hub, calendar_id, start_time, end_time).await?,
                HasEvent::Yes
            )
        {
            return Ok(HasEvent::Yes);
        }
    }

    Ok(HasEvent::No)
}

enum HasEvent {
    No,
    Yes,
}

#[derive(Debug)]
pub enum GoogleCalErr {
    FetchingEvents {
        calendar_id: String,
        // Boxed because `google_calendar3::Error` is large (~150 bytes), which
        // would otherwise make every `Result<_, GoogleCalErr>` bloated
        // (clippy::result_large_err).
        google_cal_err: Box<google_calendar3::Error>,
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
                "Google Calendar Plugin: Error fetching calendar_id {calendar_id}: {google_cal_err}"
            ),
        }
    }
}

async fn get_event(
    hub: &CalHub,
    calendar_id: &str,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
    // This is super hacky...
    log: bool,
) -> Result<Vec<Event>, GoogleCalErr> {
    let result = hub
        .events()
        .list(calendar_id)
        .add_scope(Scope::Readonly)
        .add_scope(Scope::Event)
        // all events that occur over the next 20 minutes
        .time_min(start_time)
        .time_max(end_time)
        // Expand recurring events into single events.
        .single_events(true)
        .doit()
        .await;

    // println!("\n\nevents for {}: {:?}", calendar_id, result);

    match result {
        Err(err) => Err(GoogleCalErr::FetchingEvents {
            calendar_id: String::from(calendar_id),
            google_cal_err: Box::new(err),
        }),
        Ok((_, events)) => match events.items {
            None => Ok(vec![]),
            Some(event_items) => {
                let filtered_events = filter_cal_events(event_items);
                if !filtered_events.is_empty() && log {
                    println!("There were some event items from calendar id {calendar_id}: {filtered_events:?}");
                }
                Ok(filtered_events)
            }
        },
    }
}

async fn has_event(
    hub: &CalHub,
    calendar_id: &str,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
) -> Result<HasEvent, GoogleCalErr> {
    let event_res =
        get_event(hub, calendar_id, start_time, end_time, true).await;
    event_res.map(|filtered_events| {
        if filtered_events.is_empty() {
            HasEvent::No
        } else {
            HasEvent::Yes
        }
    })
}

fn filter_cal_events(events: Vec<Event>) -> Vec<Event> {
    events.into_iter().filter(filter_event).collect()
}

fn filter_event(event: &Event) -> bool {
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

    // Ignore events where the summary (title) is "Out of office".
    // Google Calendar creates these automatically for OOO events.
    if let Some(summary) = &event.summary {
        if summary.to_lowercase() == "out of office" {
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
    fn name(&self) -> &'static str {
        "google_calendar"
    }

    fn can_break_now(&self) -> Result<CanBreak, Box<dyn std::error::Error>> {
        self.can_break().map_err(|google_cal_err| {
            Box::new(google_cal_err) as Box<dyn std::error::Error>
        })
    }
}

pub fn list_events(config: &Config) -> Result<(), InitError> {
    let google_calendar = GoogleCalendar::new(config)?;

    let event_calendar_lists = google_calendar.runtime.block_on(async {
        let mut event_calendar_lists = vec![];
        for fetcher in &google_calendar.fetchers {
            event_calendar_lists.extend(get_events(fetcher).await);
        }
        event_calendar_lists
    });

    for (email, res_event_list) in event_calendar_lists {
        println!("{email}:");
        match res_event_list {
            Err(err) => {
                println!("ERROR with Google Calendar: {err}");
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
    Ok(())
}

async fn get_events(
    cal_fetcher: &CalFetcher,
) -> Vec<(String, Result<Vec<Event>, GoogleCalErr>)> {
    let now: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
    let ten_minutes_ago: chrono::DateTime<chrono::Utc> =
        now - chrono::Duration::minutes(10);
    let in_twenty_mins: chrono::DateTime<chrono::Utc> =
        now + chrono::Duration::minutes(20);
    let start_time = ten_minutes_ago;
    let end_time = in_twenty_mins;

    let mut events_list = vec![];
    for calendar_id in &cal_fetcher.calendar_ids {
        // We only check calendar_ids that are equal to the email address we are looking for.
        //
        // TODO: Eventually, we probably want to let the user configure what email addresses
        // they want to look for events on.
        if &cal_fetcher.email == calendar_id {
            events_list.push((
                String::from(calendar_id),
                get_event(
                    &cal_fetcher.hub,
                    calendar_id,
                    start_time,
                    end_time,
                    false,
                )
                .await,
            ));
        }
    }

    events_list
}

pub fn ignore_event(config: &Config, event_id: &str) -> Result<(), InitError> {
    let google_calendar = GoogleCalendar::new(config)?;

    google_calendar.runtime.block_on(async {
        for fetcher in &google_calendar.fetchers {
            for calendar_id in &fetcher.calendar_ids {
                // We only check calendar_ids that are equal to the email address we are looking
                // for.
                //
                // TODO: Eventually, we probably want to let the user configure what email
                // addresses they want to look for events on.
                if calendar_id == &fetcher.email {
                    let mut props = HashMap::new();
                    props.insert(
                        "ignore-break-time".to_string(),
                        "true".to_string(),
                    );
                    let extended_props = EventExtendedProperties {
                        private: Some(props),
                        ..EventExtendedProperties::default()
                    };
                    let req = Event {
                        extended_properties: Some(extended_props),
                        ..Event::default()
                    };
                    let _res = fetcher
                        .hub
                        .events()
                        .patch(req, calendar_id, event_id)
                        .add_scope(Scope::Readonly)
                        .add_scope(Scope::Event)
                        .doit()
                        .await;
                    // println!("event_id: {}, calendar_id: {}, res: {:?}", event_id, calendar_id, res);
                }
            }
        }
    });
    Ok(())
}
