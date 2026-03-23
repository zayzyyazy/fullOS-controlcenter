# Control Center

A personal OS dashboard for macOS — live activity tracking, focused work, reminders, and an AI command bar, all in one place.

![Dashboard](assets/screenshots/dashboard.png)

---

## What it is

Control Center is the evolution of [activity-intelligence](https://github.com/zayzyyazy/activity-intelligence) — taken from a scripts-and-data project into a full native desktop app.

It surfaces everything you need to stay oriented throughout the day: what you're working on, what's pending, how you've been spending your time, and an AI command bar you can type anything into.

Everything runs locally. No cloud sync, no accounts, no telemetry. It connects to your file system, a SQLite database, and ActivityWatch.

---

## How it got here

This project didn't start as an app. It started as a frustration: I wanted AI in my daily life in a way that felt immediate and lightweight — not opening ChatGPT manually every time I had a thought.

**Stage 0 — ChatGPT Shortcuts**
The first layer was a system of iOS Shortcuts connected to ChatGPT. Simple but useful: generate project ideas, save finished work, capture quotes, turn messy thoughts into structured prompts, plan the day, draft emails. Over time these grew into a real ecosystem — four main shortcut groups covering Projects, Capture/Develop, Research/Verify, and Plan/Decide. Not a product yet, but a personal AI infrastructure embedded in my daily workflow.

**[Activity Intelligence v1](https://github.com/zayzyyazy/activity-intelligence) — shortcut-driven daily tracking**
I wanted AI to track my real behavior, not just help me think. So I built logging flows where I could speak or type casually from my phone and have the system interpret it — extracting whether something was a task, a reminder, an end-of-activity entry, or a neutral log. A day-check shortcut would surface what was pending and what I'd been doing. This stage taught me what actually mattered: frictionless input, AI that cleans messy text into structure, and a feedback loop where what I logged actually came back to me.

**Activity Intelligence v2 — local scripts and real persistence**
The shortcut logic was getting fragile — too dependent on live ChatGPT sessions, too messy to maintain. So I moved the important pieces into local scripts and a SQLite database. Reminders became real: add from phone, appear in the system, mark done via natural language summary. I connected it to [ActivityWatch](https://activitywatch.net) so app usage was reflected in the system automatically. The project went from "ask AI something" to "a local intelligence layer that knows what I'm doing, what I said I'd do, and what I probably need to do next." But there was still no visual home. The system existed; it didn't feel like a place.

**Control Center — a real interface, then a decision engine**
That's why I built this. I wanted a desktop app as the front door to the whole system. It started as a visual shell (Today, Projects, Ideas) wired to the existing SQLite database. Then I kept pushing: connected it to ActivityWatch for live data, added the Now strip, built the AI command bar, added Working On detection, made everything clickable, fixed a long list of production issues that separated "works in terminal" from "survives real use."

Then the more meaningful upgrade: turning it from a dashboard into a decision engine. The Next Action system replaced generic summaries with structured `NEXT_ACTION / WHY / AFTER` output. A unified capture layer let everything I typed — tasks, questions, ideas, notes — flow into SQLite and surface back through a Captured section. The AI now answers from reminders, activity, *and* recent captured items combined.

The final piece was reconnecting Control Center back to the original shortcuts. Reminders created in the app now appear in the phone shortcut day-check, and vice versa. The app and the shortcuts are no longer separate experiments — they're two interfaces to the same underlying system. One is the desktop command center; the other is the fast mobile layer.

What started as a few convenience automations became a personal operating system.

---

## Features

| Area | What it does |
|---|---|
| **Now** | Live strip showing the app you're currently in, updated every 7s |
| **Today / Focus** | Pending reminders ordered by priority — click to open the project folder |
| **Working On** | Auto-detected recently modified projects with open / file / focus actions |
| **Activity** | Top apps by time today, pulled live from ActivityWatch |
| **Projects** | Count of finished software and research projects, clickable to source files |
| **Ideas** | Most recent captured ideas from your notes, clickable to open the file |
| **AI Command Bar** | Natural language input — add reminders, capture ideas, set focus, summarize your day |

---

## AI Command Bar

Type anything in plain English and press Enter:

```
remind me to clean the desktop
```
> "Clean the desktop" added to reminders.

```
what should I do today?
```
> 2 pending reminders: fix the export bug, review PR. Top app today: Xcode (47m).

```
idea: build a pomodoro timer for the menubar
```
> Idea saved to software notes.

```
I finished planning finances and calling the dentist
```
> Matched 2 reminders. Marked done: "Plan finances", "Call dentist".

---

## Tech stack

- [Tauri v2](https://tauri.app) — Rust backend + native macOS shell
- [React 18](https://react.dev) + TypeScript — UI
- [Vite](https://vitejs.dev) — dev server and bundler
- [SQLite via rusqlite](https://github.com/rusqlite/rusqlite) — reminders and activity log
- [ActivityWatch](https://activitywatch.net) — local app usage tracking
- [OpenAI API (GPT-4o-mini)](https://platform.openai.com) — command intent classification

---

## Setup

```bash
git clone https://github.com/zayzyyazy/fullOS-controlcenter.git
cd fullOS-controlcenter
npm install
```

Set your OpenAI API key:

```bash
export OPENAI_API_KEY=sk-...
```

> The key is read at runtime by the Rust backend and is never written to disk or committed to the repo. The AI command bar won't work without it, but the rest of the dashboard will.

Run in dev mode:

```bash
npm run tauri dev
```

Build a release `.app`:

```bash
npm run tauri build
```

The `.app` will be at `src-tauri/target/release/bundle/macos/`.

---

## Configuration

Before running, update the hardcoded paths in `src-tauri/src/lib.rs` to match your machine:

```rust
const DB_PATH: &str = "/your/path/to/activity.db";
const FOCUS_FILE: &str = "/your/path/to/focus.json";
const SW_FINISHED: &str = "/your/path/to/finishedprojects.txt";
```

A config file approach is on the roadmap.

---

## Requirements

- macOS (arm64 or x86_64)
- [ActivityWatch](https://activitywatch.net) running on port 5600
- [Rust + Cargo](https://rustup.rs)
- Node.js 18+
- OpenAI API key

ActivityWatch must be running for the Now strip and Activity card to show live data. The rest of the dashboard works without it.

---

## Roadmap

This is v1. Currently building out:

- [ ] **Idea recommendations** — suggest what to work on based on your current pace and patterns
- [ ] **Auto activity sync** — automatic syncing without manual log commands
- [ ] **Expanded AI intelligence** — more context-aware responses and richer daily insights
- [ ] Config file for paths (no more hardcoded constants)
- [ ] Global quick-capture hotkey
- [ ] Richer activity breakdown (hourly chart)
- [ ] Weekly summary / export
- [ ] Notifications for overdue reminders

---

## Notes

- `focus.json` is local runtime state and is gitignored. It's created automatically the first time you set focus.
- `src-tauri/target/` is gitignored. The first build takes a few minutes as Cargo compiles dependencies.
- All data stays on your machine. The only outbound network call is to OpenAI when you use the command bar.
