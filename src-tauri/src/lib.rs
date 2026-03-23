use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const DB_PATH: &str =
    "/Users/zay/Desktop/Projects/activity-intelligence/data/activity.db";

const FOCUS_FILE: &str = "/Users/zay/Desktop/Projects/control-center/focus.json";

const SW_FINISHED: &str =
    "/Users/zay/Desktop/Software and     tools/finishedprojects.txt";
const RW_FINISHED: &str =
    "/Users/zay/Desktop/Research and writing/finishedprojectssearch.txt";
const SW_IDEAS: &str =
    "/Users/zay/Desktop/Software and     tools/ideas.txt";
const RW_IDEAS: &[&str] = &[
    "/Users/zay/Desktop/Research and writing/Humans and relationships/ideas.txt",
    "/Users/zay/Desktop/Research and writing/Ai and society/ideas.txt",
    "/Users/zay/Desktop/Research and writing/politics and power/ideas.txt",
    "/Users/zay/Desktop/Research and writing/Power and control/ideas.txt",
    "/Users/zay/Desktop/Research and writing/Identity and selfhood/ideas.txt",
];

#[derive(Serialize)]
struct FocusItem {
    id: String,
    title: String,
}

#[derive(Serialize)]
struct AppUsage {
    app: String,
    minutes: i64,
}

#[derive(Serialize)]
struct NowEvent {
    app: String,
    title: Option<String>,
    secs_ago: u64,
}

#[derive(Serialize)]
struct DashboardData {
    focus: Vec<FocusItem>,
    focus_extra: usize,
    activity: Vec<AppUsage>,
    projects_software_count: usize,
    projects_research_count: usize,
    ideas_software: Vec<String>,
    ideas_research: Vec<String>,
    working_on: Vec<ActiveProject>,
    current_focus: Option<CurrentFocus>,
    now_event: Option<NowEvent>,
}

#[derive(Serialize)]
struct ActiveProject {
    name: String,
    area: String,
    folder_path: String,
    recent_file: Option<String>,
    file_path: Option<String>,
    modified_secs_ago: u64,
}

#[derive(Serialize, Deserialize, Clone)]
struct CurrentFocus {
    name: String,
    area: String,
    folder_path: String,
    file_path: Option<String>,
    updated_at: u64,
}

#[derive(Serialize)]
struct CommandResult {
    action: String,
    message: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Deserialize)]
struct OpenAIMessage {
    content: String,
}

#[derive(Deserialize)]
struct ClassifyResult {
    intent: String,
    reminder_title: Option<String>,
    activity_label: Option<String>,
    idea_text: Option<String>,
    idea_category: Option<String>,
    focus_project: Option<String>,
}

// --- Shared helpers ---

fn ymd_to_days(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn iso_to_unix(ts: &str) -> Option<u64> {
    let b = ts.as_bytes();
    if b.len() < 19 { return None; }
    let p = |s: &[u8]| -> Option<i64> { std::str::from_utf8(s).ok()?.parse().ok() };
    let y = p(&b[0..4])?;
    let mo = p(&b[5..7])?;
    let d = p(&b[8..10])?;
    let h = p(&b[11..13])?;
    let mi = p(&b[14..16])?;
    let s = p(&b[17..19])?;
    let days = ymd_to_days(y, mo, d);
    Some((days as u64) * 86400 + h as u64 * 3600 + mi as u64 * 60 + s as u64)
}

fn fetch_now_event_from_aw() -> Option<NowEvent> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?;

    let buckets: serde_json::Value = client
        .get("http://localhost:5600/api/0/buckets")
        .send()
        .ok()?
        .json()
        .ok()?;

    let bucket_id = buckets
        .as_object()?
        .keys()
        .find(|k| k.contains("aw-watcher-window"))?
        .clone();

    let events: serde_json::Value = client
        .get(format!(
            "http://localhost:5600/api/0/buckets/{}/events",
            bucket_id
        ))
        .query(&[("limit", "1")])
        .send()
        .ok()?
        .json()
        .ok()?;

    let ev = events.as_array()?.first()?;
    let app = ev.get("data")?.get("app")?.as_str()?.to_string();

    const SKIP: &[&str] = &["loginwindow", "Dock", "SystemUIServer", "WindowServer"];
    if app.is_empty() || SKIP.contains(&app.as_str()) {
        return None;
    }

    let title = ev
        .get("data")
        .and_then(|d| d.get("title"))
        .and_then(|v| v.as_str())
        .filter(|t| !t.is_empty() && *t != app)
        .map(|t| t.to_string());

    let ts_str = ev.get("timestamp")?.as_str()?;
    let event_unix = iso_to_unix(ts_str)?;
    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let secs_ago = now_unix.saturating_sub(event_unix);

    if secs_ago > 600 {
        return None;
    }

    Some(NowEvent { app, title, secs_ago })
}

fn read_current_focus() -> Option<CurrentFocus> {
    let text = fs::read_to_string(FOCUS_FILE).ok()?;
    serde_json::from_str(&text).ok()
}

fn fetch_top_apps_from_aw() -> Option<Vec<AppUsage>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?;

    let buckets: serde_json::Value = client
        .get("http://localhost:5600/api/0/buckets")
        .send()
        .ok()?
        .json()
        .ok()?;

    let bucket_id = buckets
        .as_object()?
        .keys()
        .find(|k| k.contains("aw-watcher-window"))?
        .clone();

    // Compute start of today (UTC midnight) as ISO 8601 without chrono
    let start = {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let day_start = secs - (secs % 86400);
        let z = (day_start / 86400) as i64 + 719468;
        let era = if z >= 0 { z } else { z - 146096 } / 146097;
        let doe = (z - era * 146097) as u64;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe as i64 + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        format!("{:04}-{:02}-{:02}T00:00:00Z", y, m, d)
    };

    let events: serde_json::Value = client
        .get(format!(
            "http://localhost:5600/api/0/buckets/{}/events",
            bucket_id
        ))
        .query(&[("start", start.as_str()), ("limit", "200")])
        .send()
        .ok()?
        .json()
        .ok()?;

    let events = events.as_array()?;

    const SKIP: &[&str] = &["loginwindow", "Dock", "SystemUIServer", "WindowServer"];

    let mut totals: std::collections::HashMap<String, f64> =
        std::collections::HashMap::new();

    for ev in events {
        let duration = ev.get("duration").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let app = ev
            .get("data")
            .and_then(|d| d.get("app"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if app.is_empty() || SKIP.contains(&app.as_str()) {
            continue;
        }
        *totals.entry(app).or_insert(0.0) += duration;
    }

    let mut sorted: Vec<(String, f64)> = totals.into_iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    Some(
        sorted
            .into_iter()
            .take(3)
            .map(|(app, secs)| AppUsage {
                app,
                minutes: (secs / 60.0) as i64,
            })
            .collect(),
    )
}

fn get_dashboard_inner() -> Result<DashboardData, String> {
    let conn = Connection::open(DB_PATH).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT id, title FROM reminder_items \
             WHERE status = 'pending' \
             ORDER BY CASE WHEN due_at IS NULL THEN 1 ELSE 0 END, due_at ASC, created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let all_focus: Vec<FocusItem> = stmt
        .query_map([], |row| {
            Ok(FocusItem {
                id: row.get::<_, String>(0)?,
                title: row.get::<_, String>(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .filter(|f| !f.title.trim().is_empty())
        .collect();

    let focus_extra = all_focus.len().saturating_sub(3);
    let focus: Vec<FocusItem> = all_focus.into_iter().take(3).collect();

    let activity: Vec<AppUsage> = fetch_top_apps_from_aw().unwrap_or_default();

    let sw_text = fs::read_to_string(SW_FINISHED).unwrap_or_default();
    let projects_software_count = sw_text.matches("Summary:").count();

    let rw_text = fs::read_to_string(RW_FINISHED).unwrap_or_default();
    let projects_research_count = rw_text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count();

    let sw_ideas_text = fs::read_to_string(SW_IDEAS).unwrap_or_default();
    let ideas_software = parse_idea_titles(&sw_ideas_text, 3);

    let mut ideas_research: Vec<String> = Vec::new();
    for path in RW_IDEAS {
        if ideas_research.len() >= 3 {
            break;
        }
        let text = fs::read_to_string(path).unwrap_or_default();
        let remaining = 3 - ideas_research.len();
        let mut titles = parse_idea_titles(&text, remaining);
        ideas_research.append(&mut titles);
    }

    let working_on = scan_active_projects();
    let current_focus = read_current_focus();
    let now_event = fetch_now_event_from_aw();

    Ok(DashboardData {
        focus,
        focus_extra,
        activity,
        projects_software_count,
        projects_research_count,
        ideas_software,
        ideas_research,
        working_on,
        current_focus,
        now_event,
    })
}

fn insert_reminder(title: &str) -> Result<(), String> {
    let conn = Connection::open(DB_PATH).map_err(|e| e.to_string())?;
    let id = format!(
        "rem-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    conn.execute(
        "INSERT INTO reminder_items (id, title, note, due_at, status, created_at, completed_at) \
         VALUES (?1, ?2, NULL, NULL, 'pending', datetime('now'), NULL)",
        rusqlite::params![id, title],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn build_today_suggestion(data: &DashboardData) -> String {
    let mut parts = Vec::new();
    let total = data.focus.len() + data.focus_extra;
    if total > 0 {
        let titles: Vec<&str> = data.focus.iter().map(|f| f.title.as_str()).collect();
        parts.push(format!(
            "{} pending reminder{}: {}.",
            total,
            if total == 1 { "" } else { "s" },
            titles.join(", ")
        ));
    } else {
        parts.push("No pending reminders — your slate is clear.".to_string());
    }
    if let Some(top) = data.activity.first() {
        parts.push(format!("Top app today: {} ({}m).", top.app, top.minutes));
    }
    parts.join(" ")
}

fn mtime_secs(path: &Path) -> u64 {
    path.metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn best_mtime_and_file(dir: &Path) -> (u64, Option<String>) {
    let dir_mtime = mtime_secs(dir);
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return (dir_mtime, None),
    };
    let mut best_mtime = dir_mtime;
    let mut best_name: Option<String> = None;
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            continue;
        }
        let name = match p.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };
        if name.starts_with('.') {
            continue;
        }
        let mt = mtime_secs(&p);
        if mt > best_mtime {
            best_mtime = mt;
            best_name = Some(name);
        }
    }
    (best_mtime, best_name)
}

const SKIP_DIRS: &[&str] = &[
    "node_modules", "target", ".git", ".next", "__pycache__", "dist", ".venv",
];

fn scan_active_projects() -> Vec<ActiveProject> {
    let roots: &[(&str, &str)] = &[
        ("/Users/zay/Desktop/Projects", "Projects"),
        ("/Users/zay/Desktop/Software and     tools", "Software"),
        ("/Users/zay/Desktop/Research and writing", "Research"),
    ];

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut all: Vec<ActiveProject> = Vec::new();

    for (root_path, area) in roots {
        let entries = match fs::read_dir(root_path) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = match path.file_name() {
                Some(n) => n.to_string_lossy().to_string(),
                None => continue,
            };
            if name.starts_with('.') || SKIP_DIRS.contains(&name.as_str()) {
                continue;
            }
            let (mtime, recent_file) = best_mtime_and_file(&path);
            let modified_secs_ago = now.saturating_sub(mtime);
            let folder_path = path.to_string_lossy().to_string();
            let file_path = recent_file
                .as_ref()
                .map(|f| format!("{}/{}", folder_path, f));
            all.push(ActiveProject {
                name,
                area: area.to_string(),
                folder_path,
                recent_file,
                file_path,
                modified_secs_ago,
            });
        }
    }

    all.sort_by_key(|p| p.modified_secs_ago);
    all.truncate(5);
    all
}

async fn classify_intent(input: &str) -> Result<ClassifyResult, String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY is not set".to_string())?;

    let user_prompt = format!(
        "Classify this command into exactly one intent.\n\
         Intents: add_reminder, complete_from_summary, log_activity, capture_idea, set_focus, suggest_today, refresh, unknown.\n\
         Respond with JSON only, no markdown.\n\n\
         Format per intent:\n\
         - add_reminder: {{\"intent\":\"add_reminder\",\"reminder_title\":\"<clean title>\"}}\n\
         - complete_from_summary: {{\"intent\":\"complete_from_summary\"}}\n\
         - log_activity: {{\"intent\":\"log_activity\",\"activity_label\":\"<short label>\"}}\n\
         - capture_idea: {{\"intent\":\"capture_idea\",\"idea_text\":\"<the idea>\",\"idea_category\":\"software|research\"}}\n\
         - set_focus: {{\"intent\":\"set_focus\",\"focus_project\":\"<project name>\"}}\n\
         - suggest_today: {{\"intent\":\"suggest_today\"}}\n\
         - refresh: {{\"intent\":\"refresh\"}}\n\
         - unknown: {{\"intent\":\"unknown\"}}\n\n\
         Examples:\n\
         - \"remind me to call mom\" → {{\"intent\":\"add_reminder\",\"reminder_title\":\"Call mom\"}}\n\
         - \"I called the dentist and filed my taxes\" → {{\"intent\":\"complete_from_summary\"}}\n\
         - \"working on control center frontend\" → {{\"intent\":\"log_activity\",\"activity_label\":\"control center frontend\"}}\n\
         - \"idea: build a habit tracker\" → {{\"intent\":\"capture_idea\",\"idea_text\":\"Build a habit tracker\",\"idea_category\":\"software\"}}\n\
         - \"focus on control center\" → {{\"intent\":\"set_focus\",\"focus_project\":\"control center\"}}\n\
         - \"what should I do today\" → {{\"intent\":\"suggest_today\"}}\n\
         - \"refresh\" → {{\"intent\":\"refresh\"}}\n\n\
         Command: \"{}\"",
        input
    );

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            {
                "role": "system",
                "content": "You classify user commands into intents. Reply with JSON only, no markdown."
            },
            {
                "role": "user",
                "content": user_prompt
            }
        ],
        "temperature": 0
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("OpenAI {} — {}", status, text));
    }

    let oai: OpenAIResponse = resp
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    let content = oai
        .choices
        .first()
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| "Empty response from OpenAI".to_string())?;

    serde_json::from_str::<ClassifyResult>(&content)
        .map_err(|_| format!("Unexpected response: {}", content))
}

fn fetch_pending_reminders() -> Result<Vec<(String, String)>, String> {
    let conn = Connection::open(DB_PATH).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, title FROM reminder_items WHERE status = 'pending'")
        .map_err(|e| e.to_string())?;
    let items = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(items)
}

fn mark_done_by_ids(ids: &[String]) -> Result<usize, String> {
    let conn = Connection::open(DB_PATH).map_err(|e| e.to_string())?;
    let mut count = 0usize;
    for id in ids {
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM reminder_items WHERE id = ?1 AND status = 'pending'",
                rusqlite::params![id],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);
        if exists {
            conn.execute(
                "UPDATE reminder_items SET status = 'done', completed_at = datetime('now') WHERE id = ?1",
                rusqlite::params![id],
            )
            .map_err(|e| e.to_string())?;
            count += 1;
        }
    }
    Ok(count)
}

async fn match_completed_reminders(
    input: &str,
    pending: &[(String, String)],
) -> Result<Vec<String>, String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY is not set".to_string())?;

    let reminder_list = pending
        .iter()
        .map(|(id, title)| format!("id:{} title:{}", id, title))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "The user said: \"{}\"\n\nPending reminders:\n{}\n\n\
         Return the IDs of reminders that were completed based on what the user said.\n\
         Be conservative — only match with a clear semantic match.\n\
         Respond with JSON only: {{\"matched_ids\":[\"id1\",\"id2\"]}}",
        input, reminder_list
    );

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            {"role": "system", "content": "You match completed tasks to reminder IDs. Reply with JSON only."},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("OpenAI {} — {}", status, text));
    }

    let oai: OpenAIResponse = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
    let content = oai
        .choices
        .first()
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| "Empty response".to_string())?;

    #[derive(Deserialize)]
    struct MatchResult {
        matched_ids: Vec<String>,
    }
    serde_json::from_str::<MatchResult>(&content)
        .map(|r| r.matched_ids)
        .map_err(|_| format!("Unexpected response: {}", content))
}

fn log_activity_entry(label: &str) -> Result<(), String> {
    let label = label.trim();
    if label.is_empty() {
        return Err("Activity label is empty".to_string());
    }
    let conn = Connection::open(DB_PATH).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO passive_events (app, timestamp, duration_seconds, created_at) \
         VALUES (?1, datetime('now'), 60, datetime('now'))",
        rusqlite::params![label],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn append_idea(text: &str, category: &str) -> Result<(), String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("Idea text is empty".to_string());
    }
    let path = if category.to_lowercase().contains("research") {
        RW_IDEAS[0]
    } else {
        SW_IDEAS
    };
    let existing = fs::read_to_string(path).unwrap_or_default();
    let entry = format!("\n{}\n\nProject: Captured idea\n", text);
    let new_content = format!("{}{}", existing.trim_end(), entry);
    fs::write(path, new_content).map_err(|e| e.to_string())?;
    Ok(())
}

fn write_focus(name: &str, area: &str, folder_path: &str, file_path: Option<String>) -> Result<(), String> {
    let focus = CurrentFocus {
        name: name.to_string(),
        area: area.to_string(),
        folder_path: folder_path.to_string(),
        file_path,
        updated_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };
    let json = serde_json::to_string_pretty(&focus).map_err(|e| e.to_string())?;
    fs::write(FOCUS_FILE, json).map_err(|e| e.to_string())
}

async fn fallback_interpret(input: &str, data: &DashboardData) -> Result<CommandResult, String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY is not set".to_string())?;

    let focus_str = data
        .current_focus
        .as_ref()
        .map(|f| format!("{} ({})", f.name, f.area))
        .unwrap_or_else(|| "none".to_string());
    let reminders_str = if data.focus.is_empty() {
        "none".to_string()
    } else {
        data.focus.iter().map(|f| f.title.as_str()).collect::<Vec<_>>().join(", ")
    };
    let activity_str = if data.activity.is_empty() {
        "none".to_string()
    } else {
        data.activity
            .iter()
            .map(|a| format!("{} ({}m)", a.app, a.minutes))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let user_prompt = format!(
        "User said: \"{}\"\n\nContext:\n- Current focus: {}\n- Top reminders: {}\n- Top activity today: {}\n\n\
         Either:\n\
         1. Map to a valid intent and return JSON: {{\"intent\":\"<intent>\",...}} \
            Valid intents: add_reminder (needs reminder_title), log_activity (needs activity_label), \
            capture_idea (needs idea_text + idea_category), set_focus (needs focus_project), \
            suggest_today, refresh\n\
         2. Or return a helpful plain answer: {{\"intent\":\"answer\",\"message\":\"<your answer>\"}}\n\n\
         Reply with JSON only, no markdown.",
        input, focus_str, reminders_str, activity_str
    );

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            {
                "role": "system",
                "content": "You are an assistant for a personal control system. Interpret user intent flexibly."
            },
            {
                "role": "user",
                "content": user_prompt
            }
        ],
        "temperature": 0.3
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("OpenAI {} — {}", status, text));
    }

    let oai: OpenAIResponse = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
    let content = oai
        .choices
        .first()
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| "Empty response".to_string())?;

    #[derive(Deserialize)]
    struct FallbackResponse {
        intent: String,
        reminder_title: Option<String>,
        activity_label: Option<String>,
        idea_text: Option<String>,
        idea_category: Option<String>,
        focus_project: Option<String>,
        message: Option<String>,
    }

    let parsed = serde_json::from_str::<FallbackResponse>(&content)
        .map_err(|_| format!("Unexpected response: {}", content))?;

    match parsed.intent.as_str() {
        "suggest_today" => Ok(CommandResult {
            action: "suggest_today".to_string(),
            message: build_today_suggestion(data),
        }),
        "refresh" => Ok(CommandResult {
            action: "refresh".to_string(),
            message: "Refreshing dashboard.".to_string(),
        }),
        "add_reminder" => {
            let title = parsed
                .reminder_title
                .filter(|t| !t.trim().is_empty())
                .unwrap_or_else(|| input.to_string());
            match insert_reminder(&title) {
                Ok(_) => Ok(CommandResult {
                    action: "add_reminder".to_string(),
                    message: format!("\u{201c}{}\u{201d} added to reminders.", title),
                }),
                Err(e) => Err(e),
            }
        }
        "log_activity" => {
            let label = parsed
                .activity_label
                .filter(|l| !l.trim().is_empty())
                .unwrap_or_else(|| "Activity".to_string());
            match log_activity_entry(&label) {
                Ok(_) => Ok(CommandResult {
                    action: "log_activity".to_string(),
                    message: format!("Logged: {}.", label),
                }),
                Err(e) => Err(e),
            }
        }
        "capture_idea" => {
            let text = parsed
                .idea_text
                .filter(|t| !t.trim().is_empty())
                .unwrap_or_else(|| input.to_string());
            let category = parsed.idea_category.unwrap_or_else(|| "software".to_string());
            match append_idea(&text, &category) {
                Ok(_) => Ok(CommandResult {
                    action: "capture_idea".to_string(),
                    message: format!("Idea saved to {} notes.", category),
                }),
                Err(e) => Err(e),
            }
        }
        "set_focus" => {
            let hint = parsed.focus_project.unwrap_or_default().to_lowercase();
            let projects = scan_active_projects();
            let matched = projects.iter().find(|p| {
                let name_lower = p.name.to_lowercase();
                name_lower.contains(&hint) || hint.contains(&name_lower)
            });
            match matched {
                Some(p) => match write_focus(&p.name, &p.area, &p.folder_path, p.file_path.clone()) {
                    Ok(_) => Ok(CommandResult {
                        action: "set_focus".to_string(),
                        message: format!("Focus set to {}.", p.name),
                    }),
                    Err(e) => Err(e),
                },
                None => Ok(CommandResult {
                    action: "unknown".to_string(),
                    message: format!("No project matching \"{}\" found.", hint),
                }),
            }
        }
        "answer" | _ => Ok(CommandResult {
            action: "answer".to_string(),
            message: parsed
                .message
                .unwrap_or_else(|| "I couldn't act on that yet.".to_string()),
        }),
    }
}

// --- Tauri commands ---

#[tauri::command]
fn get_dashboard_data() -> Result<DashboardData, String> {
    get_dashboard_inner()
}

#[tauri::command]
fn add_reminder(title: String) -> Result<(), String> {
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err("Title cannot be empty".to_string());
    }
    insert_reminder(&title)
}

#[tauri::command]
fn mark_reminder_done(id: String) -> Result<(), String> {
    let conn = Connection::open(DB_PATH).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE reminder_items SET status = 'done', completed_at = datetime('now') WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn process_command(input: String) -> Result<CommandResult, String> {
    let input = input.trim().to_string();
    if input.is_empty() {
        return Ok(CommandResult {
            action: "unknown".to_string(),
            message: "Nothing to act on.".to_string(),
        });
    }

    let classified = match classify_intent(&input).await {
        Ok(c) => c,
        Err(e) => {
            return Ok(CommandResult {
                action: "unknown".to_string(),
                message: format!("Couldn't classify: {}", e),
            });
        }
    };

    match classified.intent.as_str() {
        "add_reminder" => {
            let title = classified
                .reminder_title
                .filter(|t| !t.trim().is_empty())
                .unwrap_or_else(|| input.clone());
            let title = title.trim().to_string();
            match insert_reminder(&title) {
                Ok(_) => Ok(CommandResult {
                    action: "add_reminder".to_string(),
                    message: format!("\u{201c}{}\u{201d} added to reminders.", title),
                }),
                Err(e) => Ok(CommandResult {
                    action: "unknown".to_string(),
                    message: format!("Failed to add reminder: {}", e),
                }),
            }
        }
        "suggest_today" => match get_dashboard_inner() {
            Ok(data) => Ok(CommandResult {
                action: "suggest_today".to_string(),
                message: build_today_suggestion(&data),
            }),
            Err(e) => Ok(CommandResult {
                action: "unknown".to_string(),
                message: format!("Couldn't load dashboard: {}", e),
            }),
        },
        "complete_from_summary" => {
            let pending = match fetch_pending_reminders() {
                Ok(p) => p,
                Err(e) => return Ok(CommandResult {
                    action: "unknown".to_string(),
                    message: format!("Couldn't fetch reminders: {}", e),
                }),
            };
            if pending.is_empty() {
                return Ok(CommandResult {
                    action: "complete_from_summary".to_string(),
                    message: "No pending reminders to match.".to_string(),
                });
            }
            match match_completed_reminders(&input, &pending).await {
                Ok(ids) if !ids.is_empty() => {
                    match mark_done_by_ids(&ids) {
                        Ok(count) => Ok(CommandResult {
                            action: "complete_from_summary".to_string(),
                            message: format!(
                                "Marked {} reminder{} done.",
                                count,
                                if count == 1 { "" } else { "s" }
                            ),
                        }),
                        Err(e) => Ok(CommandResult {
                            action: "unknown".to_string(),
                            message: format!("Failed to mark done: {}", e),
                        }),
                    }
                }
                Ok(_) => Ok(CommandResult {
                    action: "complete_from_summary".to_string(),
                    message: "No matching reminders found.".to_string(),
                }),
                Err(e) => Ok(CommandResult {
                    action: "unknown".to_string(),
                    message: format!("Couldn't match reminders: {}", e),
                }),
            }
        }
        "log_activity" => {
            let label = classified
                .activity_label
                .filter(|l| !l.trim().is_empty())
                .unwrap_or_else(|| "Activity".to_string());
            match log_activity_entry(&label) {
                Ok(_) => Ok(CommandResult {
                    action: "log_activity".to_string(),
                    message: format!("Logged: {}.", label),
                }),
                Err(e) => Ok(CommandResult {
                    action: "unknown".to_string(),
                    message: format!("Failed to log: {}", e),
                }),
            }
        }
        "capture_idea" => {
            let text = classified
                .idea_text
                .filter(|t| !t.trim().is_empty())
                .unwrap_or_else(|| input.clone());
            let category = classified.idea_category.unwrap_or_else(|| "software".to_string());
            match append_idea(&text, &category) {
                Ok(_) => Ok(CommandResult {
                    action: "capture_idea".to_string(),
                    message: format!("Idea saved to {} notes.", category),
                }),
                Err(e) => Ok(CommandResult {
                    action: "unknown".to_string(),
                    message: format!("Failed to save idea: {}", e),
                }),
            }
        }
        "set_focus" => {
            let hint = classified
                .focus_project
                .unwrap_or_default()
                .to_lowercase();
            let projects = scan_active_projects();
            let matched = projects.iter().find(|p| {
                let name_lower = p.name.to_lowercase();
                name_lower.contains(&hint) || hint.contains(&name_lower)
            });
            match matched {
                Some(p) => match write_focus(&p.name, &p.area, &p.folder_path, p.file_path.clone()) {
                    Ok(_) => Ok(CommandResult {
                        action: "set_focus".to_string(),
                        message: format!("Focus set to {}.", p.name),
                    }),
                    Err(e) => Ok(CommandResult {
                        action: "unknown".to_string(),
                        message: format!("Failed to set focus: {}", e),
                    }),
                },
                None => Ok(CommandResult {
                    action: "unknown".to_string(),
                    message: format!("No project matching \"{}\" found.", hint),
                }),
            }
        }
        "refresh" => Ok(CommandResult {
            action: "refresh".to_string(),
            message: "Refreshing dashboard.".to_string(),
        }),
        _ => match get_dashboard_inner() {
            Ok(data) => match fallback_interpret(&input, &data).await {
                Ok(result) => Ok(result),
                Err(_) => Ok(CommandResult {
                    action: "unknown".to_string(),
                    message: "I couldn't act on that yet.".to_string(),
                }),
            },
            Err(_) => Ok(CommandResult {
                action: "unknown".to_string(),
                message: "I couldn't act on that yet.".to_string(),
            }),
        },
    }
}

// --- Utilities ---

fn parse_idea_titles(text: &str, limit: usize) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let n = lines.len();
    let mut titles = Vec::new();

    for i in 0..n {
        if titles.len() >= limit {
            break;
        }
        let trimmed = lines[i].trim();
        if trimmed.is_empty()
            || trimmed.starts_with('{')
            || trimmed.starts_with('\\')
            || trimmed.starts_with('}')
        {
            continue;
        }
        let next_blank = i + 1 < n && lines[i + 1].trim().is_empty();
        let after_project = i + 2 < n && lines[i + 2].trim().starts_with("Project:");
        if next_blank && after_project {
            let title = trimmed
                .trim_start_matches('[')
                .trim_end_matches(']')
                .trim()
                .to_string();
            if !title.is_empty() {
                titles.push(title);
            }
        }
    }
    titles
}

#[tauri::command]
fn set_focus(
    name: String,
    area: String,
    folder_path: String,
    file_path: Option<String>,
) -> Result<(), String> {
    write_focus(&name, &area, &folder_path, file_path)
}

const ALLOWED_ROOTS: &[&str] = &[
    "/Users/zay/Desktop/Projects",
    "/Users/zay/Desktop/Software and     tools",
    "/Users/zay/Desktop/Research and writing",
];

#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    let allowed = ALLOWED_ROOTS.iter().any(|root| path.starts_with(root));
    if !allowed {
        return Err("Path is outside allowed scope".to_string());
    }
    std::process::Command::new("open")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_dashboard_data,
            add_reminder,
            open_path,
            set_focus,
            mark_reminder_done,
            process_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
