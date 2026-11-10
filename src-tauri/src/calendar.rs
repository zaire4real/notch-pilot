//! Calendar + Reminders — v0.2 Phase 3 (F2/F3).
//!
//! Approach 1 from the plan: AppleScript (`osascript`) to
//! Calendar.app and Reminders.app. No new crates.
//!
//! Honest limitations of this approach:
//! - It needs macOS Automation permission the first time
//!   (System Settings → Privacy & Security → Automation → allow
//!   NotchPilot to control Calendar / Reminders). If denied, the
//!   commands return `status: "permission_denied"` and the frontend
//!   shows a clear message instead of a silent blank section (R7).
//! - Calendar.app / Reminders.app must already be running. The
//!   scripts check `application "X" is running` first and return
//!   `status: "not_running"` without launching the app (launching
//!   Calendar.app uninvited on every panel expand would pop a
//!   window in the user's face). The frontend tells the user to
//!   open the app.
//! - Dates come back from AppleScript as local wall-clock
//!   components (YYYY-MM-DD HH:MM); Rust converts them to epoch
//!   seconds with the local UTC offset read from `date +%z`, so
//!   the countdown to the next event is correct in any timezone.
//! - Reminders are identified by their AppleScript `id` (unique,
//!   read-only), so checking one off completes exactly that
//!   reminder even when two share a name.
//!
//! The `NSCalendarsUsageDescription` / `NSRemindersUsageDescription`
//! keys in `src-tauri/Info.plist` are for EventKit (the plan's
//! Approach 2); AppleScript uses Automation TCC instead, so those
//! keys are not what gates this phase — they are included now so a
//! later EventKit move does not need a second plist change.

use serde::Serialize;

use crate::settings;

// ---------------------------------------------------------------------------
// Data model (serialized to the frontend)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CalendarEvent {
    pub title: String,
    pub calendar: String,
    /// Local wall-clock time, "YYYY-MM-DD HH:MM".
    pub start: String,
    pub end: String,
    pub start_epoch: i64,
    pub end_epoch: i64,
    pub all_day: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct NextEvent {
    pub title: String,
    pub start: String,
    /// Whole minutes until the event starts; 0 while it is ongoing.
    pub minutes_until: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CalendarData {
    /// "ok" | "not_running" | "permission_denied" | "unavailable".
    pub status: String,
    pub events: Vec<CalendarEvent>,
    pub tomorrow: Option<CalendarEvent>,
    pub next: Option<NextEvent>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ReminderItem {
    pub id: String,
    pub title: String,
    pub list: String,
    /// Local wall-clock due time, "YYYY-MM-DD HH:MM", or "" when none.
    pub due: String,
    pub due_epoch: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RemindersData {
    /// "ok" | "disabled" | "not_running" | "permission_denied" |
    /// "unavailable".
    pub status: String,
    pub reminders: Vec<ReminderItem>,
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_calendar_data(app: tauri::AppHandle) -> CalendarData {
    let empty = |status: &str| CalendarData {
        status: status.to_string(),
        events: Vec::new(),
        tomorrow: None,
        next: None,
    };
    // Which calendars to show comes from Settings → Calendar
    // (Phase 0, persist-only until now). Empty = all calendars.
    let settings = settings::load_settings(&app);
    let allowed = settings.calendar.calendars.clone();
    let script = calendar_script(&allowed);
    let output = match run_osascript(&script) {
        Ok(out) => out,
        Err(stderr) => return empty(&classify_error(&stderr)),
    };
    parse_calendar_output(&output)
}

#[tauri::command]
pub fn get_reminders(app: tauri::AppHandle) -> RemindersData {
    let empty = |status: &str| RemindersData {
        status: status.to_string(),
        reminders: Vec::new(),
    };
    // Reminders on/off comes from Settings → Calendar. When off,
    // the frontend hides the section entirely.
    let settings = settings::load_settings(&app);
    if !settings.calendar.reminders {
        return empty("disabled");
    }
    let output = match run_osascript(REMINDERS_SCRIPT) {
        Ok(out) => out,
        Err(stderr) => return empty(&classify_error(&stderr)),
    };
    parse_reminders_output(&output)
}

#[tauri::command]
pub fn complete_reminder(reminder_id: String) -> Result<(), String> {
    let script = format!(
        "if application \"Reminders\" is running then\n\
         \twith timeout of 25 seconds\n\
         \t\ttell application \"Reminders\"\n\
         \t\t\trepeat with aList in lists\n\
         \t\t\t\ttry\n\
         \t\t\t\t\tset rmd to first reminder of aList whose id is \"{escaped}\"\n\
         \t\t\t\t\tset completed of rmd to true\n\
         \t\t\t\t\treturn \"STATUS:OK\"\n\
         \t\t\t\ton error\n\
         \t\t\t\tend try\n\
         \t\t\tend repeat\n\
         \t\t\treturn \"STATUS:NOT_FOUND\"\n\
         \t\tend tell\n\
         \tend timeout\n\
         else\n\
         \treturn \"STATUS:NOT_RUNNING\"\n\
         end if",
        escaped = applescript_escape(&reminder_id),
    );
    match run_osascript(&script) {
        Ok(out) => {
            let out = out.trim();
            if out == "STATUS:OK" {
                Ok(())
            } else if out == "STATUS:NOT_RUNNING" {
                Err("Reminders.app is not running".to_string())
            } else if out == "STATUS:NOT_FOUND" {
                Err("Reminder not found".to_string())
            } else {
                Err("Could not complete the reminder".to_string())
            }
        }
        Err(stderr) => Err(classify_error(&stderr).replace('_', " ")),
    }
}

// ---------------------------------------------------------------------------
// AppleScript
// ---------------------------------------------------------------------------

/// Escape a value for embedding in an AppleScript double-quoted
/// string literal.
fn applescript_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Calendar script. `allowed` is the Settings calendar-name filter
/// (empty = all calendars). Returns one line per event plus a
/// `STATUS:` first line. Fields are separated by ASCII 31 (unit
/// separator) so `|` and commas in titles cannot break parsing.
fn calendar_script(allowed: &[String]) -> String {
    let allowed_list = if allowed.is_empty() {
        "{}".to_string()
    } else {
        let names: Vec<String> = allowed
            .iter()
            .map(|n| format!("\"{}\"", applescript_escape(n)))
            .collect();
        format!("{{{}}}", names.join(", "))
    };
    format!(
        "on pad(n)\n\
         \tif n < 10 then return \"0\" & (n as text)\n\
         \treturn n as text\n\
         end pad\n\
         on fmtDate(d)\n\
         \tset y to year of d as integer\n\
         \tset m to (month of d) as integer\n\
         \tset dd to day of d as integer\n\
         \tset hh to hours of d as integer\n\
         \tset mm to minutes of d as integer\n\
         \treturn (y as text) & \"-\" & my pad(m) & \"-\" & my pad(dd) & \" \" & my pad(hh) & \":\" & my pad(mm)\n\
         end fmtDate\n\
         set todayStart to current date\n\
         set time of todayStart to 0\n\
         set dayAfterStart to todayStart + 2 * days\n\
         set sep to ASCII character 31\n\
         if application \"Calendar\" is running then\n\
         \twith timeout of 25 seconds\n\
         \t\ttell application \"Calendar\"\n\
         \t\t\tset allowedNames to {allowed_list}\n\
         \t\t\tset outText to \"STATUS:OK\"\n\
         \t\t\trepeat with aCal in calendars\n\
         \t\t\t\tset calName to title of aCal\n\
         \t\t\t\tif allowedNames is {{}} or calName is in allowedNames then\n\
         \t\t\t\t\tset evs to every event of aCal whose end date > todayStart and start date < dayAfterStart\n\
         \t\t\t\t\trepeat with ev in evs\n\
         \t\t\t\t\t\tset sd to start date of ev\n\
         \t\t\t\t\t\tset ed to end date of ev\n\
         \t\t\t\t\t\tif allday event of ev then\n\
         \t\t\t\t\t\t\tset allDayFlag to \"1\"\n\
         \t\t\t\t\t\telse\n\
         \t\t\t\t\t\t\tset allDayFlag to \"0\"\n\
         \t\t\t\t\t\tend if\n\
         \t\t\t\t\t\tset outText to outText & linefeed & \"EV\" & sep & my fmtDate(sd) & sep & my fmtDate(ed) & sep & allDayFlag & sep & calName & sep & (summary of ev)\n\
         \t\t\t\t\tend repeat\n\
         \t\t\t\tend if\n\
         \t\t\tend repeat\n\
         \t\t\treturn outText\n\
         \t\tend tell\n\
         \tend timeout\n\
         else\n\
         \treturn \"STATUS:NOT_RUNNING\"\n\
         end if"
    )
}

/// Reminders script: every incomplete reminder in every list, with
/// its stable `id`, list name and due date ("" when none).
const REMINDERS_SCRIPT: &str = "on pad(n)\n\
\tif n < 10 then return \"0\" & (n as text)\n\
\treturn n as text\n\
end pad\n\
on fmtDate(d)\n\
\tset y to year of d as integer\n\
\tset m to (month of d) as integer\n\
\tset dd to day of d as integer\n\
\tset hh to hours of d as integer\n\
\tset mm to minutes of d as integer\n\
\treturn (y as text) & \"-\" & my pad(m) & \"-\" & my pad(dd) & \" \" & my pad(hh) & \":\" & my pad(mm)\n\
end fmtDate\n\
set sep to ASCII character 31\n\
if application \"Reminders\" is running then\n\
\twith timeout of 25 seconds\n\
\t\ttell application \"Reminders\"\n\
\t\t\tset outText to \"STATUS:OK\"\n\
\t\t\trepeat with aList in lists\n\
\t\t\t\tset listName to name of aList\n\
\t\t\t\tset rems to every reminder of aList whose completed is false\n\
\t\t\t\trepeat with rmd in rems\n\
\t\t\t\t\tset dueText to \"\"\n\
\t\t\t\t\ttry\n\
\t\t\t\t\t\tset dd to due date of rmd\n\
\t\t\t\t\t\tif dd is not missing value then set dueText to my fmtDate(dd)\n\
\t\t\t\t\tend try\n\
\t\t\t\t\tset outText to outText & linefeed & \"RM\" & sep & (id of rmd) & sep & listName & sep & dueText & sep & (name of rmd)\n\
\t\t\t\tend repeat\n\
\t\t\tend repeat\n\
\t\t\treturn outText\n\
\t\tend tell\n\
\tend timeout\n\
else\n\
\treturn \"STATUS:NOT_RUNNING\"\n\
end if";

/// Run osascript. On failure the stderr text is returned so the
/// caller can tell a permission denial apart from any other error.
fn run_osascript(script: &str) -> Result<String, String> {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Map osascript stderr to a status string. -1743
/// (errAEEventNotPermitted) / "not authorized" is the Automation
/// permission denial; anything else is "unavailable".
fn classify_error(stderr: &str) -> String {
    let lower = stderr.to_lowercase();
    if lower.contains("-1743")
        || lower.contains("not authori")
        || lower.contains("permission")
        || lower.contains("not permitted")
    {
        "permission_denied".to_string()
    } else {
        "unavailable".to_string()
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

fn parse_calendar_output(output: &str) -> CalendarData {
    let mut status = "unavailable".to_string();
    let mut events: Vec<CalendarEvent> = Vec::new();
    let today = today_date_string();
    for line in output.lines() {
        if let Some(rest) = line.strip_prefix("STATUS:") {
            status = rest.trim().to_string();
            continue;
        }
        let fields: Vec<&str> = line.split('\u{1f}').collect();
        if fields.len() < 6 || fields[0] != "EV" {
            continue;
        }
        let start = fields[1].to_string();
        let end = fields[2].to_string();
        let Some(start_epoch) = parse_local_datetime(&start) else {
            continue;
        };
        let end_epoch = parse_local_datetime(&end).unwrap_or(start_epoch);
        events.push(CalendarEvent {
            title: fields[5].to_string(),
            calendar: fields[4].to_string(),
            start,
            end,
            start_epoch,
            end_epoch,
            all_day: fields[3] == "1",
        });
    }
    if status != "OK" && status != "NOT_RUNNING" {
        status = "unavailable".to_string();
    }
    let status = match status.as_str() {
        "OK" => "ok",
        "NOT_RUNNING" => "not_running",
        other => other,
    }
    .to_string();
    // Sort by start time; today vs tomorrow is decided by the date
    // part of the start string (the script only returns today +
    // tomorrow, so anything not dated today is tomorrow).
    events.sort_by_key(|e| e.start_epoch);
    let mut today_events: Vec<CalendarEvent> = Vec::new();
    let mut tomorrow: Option<CalendarEvent> = None;
    for event in events {
        if event.start.len() >= 10 && event.start[..10] == today {
            today_events.push(event);
        } else if tomorrow.is_none() {
            tomorrow = Some(event);
        }
    }
    // Next event: the earliest one that has not ended yet (an
    // ongoing event counts, with 0 minutes to go).
    let now = now_epoch();
    let next = today_events
        .iter()
        .filter(|e| e.end_epoch > now)
        .min_by_key(|e| e.start_epoch)
        .map(|e| NextEvent {
            title: e.title.clone(),
            start: e.start.clone(),
            minutes_until: ((e.start_epoch - now).max(0) / 60),
        });
    CalendarData {
        status,
        events: today_events,
        tomorrow,
        next,
    }
}

fn parse_reminders_output(output: &str) -> RemindersData {
    let mut status = "unavailable".to_string();
    let mut reminders: Vec<ReminderItem> = Vec::new();
    for line in output.lines() {
        if let Some(rest) = line.strip_prefix("STATUS:") {
            status = rest.trim().to_string();
            continue;
        }
        let fields: Vec<&str> = line.split('\u{1f}').collect();
        if fields.len() < 5 || fields[0] != "RM" {
            continue;
        }
        let due = fields[3].to_string();
        let due_epoch = if due.is_empty() {
            None
        } else {
            parse_local_datetime(&due)
        };
        reminders.push(ReminderItem {
            id: fields[1].to_string(),
            title: fields[4].to_string(),
            list: fields[2].to_string(),
            due,
            due_epoch,
        });
    }
    if status != "OK" && status != "NOT_RUNNING" {
        status = "unavailable".to_string();
    }
    let status = match status.as_str() {
        "OK" => "ok",
        "NOT_RUNNING" => "not_running",
        other => other,
    }
    .to_string();
    // Due reminders first (earliest due first); undated last,
    // then alphabetical so the order is stable.
    reminders.sort_by(|a, b| match (a.due_epoch, b.due_epoch) {
        (Some(x), Some(y)) => x.cmp(&y).then_with(|| a.title.cmp(&b.title)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.title.cmp(&b.title),
    });
    reminders.truncate(50);
    RemindersData { status, reminders }
}

// ---------------------------------------------------------------------------
// Local wall-clock time <-> epoch (no chrono crate; Hinnant's civil
// date algorithms + the UTC offset from `date +%z`)
// ---------------------------------------------------------------------------

fn now_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Local UTC offset in seconds, e.g. +28800 for CST (+0800).
/// `date +%z` prints "+0800" / "-0500" on macOS and Linux.
fn local_offset_seconds() -> i64 {
    let Ok(output) = std::process::Command::new("date")
        .arg("+%z")
        .output()
    else {
        return 0;
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let text = text.trim();
    if text.len() != 5 {
        return 0;
    }
    let sign: i64 = if text.starts_with('-') { -1 } else { 1 };
    let hours: i64 = text[1..3].parse().unwrap_or(0);
    let minutes: i64 = text[3..5].parse().unwrap_or(0);
    sign * (hours * 3600 + minutes * 60)
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// Parse "YYYY-MM-DD HH:MM" (local wall clock) to epoch seconds.
fn parse_local_datetime(text: &str) -> Option<i64> {
    let mut parts = text.split(['-', ' ', ':']);
    let year: i64 = parts.next()?.parse().ok()?;
    let month: i64 = parts.next()?.parse().ok()?;
    let day: i64 = parts.next()?.parse().ok()?;
    let hour: i64 = parts.next()?.parse().ok()?;
    let minute: i64 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(
        days_from_civil(year, month, day) * 86400 + hour * 3600 + minute * 60
            - local_offset_seconds(),
    )
}

/// Today's local date as "YYYY-MM-DD".
fn today_date_string() -> String {
    let local_now = now_epoch() + local_offset_seconds();
    let days = local_now.div_euclid(86400);
    // civil_from_days (Hinnant's inverse).
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::{days_from_civil, parse_calendar_output, parse_reminders_output};

    #[test]
    fn days_from_civil_matches_epoch() {
        // 1970-01-01 and 2026-11-10.
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2026, 11, 10), 20767);
    }

    #[test]
    fn parses_calendar_output() {
        // Build the input from today's real date so the test does
        // not depend on the day it runs.
        use super::today_date_string;
        let today = today_date_string();
        // The parser only needs "not today" for the tomorrow event,
        // so any other date string works — use a fixed far-future date.
        let tomorrow = "2999-01-02";
        let out = format!(
            "STATUS:OK\nEV\u{1f}{today} 10:00\u{1f}{today} 11:00\u{1f}0\u{1f}Work\u{1f}Design review\nEV\u{1f}{tomorrow} 09:00\u{1f}{tomorrow} 16:00\u{1f}1\u{1f}Work\u{1f}Safety training"
        );
        let data = parse_calendar_output(&out);
        assert_eq!(data.status, "ok");
        assert_eq!(data.events.len(), 1);
        assert_eq!(data.events[0].title, "Design review");
        assert!(!data.events[0].all_day);
        assert!(data.tomorrow.is_some());
        assert!(data.tomorrow.as_ref().unwrap().all_day);
    }

    #[test]
    fn parses_not_running_status() {
        let data = parse_calendar_output("STATUS:NOT_RUNNING");
        assert_eq!(data.status, "not_running");
        assert!(data.events.is_empty());
    }

    #[test]
    fn parses_reminders_output() {
        let out = "STATUS:OK\nRM\u{1f}x-coredata://abc\u{1f}Reminders\u{1f}2026-11-10 18:00\u{1f}Buy mouthwash\nRM\u{1f}x-coredata://def\u{1f}Reminders\u{1f}\u{1f}File clock-in correction";
        let data = parse_reminders_output(out);
        assert_eq!(data.status, "ok");
        assert_eq!(data.reminders.len(), 2);
        assert_eq!(data.reminders[0].title, "Buy mouthwash");
        assert_eq!(data.reminders[1].due_epoch, None);
    }
}
