# Activity Intelligence (Control Center)

A personal decision engine that turns thoughts into clear, executable actions.

## Version

v2 — Decision Engine Upgrade

## What changed

- AI now outputs ONE clear next action instead of generic suggestions
- Structured decision format: NEXT_ACTION / WHY / AFTER
- Captured system (tasks, ideas, notes, questions) fully integrated
- AI now uses real context (focus + captured items)
- Actionable UI (filters, expand, mark done, promote)
- Improved focus matching with scoring system
- Layout redesigned around execution (Next Action → Input → Captured)

## Core Idea

This is not a dashboard.

This is a system that tells you exactly what to do next.

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

- Tauri
- React
- Rust
- SQLite
- OpenAI API

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

Run in dev mode:

```bash
npm run tauri dev
```

Build a release `.app`:

```bash
npm run tauri build
```

The `.app` will be at `src-tauri/target/release/bundle/macos/`.

## Requirements

- macOS (arm64 or x86_64)
- [ActivityWatch](https://activitywatch.net) running on port 5600
- Rust + Cargo
- Node.js 18+
- OpenAI API key

## Status

Actively evolving — moving toward full personal operating system
