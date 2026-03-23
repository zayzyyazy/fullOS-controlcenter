use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const DB_PATH: &str =
    "/Users/zay/Desktop/Projects/activity-intelligence/data/activity.db";

const FOCUS_FILE: &str = "/Users/zay/Desktop/Projects/control-center/focus.json";

const ENV_FILE: &str = "/Users/zay/Desktop/Projects/control-center/.env";

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
    insight: String,
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

#[derive(Serialize, Clone)]
struct Item {
    id: String,
    item_type: String,
    title: String,
    content: String,
    status: String,
    tags: Option<String>,
    related_project: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct CaptureResult {
    item_id: String,
    item_type: String,
    title: String,
    message: String,
}

#[derive(Deserialize)]
struct AiCaptureResult {
    item_type: String,
    title: String,
    content: String,
    tags: Option<String>,
    related_project: Option<String>,
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

fn openai_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent("control-center/1.0")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

#[derive(Deserialize)]
struct ClassifyResult {
    intent: String,
    reminder_title: Option<String>,
    activity_label: Option<String>,
    idea_text: Option<String>,
    idea_category: Option<String>,
    focus_project: Option<String>,
    project_text: Option<String>,
    project_category: Option<String>,
}

#[derive(Deserialize)]
struct ProjectCleanResult {
    title: String,
    summary: String,
    tools: Option<String>,
    steps: Option<Vec<String>>,
    time_spent: Option<String>,
    category: String,
}

#[derive(Deserialize)]
struct IdeaCleanResult {
    title: String,
    project: String,
    category: String,
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

// --- Startup helpers ---

fn load_env_from_file(path: &str) -> bool {
    let contents = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return false,
    };
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            let key = key.trim();
            let val = val.trim();
            // Only set if not already in environment (shell env takes precedence)
            if std::env::var(key).is_err() {
                unsafe { std::env::set_var(key, val); }
            }
        }
    }
    true
}

fn startup_init() {
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "<unknown>".to_string());
    println!("STARTUP: current_dir={}", cwd);
    println!("STARTUP: DB_PATH={} exists={}", DB_PATH, Path::new(DB_PATH).exists());
    println!("STARTUP: FOCUS_FILE={} exists={}", FOCUS_FILE, Path::new(FOCUS_FILE).exists());

    let env_found = load_env_from_file(ENV_FILE);
    println!("STARTUP: .env path={} found={}", ENV_FILE, env_found);

    let key_loaded = std::env::var("OPENAI_API_KEY").is_ok();
    println!("STARTUP: OPENAI_API_KEY loaded={}", key_loaded);

    match ensure_items_table() {
        Ok(_) => println!("STARTUP: items table ready"),
        Err(e) => println!("STARTUP: could not init items table: {}", e),
    }
}

fn load_reminders_from_db() -> (Vec<FocusItem>, usize) {
    let result: Result<(Vec<FocusItem>, usize), String> = (|| {
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
        let focus = all_focus.into_iter().take(3).collect();
        Ok((focus, focus_extra))
    })();
    result.unwrap_or_else(|e| {
        println!("WARN load_reminders_from_db: DB unavailable — {}", e);
        (vec![], 0)
    })
}

async fn fetch_now_event_from_aw() -> Option<NowEvent> {
    println!("DEBUG fetch_now_event_from_aw: sending request to ActivityWatch");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?;

    let buckets: serde_json::Value = client
        .get("http://localhost:5600/api/0/buckets")
        .send()
        .await
        .ok()?
        .json()
        .await
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
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    println!("DEBUG fetch_now_event_from_aw: got response");

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

async fn fetch_top_apps_from_aw() -> Option<Vec<AppUsage>> {
    println!("DEBUG fetch_top_apps_from_aw: sending request to ActivityWatch");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?;

    let buckets: serde_json::Value = client
        .get("http://localhost:5600/api/0/buckets")
        .send()
        .await
        .ok()?
        .json()
        .await
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
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    println!("DEBUG fetch_top_apps_from_aw: got response");

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

async fn get_dashboard_inner() -> Result<DashboardData, String> {
    // Phase A: all synchronous DB/file work, completed before any await
    let (focus, focus_extra) = load_reminders_from_db();

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

    // Phase B: async network calls only
    let activity: Vec<AppUsage> = fetch_top_apps_from_aw().await.unwrap_or_default();
    let now_event = fetch_now_event_from_aw().await;

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
        insight: String::new(),
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

/// AI-powered suggest_today that includes captured items as context.
/// Returns a JSON-encoded decision object if parsing succeeds, or a plain string fallback.
async fn build_today_suggestion_ai(data: &DashboardData) -> String {
    let api_key = match std::env::var("OPENAI_API_KEY") {
        Ok(k) => k,
        Err(_) => return build_today_suggestion(data),
    };

    let items = list_recent_items(6).unwrap_or_default();
    println!(
        "DEBUG build_today_suggestion_ai: {} captured items in context",
        items.len()
    );
    for item in &items {
        println!("  context item: [{}] {}", item.item_type, item.title);
    }
    let items_str = format_items_for_context(&items);

    let focus = data
        .current_focus
        .as_ref()
        .map(|f| format!("{} ({})", f.name, f.area))
        .unwrap_or_else(|| "none".to_string());
    let reminders: Vec<&str> = data.focus.iter().map(|r| r.title.as_str()).collect();

    let user_prompt = format!(
        "USER CONTEXT:\n\
         - Current focus: {}\n\
         - Reminders: {:?}\n\
         - Recent captured items:\n{}",
        focus, reminders, items_str
    );

    let client = match openai_client() {
        Ok(c) => c,
        Err(_) => return build_today_suggestion(data),
    };

    let body = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            {
                "role": "system",
                "content": "You are an execution engine.\n\
                            You do NOT suggest. You COMMAND the next physical action.\n\n\
                            STRICT RULES:\n\n\
                            NEXT_ACTION:\n\
                            - MUST start with a strong verb (Open, Run, Write, Fix, Call, Review, Push, Deploy, Test, Send)\n\
                            - MUST be physically executable right now\n\
                            - MUST include specific object (file, project name, task name, person)\n\
                            - MUST be <= 10 words\n\
                            - MUST NOT contain: thing, item, stuff, work on, handle, improve\n\
                            - If captured items exist, prioritize them over generic actions\n\n\
                            BAD: \"Test the item\" / \"Work on dashboard\" / \"Improve UI\"\n\
                            GOOD: \"Run onboarding flow in control-center app\" / \"Fix input field padding in Home.tsx\"\n\n\
                            WHY:\n\
                            - MAX 12 words\n\
                            - Must reference focus OR a specific captured item by name\n\
                            - No filler words\n\n\
                            AFTER:\n\
                            - MAX 8 words OR NONE\n\
                            - Must be a real next step, not generic advice\n\n\
                            ABSOLUTE RULE: If you cannot find a concrete action, pick the closest real task.\n\n\
                            Return EXACTLY:\n\
                            NEXT_ACTION: <action>\n\
                            WHY: <reason>\n\
                            AFTER: <next step or NONE>"
            },
            {"role": "user", "content": user_prompt}
        ],
        "temperature": 0.2,
        "max_tokens": 120
    });

    let resp = match client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return build_today_suggestion(data),
    };

    if !resp.status().is_success() {
        return build_today_suggestion(data);
    }

    let oai: OpenAIResponse = match resp.json().await {
        Ok(o) => o,
        Err(_) => return build_today_suggestion(data),
    };

    let text = match oai.choices.first() {
        Some(c) => c.message.content.trim().to_string(),
        None => return build_today_suggestion(data),
    };

    parse_ai_decision(&text).unwrap_or_else(|| {
        if text.is_empty() {
            build_today_suggestion(data)
        } else {
            text
        }
    })
}

/// Parse NEXT_ACTION / WHY / AFTER format into a JSON string.
/// Returns None if the format is not recognized (triggers fallback).
fn parse_ai_decision(text: &str) -> Option<String> {
    let mut action: Option<String> = None;
    let mut why: Option<String> = None;
    let mut after_val: serde_json::Value = serde_json::Value::Null;

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("NEXT_ACTION:") {
            action = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("WHY:") {
            why = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("AFTER:") {
            let val = rest.trim();
            if !val.eq_ignore_ascii_case("none") && !val.is_empty() {
                after_val = serde_json::Value::String(val.to_string());
            }
        }
    }

    // Reject vague NEXT_ACTION values — caller falls back to dumb suggestion
    let vague_words = ["thing", "item", "stuff", "work on", "handle", "improve"];
    if let Some(ref a) = action {
        let lower = a.to_lowercase();
        if vague_words.iter().any(|w| lower.contains(w)) {
            println!("DEBUG parse_ai_decision: rejected vague action {:?}", a);
            return None;
        }
    }

    match (action, why) {
        (Some(a), Some(w)) => {
            let decision = serde_json::json!({
                "__decision__": true,
                "action": a,
                "why": w,
                "after": after_val
            });
            serde_json::to_string(&decision).ok()
        }
        _ => None,
    }
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
    all
}

async fn classify_intent(input: &str) -> Result<ClassifyResult, String> {
    println!("DEBUG classify_intent: calling OpenAI for input={:?}", &input[..input.len().min(60)]);
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY is not set".to_string())?;

    let user_prompt = format!(
        "Classify this command for a personal productivity system. Choose exactly one intent.\n\n\
         Intents and when to use them:\n\
         - set_focus: user names or describes a project as their current focus or priority\n\
         - add_reminder: user wants to be reminded of something (\"remind me\", \"don't forget\", \"need to do\")\n\
         - complete_from_summary: user describes tasks they ALREADY FINISHED (\"I did X\", \"finished X\", \"completed X\", \"called X\")\n\
         - log_activity: user describes what they are currently doing or working on RIGHT NOW (not setting focus, just logging)\n\
         - capture_idea: user has a new product/tool/app/research idea to store for later\n\
         - suggest_today: user asking what to work on, wants a daily summary, or asks how their day looks\n\
         - refresh: user wants to reload or update the dashboard\n\
         - save_project: user describing a project they have COMPLETED and want to archive\n\
         - capture_task: user states a task/to-do not phrased as a time-sensitive reminder (e.g. \"task: X\", \"I need to finish X\", \"should do X\")\n\
         - capture_note: freeform thought, observation, feeling, or reflection that is not an action or question\n\
         - capture_question: a question the user wants to think about or research later\n\
         - capture_project_note: an update or note about a specific ongoing project (\"project: X\")\n\
         - capture_reminder_candidate: something to be reminded about, using \"reminder:\" prefix\n\
         - unknown: anything else\n\n\
         Key distinctions:\n\
         - \"focus on X\", \"my focus is X\", \"today my focus is X\", \"I want to focus on X\" → ALWAYS set_focus, even if phrased as a sentence like \"today my focus is building the interface for control center\"\n\
         - \"save project:\" prefix → always save_project\n\
         - Past tense finishing → complete_from_summary\n\
         - Future reminder → add_reminder\n\
         - Idea for a future project/tool → capture_idea\n\
         - Describing current ongoing work (not naming a project as focus) → log_activity\n\
         - For set_focus: extract the project name from the sentence. If user says \"today my focus is building interface for control center\", focus_project = \"control center\"\n\n\
         JSON format per intent (no markdown):\n\
         - set_focus: {{\"intent\":\"set_focus\",\"focus_project\":\"<project name extracted from sentence>\"}}\n\
         - add_reminder: {{\"intent\":\"add_reminder\",\"reminder_title\":\"<clean title>\"}}\n\
         - complete_from_summary: {{\"intent\":\"complete_from_summary\"}}\n\
         - log_activity: {{\"intent\":\"log_activity\",\"activity_label\":\"<short label>\"}}\n\
         - capture_idea: {{\"intent\":\"capture_idea\",\"idea_text\":\"<the idea>\",\"idea_category\":\"software|research\"}}\n\
         - suggest_today: {{\"intent\":\"suggest_today\"}}\n\
         - refresh: {{\"intent\":\"refresh\"}}\n\
         - save_project: {{\"intent\":\"save_project\",\"project_text\":\"<raw description>\",\"project_category\":\"software|research\"}}\n\
         - unknown: {{\"intent\":\"unknown\"}}\n\
         - capture_task: {{\"intent\":\"capture_task\"}}\n\
         - capture_note: {{\"intent\":\"capture_note\"}}\n\
         - capture_question: {{\"intent\":\"capture_question\"}}\n\
         - capture_project_note: {{\"intent\":\"capture_project_note\"}}\n\
         - capture_reminder_candidate: {{\"intent\":\"capture_reminder_candidate\"}}\n\n\
         Examples:\n\
         - \"focus on control center\" → {{\"intent\":\"set_focus\",\"focus_project\":\"control center\"}}\n\
         - \"my focus is control center\" → {{\"intent\":\"set_focus\",\"focus_project\":\"control center\"}}\n\
         - \"today my focus is control center\" → {{\"intent\":\"set_focus\",\"focus_project\":\"control center\"}}\n\
         - \"I want to focus on control center\" → {{\"intent\":\"set_focus\",\"focus_project\":\"control center\"}}\n\
         - \"today my focus is building interface for control center\" → {{\"intent\":\"set_focus\",\"focus_project\":\"control center\"}}\n\
         - \"working on cleanup app UI\" → {{\"intent\":\"log_activity\",\"activity_label\":\"cleanup app UI\"}}\n\
         - \"what should I do now\" → {{\"intent\":\"suggest_today\"}}\n\
         - \"I finished the reminder system\" → {{\"intent\":\"complete_from_summary\"}}\n\
         - \"remind me to call mom\" → {{\"intent\":\"add_reminder\",\"reminder_title\":\"Call mom\"}}\n\
         - \"I called the dentist and filed my taxes\" → {{\"intent\":\"complete_from_summary\"}}\n\
         - \"idea: build a habit tracker\" → {{\"intent\":\"capture_idea\",\"idea_text\":\"build a habit tracker\",\"idea_category\":\"software\"}}\n\
         - \"thinking about making a menu bar timer app\" → {{\"intent\":\"capture_idea\",\"idea_text\":\"menu bar timer app\",\"idea_category\":\"software\"}}\n\
         - \"what should I do today\" → {{\"intent\":\"suggest_today\"}}\n\
         - \"refresh\" → {{\"intent\":\"refresh\"}}\n\
         - \"save project: built a Tauri desktop app with AI commands\" → {{\"intent\":\"save_project\",\"project_text\":\"built a Tauri desktop app with AI commands\",\"project_category\":\"software\"}}\n\
         - \"save project: wrote an essay on AI and society\" → {{\"intent\":\"save_project\",\"project_text\":\"wrote an essay on AI and society\",\"project_category\":\"research\"}}\n\
         - \"I keep feeling like reminders and projects overlap visually\" → {{\"intent\":\"capture_note\"}}\n\
         - \"task: review the PR before end of day\" → {{\"intent\":\"capture_task\"}}\n\
         - \"question: what is the best way to structure tasks vs notes?\" → {{\"intent\":\"capture_question\"}}\n\
         - \"project: redesigned the command bar hierarchy\" → {{\"intent\":\"capture_project_note\"}}\n\
         - \"I need to clean up the database schema\" → {{\"intent\":\"capture_task\"}}\n\
         - \"wondering if I should separate tasks and reminders\" → {{\"intent\":\"capture_question\"}}\n\n\
         Command: \"{}\"",
        input
    );

    let client = openai_client()?;
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
        .map_err(|e| {
            let key_present = std::env::var("OPENAI_API_KEY").is_ok();
            eprintln!(
                "OpenAI send error: {:?} | endpoint=https://api.openai.com/v1/chat/completions | key_present={}",
                e, key_present
            );
            format!("Request failed: {:?}", e)
        })?;

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

    println!("DEBUG classify_intent: got response intent area={:?}", &content[..content.len().min(80)]);
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

    let client = openai_client()?;
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
        .map_err(|e| {
            let key_present = std::env::var("OPENAI_API_KEY").is_ok();
            eprintln!(
                "OpenAI send error: {:?} | endpoint=https://api.openai.com/v1/chat/completions | key_present={}",
                e, key_present
            );
            format!("Request failed: {:?}", e)
        })?;

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

fn write_idea_entry(title: &str, project: &str, category: &str) -> Result<String, String> {
    let title = title.trim();
    let project = project.trim();
    if title.is_empty() {
        return Err("Idea title is empty".to_string());
    }
    let path = if category.to_lowercase().contains("research") {
        RW_IDEAS[0]
    } else {
        SW_IDEAS
    };
    let existing = fs::read_to_string(path).unwrap_or_default();
    let entry = format!("\n{}\n\nProject: {}\n", title, project);
    let new_content = format!("{}{}", existing.trim_end(), entry);
    fs::write(path, new_content).map_err(|e| e.to_string())?;
    let label = if category.to_lowercase().contains("research") {
        "Research ideas"
    } else {
        "Software ideas"
    };
    Ok(label.to_string())
}

// --- Items (universal capture) ---

fn ensure_items_table() -> Result<(), String> {
    let conn = Connection::open(DB_PATH).map_err(|e| e.to_string())?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS items (
            id               TEXT PRIMARY KEY,
            item_type        TEXT NOT NULL,
            title            TEXT NOT NULL,
            content          TEXT NOT NULL DEFAULT '',
            status           TEXT NOT NULL DEFAULT 'active',
            tags             TEXT,
            related_project  TEXT,
            created_at       TEXT NOT NULL,
            updated_at       TEXT NOT NULL
        );",
    )
    .map_err(|e| e.to_string())
}

fn insert_item(
    item_type: &str,
    title: &str,
    content: &str,
    tags: Option<&str>,
    related_project: Option<&str>,
) -> Result<String, String> {
    let _ = ensure_items_table();
    let conn = Connection::open(DB_PATH).map_err(|e| e.to_string())?;
    let id = format!(
        "item-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    conn.execute(
        "INSERT INTO items \
         (id, item_type, title, content, status, tags, related_project, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?6, datetime('now'), datetime('now'))",
        rusqlite::params![id, item_type, title, content, tags, related_project],
    )
    .map_err(|e| e.to_string())?;
    Ok(id)
}

fn list_recent_items(limit: usize) -> Result<Vec<Item>, String> {
    let _ = ensure_items_table();
    let conn = Connection::open(DB_PATH).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, item_type, title, content, status, tags, related_project, \
                    created_at, updated_at \
             FROM items \
             WHERE status = 'active' \
             ORDER BY created_at DESC \
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let items = stmt
        .query_map(rusqlite::params![limit as i64], |row| {
            Ok(Item {
                id: row.get(0)?,
                item_type: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                status: row.get(4)?,
                tags: row.get(5)?,
                related_project: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(items)
}

/// Format a slice of active items into a compact, prompt-ready string.
fn format_items_for_context(items: &[Item]) -> String {
    if items.is_empty() {
        return "none".to_string();
    }
    items
        .iter()
        .map(|item| {
            let mut line = format!("[{}] {}", item.item_type, item.title);
            if let Some(proj) = &item.related_project {
                if !proj.is_empty() {
                    line.push_str(&format!(" (project: {})", proj));
                }
            }
            if !item.content.is_empty() && item.content != item.title {
                let snip_len = item.content.len().min(100);
                let snip = &item.content[..snip_len];
                line.push_str(&format!(" — {}", snip));
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn classify_capture_rule_based(input: &str) -> (&'static str, String) {
    let lower = input.trim().to_lowercase();
    let prefixes: &[(&str, &'static str)] = &[
        ("task:", "task"),
        ("idea:", "idea"),
        ("question:", "question"),
        ("note:", "note"),
        ("project:", "project_note"),
        ("reminder:", "reminder_candidate"),
    ];
    for (prefix, item_type) in prefixes {
        if lower.starts_with(prefix) {
            let raw = input[prefix.len()..].trim().to_string();
            let title = if raw.is_empty() { input.trim().to_string() } else { raw };
            return (item_type, title);
        }
    }
    ("note", input.trim().to_string())
}

async fn classify_capture_with_ai(input: &str) -> Result<AiCaptureResult, String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY is not set".to_string())?;

    let prompt = format!(
        "Classify and structure this personal capture. Return JSON only, no markdown.\n\n\
         Input: \"{}\"\n\n\
         Classify into exactly one item_type:\n\
         - task: something to do (action, TODO, next step)\n\
         - idea: a new concept, product idea, or creative thought\n\
         - question: something to think about or research\n\
         - note: observation, reflection, feeling, or general thought\n\
         - project_note: update or note about a specific project\n\
         - reminder_candidate: something time-sensitive to remember\n\n\
         Return JSON:\n\
         {{\"item_type\":\"note\",\"title\":\"<clean 3-8 word title>\",\
           \"content\":\"<original text, lightly cleaned>\",\
           \"tags\":null,\"related_project\":null}}",
        input
    );

    let client = openai_client()?;
    let body = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            {"role": "system", "content": "You classify and structure personal captures. Reply with JSON only, no markdown."},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.1
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

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

    serde_json::from_str::<AiCaptureResult>(&content)
        .map_err(|_| format!("Unexpected response: {}", content))
}

async fn reformat_idea(text: &str, hint_category: &str) -> Result<IdeaCleanResult, String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY is not set".to_string())?;

    let prompt = format!(
        "You are capturing a product or research idea for a personal knowledge system.\n\
         The user described this idea: \"{}\"\n\
         Category hint: \"{}\"\n\n\
         Generate a clean idea entry:\n\
         - title: short, specific, memorable name (3–6 words, title case, no filler like \"App\" or \"Tool\" unless essential)\n\
         - project: a brief context label that says what kind of idea this is (e.g. \"Productivity software\", \"AI research\", \"Writing project\", \"Developer tooling\")\n\
         - category: \"software\" if it's a technical product/tool/app/script; \"research\" if it's academic, writing, or analysis\n\n\
         Rules:\n\
         - Do not invent details not in the input\n\
         - Title must be specific enough to stand out from other ideas\n\
         - Do not use \"Captured idea\" as the project value — be descriptive\n\n\
         Respond with JSON only, no markdown:\n\
         {{\"title\":\"...\",\"project\":\"...\",\"category\":\"software\"}}",
        text, hint_category
    );

    let client = openai_client()?;
    let body = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            {"role": "system", "content": "You produce clean structured idea entries. Reply with JSON only, no markdown."},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.2
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            let key_present = std::env::var("OPENAI_API_KEY").is_ok();
            eprintln!(
                "OpenAI send error: {:?} | endpoint=https://api.openai.com/v1/chat/completions | key_present={}",
                e, key_present
            );
            format!("Request failed: {:?}", e)
        })?;

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

    serde_json::from_str::<IdeaCleanResult>(&content)
        .map_err(|_| format!("Unexpected response: {}", content))
}

async fn reformat_project(raw: &str, hint_category: &str) -> Result<ProjectCleanResult, String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY is not set".to_string())?;

    let prompt = format!(
        "You are a project archivist. The user has described a finished project in casual, informal text.\n\
         Produce a clean, well-structured entry. Be specific, not generic. Sound polished, not robotic.\n\n\
         Category hint: \"{}\"\n\
         Raw input: \"{}\"\n\n\
         Rules:\n\
         - title: short, clear, specific project title (3–7 words, title case — not generic like \"Software Project\")\n\
         - summary: 3–5 sentences, clean and specific, describing what was built/written, why, and how. No filler phrases.\n\
         - tools: comma-separated list of tools/languages/frameworks/libraries used, or null if none mentioned\n\
         - steps: array of 3–6 key steps or phases taken (e.g. [\"Designed schema\", \"Built API layer\", \"Integrated with frontend\"]), or null if not enough detail\n\
         - time_spent: duration if mentioned (e.g. \"4–6 hours\"), or null\n\
         - category: \"software\" if clearly technical (code, app, tool, script, API, data); \"research\" otherwise\n\
         - if category is ambiguous, default to \"research\" unless obviously technical\n\
         - do not invent details not implied by the raw text\n\n\
         Respond with JSON only, no markdown:\n\
         {{\"title\":\"...\",\"summary\":\"...\",\"tools\":null,\"steps\":null,\"time_spent\":null,\"category\":\"software\"}}",
        hint_category, raw
    );

    let client = openai_client()?;
    let body = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            {"role": "system", "content": "You produce clean structured project entries. Reply with JSON only, no markdown."},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.2
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            let key_present = std::env::var("OPENAI_API_KEY").is_ok();
            eprintln!(
                "OpenAI send error: {:?} | endpoint=https://api.openai.com/v1/chat/completions | key_present={}",
                e, key_present
            );
            format!("Request failed: {:?}", e)
        })?;

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

    let result = serde_json::from_str::<ProjectCleanResult>(&content)
        .map_err(|_| format!("Unexpected response: {}", content))?;

    if result.title.trim().is_empty() || result.summary.trim().is_empty() {
        return Err("AI returned empty title or summary".to_string());
    }

    Ok(result)
}

fn append_finished_project(entry: &ProjectCleanResult) -> Result<String, String> {
    let (path, label) = if entry.category.to_lowercase().contains("software") {
        (SW_FINISHED, "Software & Tools")
    } else {
        (RW_FINISHED, "Research & Writing")
    };

    // Safety: only write to these two known files
    if path != SW_FINISHED && path != RW_FINISHED {
        return Err("Invalid target file".to_string());
    }

    let existing = fs::read_to_string(path).unwrap_or_default();

    let mut block = format!("\n{}\n\nSummary:\n{}\n", entry.title.trim(), entry.summary.trim());
    if let Some(tools) = &entry.tools {
        let tools = tools.trim();
        if !tools.is_empty() {
            block.push_str(&format!("\nTools:\n{}\n", tools));
        }
    }
    if let Some(steps) = &entry.steps {
        let filtered: Vec<&String> = steps.iter().filter(|s| !s.trim().is_empty()).collect();
        if !filtered.is_empty() {
            let lines: Vec<String> = filtered.iter().map(|s| format!("- {}", s.trim())).collect();
            block.push_str(&format!("\nSteps:\n{}\n", lines.join("\n")));
        }
    }
    let time_val = entry
        .time_spent
        .as_deref()
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .unwrap_or("not specified");
    block.push_str(&format!("\nTime:\n{}\n", time_val));

    let new_content = format!("{}{}", existing.trim_end(), block);
    fs::write(path, new_content).map_err(|e| e.to_string())?;

    Ok(label.to_string())
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

fn normalize_for_match(s: &str) -> String {
    s.chars()
        .map(|c| if c == '-' || c == '_' { ' ' } else { c })
        .collect::<String>()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Score how well a project name matches the input text.
/// +3 if full project name appears in input (substring)
/// +2 if all words of project appear in input
/// +1 per matching word
fn score_project_match(input_norm: &str, project_norm: &str) -> i32 {
    if project_norm.is_empty() {
        return 0;
    }
    let mut score = 0i32;
    if input_norm.contains(project_norm) {
        score += 3;
    }
    let p_words: Vec<&str> = project_norm.split_whitespace().collect();
    let i_words: Vec<&str> = input_norm.split_whitespace().collect();
    if !p_words.is_empty() {
        if p_words.iter().all(|w| i_words.contains(w)) {
            score += 2;
        }
        score += p_words.iter().filter(|w| i_words.contains(w)).count() as i32;
    }
    score
}

/// Find the best matching project from natural language input.
/// Uses scoring: highest score wins; ties broken by most-recently-active
/// (projects must be pre-sorted by modified_secs_ago ascending).
fn extract_best_project_match<'a>(input: &str, projects: &'a [ActiveProject]) -> Option<&'a ActiveProject> {
    let input_norm = normalize_for_match(input);
    println!("DEBUG extract_best_project_match: input_norm={:?}", input_norm);

    let mut best_score = 0i32;
    let mut best: Option<&'a ActiveProject> = None;

    for project in projects {
        let project_norm = normalize_for_match(&project.name);
        let score = score_project_match(&input_norm, &project_norm);
        if score > best_score {
            best_score = score;
            best = Some(project);
        }
    }

    println!(
        "DEBUG extract_best_project_match: best={:?} score={}",
        best.map(|p| p.name.as_str()),
        best_score
    );
    if best_score > 0 { best } else { None }
}

async fn fallback_interpret(input: &str, data: &DashboardData) -> Result<CommandResult, String> {
    println!("DEBUG fallback_interpret: calling OpenAI for input={:?}", &input[..input.len().min(60)]);
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
        data.focus
            .iter()
            .take(3)
            .enumerate()
            .map(|(i, f)| format!("{}. {}", i + 1, f.title))
            .collect::<Vec<_>>()
            .join("; ")
    };

    let recent_activity_str = if data.working_on.is_empty() {
        "none".to_string()
    } else {
        data.working_on
            .iter()
            .take(5)
            .map(|p| {
                let t = p.modified_secs_ago;
                let when = if t < 120 { "just now".to_string() }
                    else if t < 3600 { format!("{}m ago", t / 60) }
                    else if t < 86400 { format!("{}h ago", t / 3600) }
                    else { format!("{}d ago", t / 86400) };
                format!("{} ({})", p.name, when)
            })
            .collect::<Vec<_>>()
            .join(", ")
    };

    let recent_items = list_recent_items(7).unwrap_or_default();
    println!(
        "DEBUG fallback_interpret: {} captured items in context",
        recent_items.len()
    );
    for item in &recent_items {
        println!("  context item: [{}] {}", item.item_type, item.title);
    }
    let items_str = format_items_for_context(&recent_items);

    let user_prompt = format!(
        "User said: \"{}\"\n\n\
         Dashboard snapshot:\n\
         - Current focus: {}\n\
         - Pending reminders: {}\n\
         - Recent activity (last 5): {}\n\
         - Recent captured items (tasks/notes/ideas/questions — most recent first):\n{}\n\n\
         Instructions:\n\
         1. If the input maps to a system action, return JSON: {{\"intent\":\"<intent>\",...}}\n\
            Valid intents: add_reminder (needs reminder_title), log_activity (needs activity_label), \
            capture_idea (needs idea_text + idea_category), set_focus (needs focus_project), \
            save_project (needs project_text + project_category), suggest_today, refresh\n\
         2. Otherwise, respond as a sharp personal assistant.\n\
            - Lead with the most important next action — not an observation or summary.\n\
            - Reference specific project names, reminder titles, and captured items by name.\n\
            - If a captured task is related to the current focus, mention it directly.\n\
            - If a captured question or note suggests distraction or confusion, call it out.\n\
            - If reminders are pending, treat them as time-sensitive.\n\
            - 2–4 sentences, plain text, no filler. No 'I see that...' openers.\n\
            Return: {{\"intent\":\"answer\",\"message\":\"<your response>\"}}\n\n\
         Reply with JSON only, no markdown.",
        input, focus_str, reminders_str, recent_activity_str, items_str
    );

    let client = openai_client()?;
    let body = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            {
                "role": "system",
                "content": "You are a sharp personal operating system. Rules: \
                            (1) Lead with the most important next action — never with an observation or question. \
                            (2) Reference the user's actual focus, reminders, and activity by name. \
                            (3) If there is a clear next step, state it directly in the first sentence. \
                            (4) 2–4 sentences max. No filler. No generic advice. No 'I see that...' openers."
            },
            {
                "role": "user",
                "content": user_prompt
            }
        ],
        "temperature": 0.5
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            let key_present = std::env::var("OPENAI_API_KEY").is_ok();
            eprintln!(
                "OpenAI send error: {:?} | endpoint=https://api.openai.com/v1/chat/completions | key_present={}",
                e, key_present
            );
            format!("Request failed: {:?}", e)
        })?;

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
        project_text: Option<String>,
        project_category: Option<String>,
        message: Option<String>,
    }

    let parsed = serde_json::from_str::<FallbackResponse>(&content)
        .map_err(|_| format!("Unexpected response: {}", content))?;

    match parsed.intent.as_str() {
        "suggest_today" => Ok(CommandResult {
            action: "suggest_today".to_string(),
            message: build_today_suggestion_ai(data).await,
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
            match reformat_idea(&text, &category).await {
                Ok(clean) => {
                    // Always insert into items table so it appears in Captured
                    let _ = insert_item("idea", &clean.title, &text, None, Some(clean.project.as_str()));
                    let _ = write_idea_entry(&clean.title, &clean.project, &clean.category);
                    Ok(CommandResult {
                        action: "capture_idea".to_string(),
                        message: format!("Saved idea \u{2192} \u{201c}{}\u{201d}", clean.title),
                    })
                },
                Err(_) => {
                    let _ = insert_item("idea", &text, &text, None, None);
                    let _ = write_idea_entry(&text, "Captured idea", &category);
                    Ok(CommandResult {
                        action: "capture_idea".to_string(),
                        message: format!("Saved idea \u{2192} \u{201c}{}\u{201d}", text),
                    })
                },
            }
        }
        "set_focus" => {
            let hint = parsed.focus_project.unwrap_or_default();
            println!("DEBUG set_focus (fallback): raw_input={:?} focus_project={:?}", input, hint);
            let projects = scan_active_projects();
            let matched = extract_best_project_match(&hint, &projects)
                .or_else(|| extract_best_project_match(input, &projects));
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
                    message: format!("No project matching \"{}\". Available: {}",
                        hint,
                        projects.iter().take(5).map(|p| p.name.as_str()).collect::<Vec<_>>().join(", ")
                    ),
                }),
            }
        }
        "save_project" => {
            let raw = parsed
                .project_text
                .filter(|t| !t.trim().is_empty())
                .unwrap_or_else(|| input.to_string());
            let hint = parsed
                .project_category
                .unwrap_or_else(|| "software".to_string());
            match reformat_project(&raw, &hint).await {
                Ok(entry) => match append_finished_project(&entry) {
                    Ok(label) => Ok(CommandResult {
                        action: "save_project".to_string(),
                        message: format!("Saved \u{201c}{}\u{201d} to {}.", entry.title, label),
                    }),
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
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

async fn generate_insight(data: &DashboardData) -> Result<String, String> {
    println!("DEBUG generate_insight: calling OpenAI");
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY is not set".to_string())?;

    let focus = data
        .current_focus
        .as_ref()
        .map(|f| f.name.clone())
        .unwrap_or_else(|| "none".to_string());
    let reminders: Vec<&str> = data.focus.iter().map(|r| r.title.as_str()).collect();
    let activity: Vec<&str> = data.working_on.iter().take(3).map(|a| a.name.as_str()).collect();

    let recent_items = list_recent_items(5).unwrap_or_default();
    println!(
        "DEBUG generate_insight: {} captured items in context",
        recent_items.len()
    );
    for item in &recent_items {
        println!("  context item: [{}] {}", item.item_type, item.title);
    }
    let items_str = format_items_for_context(&recent_items);

    let prompt = format!(
        "Personal snapshot:\n\
         Current focus: {}\n\
         Pending reminders: {:?}\n\
         Recent activity: {:?}\n\
         Recent captured items:\n{}\n\n\
         Give exactly 1–2 sentences telling the user what to do next. \
         Lead with the most concrete action. Reference specific items, project names, or reminder titles. \
         No filler phrases. No generic advice.",
        focus, reminders, activity, items_str
    );

    let client = openai_client()?;
    let body = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            {
                "role": "system",
                "content": "You are a sharp personal assistant. Give brief, direct, actionable insights. \
                            Reference specific captured items and project names. Never give generic advice."
            },
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.35,
        "max_tokens": 100
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            let key_present = std::env::var("OPENAI_API_KEY").is_ok();
            eprintln!(
                "OpenAI send error: {:?} | endpoint=https://api.openai.com/v1/chat/completions | key_present={}",
                e, key_present
            );
            format!("Request failed: {:?}", e)
        })?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("OpenAI {} — {}", status, text));
    }

    let oai: OpenAIResponse = resp
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    println!("DEBUG generate_insight: got response");
    oai.choices
        .first()
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| "Empty response".to_string())
}

// --- Tauri commands ---

#[tauri::command]
async fn get_dashboard_data() -> Result<DashboardData, String> {
    let mut data = get_dashboard_inner().await?;
    data.insight = generate_insight(&data).await.unwrap_or_else(|_| {
        "Stay focused and continue your current work.".to_string()
    });
    Ok(data)
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
async fn capture_input(input: String) -> Result<CaptureResult, String> {
    let input = input.trim().to_string();
    if input.is_empty() {
        return Err("Input is empty".to_string());
    }

    let (item_type, title, content, tags, related_project) =
        match classify_capture_with_ai(&input).await {
            Ok(ai) => {
                let validated_type = match ai.item_type.as_str() {
                    "task" | "idea" | "question" | "note"
                    | "project_note" | "reminder_candidate" => ai.item_type,
                    _ => "note".to_string(),
                };
                (validated_type, ai.title, ai.content, ai.tags, ai.related_project)
            }
            Err(_) => {
                let (t, clean_title) = classify_capture_rule_based(&input);
                (t.to_string(), clean_title.clone(), clean_title, None, None)
            }
        };

    let item_id = insert_item(
        &item_type,
        &title,
        &content,
        tags.as_deref(),
        related_project.as_deref(),
    )?;

    let type_label = match item_type.as_str() {
        "task" => "task",
        "idea" => "idea",
        "question" => "question",
        "project_note" => "project note",
        "reminder_candidate" => "reminder",
        _ => "note",
    };

    Ok(CaptureResult {
        item_id,
        item_type,
        title: title.clone(),
        message: format!("Saved {} \u{2192} \u{201c}{}\u{201d}", type_label, title),
    })
}

#[tauri::command]
fn get_recent_items(limit: Option<usize>) -> Result<Vec<Item>, String> {
    list_recent_items(limit.unwrap_or(8))
}

#[tauri::command]
fn mark_item_done(item_id: String) -> Result<(), String> {
    let _ = ensure_items_table();
    let conn = Connection::open(DB_PATH).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE items SET status = 'done', updated_at = datetime('now') WHERE id = ?1",
        rusqlite::params![item_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn promote_item_to_reminder(item_id: String) -> Result<CommandResult, String> {
    let _ = ensure_items_table();
    let conn = Connection::open(DB_PATH).map_err(|e| e.to_string())?;
    let (title, content) = conn
        .query_row(
            "SELECT title, content FROM items WHERE id = ?1",
            rusqlite::params![item_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|e| format!("Item not found: {}", e))?;
    let reminder_title = if !title.trim().is_empty() { title } else { content };
    insert_reminder(&reminder_title)?;
    Ok(CommandResult {
        action: "add_reminder".to_string(),
        message: format!(
            "Added to reminders \u{2192} \u{201c}{}\u{201d}",
            reminder_title
        ),
    })
}

#[tauri::command]
fn set_focus_by_name(name: String) -> Result<CommandResult, String> {
    let projects = scan_active_projects();
    match extract_best_project_match(&name, &projects) {
        Some(p) => match write_focus(&p.name, &p.area, &p.folder_path, p.file_path.clone()) {
            Ok(_) => Ok(CommandResult {
                action: "set_focus".to_string(),
                message: format!("Focus set to {}.", p.name),
            }),
            Err(e) => Err(e),
        },
        None => Ok(CommandResult {
            action: "unknown".to_string(),
            message: format!(
                "No project found matching \u{201c}{}\u{201d}.",
                name
            ),
        }),
    }
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

    // Hard-prefix routing — checked FIRST, each branch returns immediately.
    // classify_intent is never called for any of these prefixes.
    let lower = input.to_lowercase();

    if lower.starts_with("save project:") {
        println!("ROUTE: save_project (software)");
        let raw = input["save project:".len()..].trim().to_string();
        let raw = if raw.is_empty() { input.clone() } else { raw };
        println!("DEBUG save_project: calling reformat_project with raw={:?}", &raw[..raw.len().min(80)]);
        return match reformat_project(&raw, "software").await {
            Ok(mut entry) => {
                // Force category to software regardless of what the AI guessed
                entry.category = "software".to_string();
                println!("DEBUG save_project: reformat_project returned title={:?} category={:?}", entry.title, entry.category);
                println!("DEBUG save_project: writing to path={:?}", SW_FINISHED);
                match append_finished_project(&entry) {
                    Ok(label) => {
                        let msg = format!("Saved \u{201c}{}\u{201d} to {}.", entry.title, label);
                        println!("DEBUG save_project: success message={:?}", msg);
                        Ok(CommandResult { action: "save_project".to_string(), message: msg })
                    }
                    Err(e) => Ok(CommandResult { action: "unknown".to_string(), message: format!("Failed to save project: {}", e) }),
                }
            }
            Err(e) => Ok(CommandResult { action: "unknown".to_string(), message: format!("Failed to reformat project: {}", e) }),
        };
    }

    if lower.starts_with("save research project:") {
        println!("ROUTE: save_project (research)");
        let raw = input["save research project:".len()..].trim().to_string();
        let raw = if raw.is_empty() { input.clone() } else { raw };
        println!("DEBUG save_project: calling reformat_project with raw={:?}", &raw[..raw.len().min(80)]);
        return match reformat_project(&raw, "research").await {
            Ok(mut entry) => {
                // Force category to research regardless of what the AI guessed
                entry.category = "research".to_string();
                println!("DEBUG save_project: reformat_project returned title={:?} category={:?}", entry.title, entry.category);
                println!("DEBUG save_project: writing to path={:?}", RW_FINISHED);
                match append_finished_project(&entry) {
                    Ok(label) => {
                        let msg = format!("Saved research project \u{201c}{}\u{201d} to {}.", entry.title, label);
                        println!("DEBUG save_project: success message={:?}", msg);
                        Ok(CommandResult { action: "save_project".to_string(), message: msg })
                    }
                    Err(e) => Ok(CommandResult { action: "unknown".to_string(), message: format!("Failed to save project: {}", e) }),
                }
            }
            Err(e) => Ok(CommandResult { action: "unknown".to_string(), message: format!("Failed to reformat project: {}", e) }),
        };
    }
    if lower.starts_with("idea:") {
        println!("ROUTE: capture_idea");
        let text = input["idea:".len()..].trim().to_string();
        let text = if text.is_empty() { input.clone() } else { text };
        return match reformat_idea(&text, "software").await {
            Ok(clean) => {
                // Always insert into items table so it appears in Captured
                let _ = insert_item("idea", &clean.title, &text, None, Some(clean.project.as_str()));
                match write_idea_entry(&clean.title, &clean.project, &clean.category) {
                    Ok(_label) => Ok(CommandResult {
                        action: "capture_idea".to_string(),
                        message: format!("Saved idea \u{2192} \u{201c}{}\u{201d}", clean.title),
                    }),
                    Err(_) => Ok(CommandResult {
                        action: "capture_idea".to_string(),
                        message: format!("Saved idea \u{2192} \u{201c}{}\u{201d}", clean.title),
                    }),
                }
            },
            Err(_) => {
                let _ = insert_item("idea", &text, &text, None, None);
                let _ = write_idea_entry(&text, "Captured idea", "software");
                Ok(CommandResult {
                    action: "capture_idea".to_string(),
                    message: format!("Saved idea \u{2192} \u{201c}{}\u{201d}", text),
                })
            },
        };
    }
    if lower.starts_with("working on ") {
        println!("ROUTE: log_activity (working on)");
        let label = input["working on ".len()..].trim().to_string();
        return match log_activity_entry(&label) {
            Ok(_) => Ok(CommandResult {
                action: "log_activity".to_string(),
                message: format!("Logged: {}.", label),
            }),
            Err(e) => Ok(CommandResult { action: "unknown".to_string(), message: format!("Failed to log: {}", e) }),
        };
    }
    if lower.starts_with("doing ") {
        println!("ROUTE: log_activity (doing)");
        let label = input["doing ".len()..].trim().to_string();
        return match log_activity_entry(&label) {
            Ok(_) => Ok(CommandResult {
                action: "log_activity".to_string(),
                message: format!("Logged: {}.", label),
            }),
            Err(e) => Ok(CommandResult { action: "unknown".to_string(), message: format!("Failed to log: {}", e) }),
        };
    }
    // Hard-route capture prefixes to items storage
    if lower.starts_with("task:") {
        let raw = input["task:".len()..].trim().to_string();
        let title = if raw.is_empty() { input.clone() } else { raw };
        return match insert_item("task", &title, &title, None, None) {
            Ok(_) => Ok(CommandResult {
                action: "capture_task".to_string(),
                message: format!("Saved task \u{2192} \u{201c}{}\u{201d}", title),
            }),
            Err(e) => Ok(CommandResult { action: "unknown".to_string(), message: format!("Failed to save: {}", e) }),
        };
    }
    if lower.starts_with("question:") {
        let raw = input["question:".len()..].trim().to_string();
        let title = if raw.is_empty() { input.clone() } else { raw };
        return match insert_item("question", &title, &title, None, None) {
            Ok(_) => Ok(CommandResult {
                action: "capture_question".to_string(),
                message: format!("Saved question \u{2192} \u{201c}{}\u{201d}", title),
            }),
            Err(e) => Ok(CommandResult { action: "unknown".to_string(), message: format!("Failed to save: {}", e) }),
        };
    }
    if lower.starts_with("note:") {
        let raw = input["note:".len()..].trim().to_string();
        let title = if raw.is_empty() { input.clone() } else { raw };
        return match insert_item("note", &title, &title, None, None) {
            Ok(_) => Ok(CommandResult {
                action: "capture_note".to_string(),
                message: format!("Saved note \u{2192} \u{201c}{}\u{201d}", title),
            }),
            Err(e) => Ok(CommandResult { action: "unknown".to_string(), message: format!("Failed to save: {}", e) }),
        };
    }
    if lower.starts_with("project:") {
        let raw = input["project:".len()..].trim().to_string();
        let title = if raw.is_empty() { input.clone() } else { raw };
        return match insert_item("project_note", &title, &title, None, None) {
            Ok(_) => Ok(CommandResult {
                action: "capture_project_note".to_string(),
                message: format!("Saved project note \u{2192} \u{201c}{}\u{201d}", title),
            }),
            Err(e) => Ok(CommandResult { action: "unknown".to_string(), message: format!("Failed to save: {}", e) }),
        };
    }
    if lower.starts_with("reminder:") {
        let raw = input["reminder:".len()..].trim().to_string();
        let title = if raw.is_empty() { input.clone() } else { raw };
        return match insert_item("reminder_candidate", &title, &title, None, None) {
            Ok(_) => Ok(CommandResult {
                action: "capture_reminder_candidate".to_string(),
                message: format!("Saved reminder \u{2192} \u{201c}{}\u{201d}", title),
            }),
            Err(e) => Ok(CommandResult { action: "unknown".to_string(), message: format!("Failed to save: {}", e) }),
        };
    }

    // Hard-route all focus-setting patterns before hitting classify_intent
    let focus_hint: Option<&str> = if lower.starts_with("focus on ") {
        Some(&input["focus on ".len()..])
    } else if lower.starts_with("my focus is ") {
        Some(&input["my focus is ".len()..])
    } else if lower.starts_with("today my focus is ") {
        Some(&input["today my focus is ".len()..])
    } else if lower.starts_with("i want to focus on ") {
        Some(&input["i want to focus on ".len()..])
    } else {
        None
    };

    if let Some(hint_raw) = focus_hint {
        println!("ROUTE: set_focus (hard prefix)");
        println!("DEBUG set_focus: raw_input={:?}", input);
        let projects = scan_active_projects();
        let matched = extract_best_project_match(hint_raw, &projects);
        return match matched {
            Some(p) => match write_focus(&p.name, &p.area, &p.folder_path, p.file_path.clone()) {
                Ok(_) => Ok(CommandResult {
                    action: "set_focus".to_string(),
                    message: format!("Focus set to {}.", p.name),
                }),
                Err(e) => Ok(CommandResult { action: "unknown".to_string(), message: format!("Failed to set focus: {}", e) }),
            },
            None => Ok(CommandResult {
                action: "unknown".to_string(),
                message: format!("No project matching \"{}\". Available: {}",
                    hint_raw.trim(),
                    projects.iter().take(5).map(|p| p.name.as_str()).collect::<Vec<_>>().join(", ")
                ),
            }),
        };
    }

    println!("ROUTE: classify_intent for input: {:?}", &input[..input.len().min(60)]);
    let classified = match classify_intent(&input).await {
        Ok(c) => c,
        Err(_e) => {
            return Ok(CommandResult {
                action: "unknown".to_string(),
                message: "AI request failed. The app loaded, but the network call to OpenAI did not complete.".to_string(),
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
        "suggest_today" => match get_dashboard_inner().await {
            Ok(data) => {
                let msg = build_today_suggestion_ai(&data).await;
                Ok(CommandResult {
                    action: "suggest_today".to_string(),
                    message: msg,
                })
            }
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
            match reformat_idea(&text, &category).await {
                Ok(clean) => {
                    // Always insert into items table so it appears in Captured
                    let _ = insert_item("idea", &clean.title, &text, None, Some(clean.project.as_str()));
                    let _ = write_idea_entry(&clean.title, &clean.project, &clean.category);
                    Ok(CommandResult {
                        action: "capture_idea".to_string(),
                        message: format!("Saved idea \u{2192} \u{201c}{}\u{201d}", clean.title),
                    })
                },
                Err(_) => {
                    let _ = insert_item("idea", &text, &text, None, None);
                    let _ = write_idea_entry(&text, "Captured idea", &category);
                    Ok(CommandResult {
                        action: "capture_idea".to_string(),
                        message: format!("Saved idea \u{2192} \u{201c}{}\u{201d}", text),
                    })
                },
            }
        }
        "set_focus" => {
            let hint = classified.focus_project.unwrap_or_default();
            println!("DEBUG set_focus (classified): raw_input={:?} focus_project={:?}", input, hint);
            let projects = scan_active_projects();
            // Try scored match on AI-extracted name first; fall back to full input
            let matched = extract_best_project_match(&hint, &projects)
                .or_else(|| extract_best_project_match(&input, &projects));
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
                    message: format!("No project matching \"{}\". Available: {}",
                        hint,
                        projects.iter().take(5).map(|p| p.name.as_str()).collect::<Vec<_>>().join(", ")
                    ),
                }),
            }
        }
        "capture_task" | "capture_note" | "capture_question"
        | "capture_project_note" | "capture_reminder_candidate" => {
            let item_type = match classified.intent.as_str() {
                "capture_task" => "task",
                "capture_question" => "question",
                "capture_project_note" => "project_note",
                "capture_reminder_candidate" => "reminder_candidate",
                _ => "note",
            };
            let type_label = match item_type {
                "task" => "task",
                "question" => "question",
                "project_note" => "project note",
                "reminder_candidate" => "reminder",
                _ => "note",
            };
            match insert_item(item_type, &input, &input, None, None) {
                Ok(_) => Ok(CommandResult {
                    action: format!("capture_{}", item_type),
                    message: format!("Saved {} \u{2192} \u{201c}{}\u{201d}", type_label, input),
                }),
                Err(e) => Ok(CommandResult {
                    action: "unknown".to_string(),
                    message: format!("Failed to save: {}", e),
                }),
            }
        }
        "refresh" => Ok(CommandResult {
            action: "refresh".to_string(),
            message: "Refreshing dashboard.".to_string(),
        }),
        "save_project" => {
            let raw = classified
                .project_text
                .filter(|t| !t.trim().is_empty())
                .unwrap_or_else(|| input.clone());
            let hint = classified
                .project_category
                .unwrap_or_else(|| "software".to_string());
            match reformat_project(&raw, &hint).await {
                Ok(entry) => match append_finished_project(&entry) {
                    Ok(label) => Ok(CommandResult {
                        action: "save_project".to_string(),
                        message: format!("Saved \u{201c}{}\u{201d} to {}.", entry.title, label),
                    }),
                    Err(e) => Ok(CommandResult {
                        action: "unknown".to_string(),
                        message: format!("Failed to save project: {}", e),
                    }),
                },
                Err(e) => Ok(CommandResult {
                    action: "unknown".to_string(),
                    message: format!("Failed to reformat project: {}", e),
                }),
            }
        }
        _ => match get_dashboard_inner().await {
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
    startup_init();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_dashboard_data,
            add_reminder,
            open_path,
            set_focus,
            mark_reminder_done,
            process_command,
            capture_input,
            get_recent_items,
            mark_item_done,
            promote_item_to_reminder,
            set_focus_by_name
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
