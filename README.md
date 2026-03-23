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

## Stack

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
