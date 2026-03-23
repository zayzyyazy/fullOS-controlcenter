import React, { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface FocusItem {
  id: string;
  title: string;
}
interface ActiveProject {
  name: string;
  area: string;
  folder_path: string;
  recent_file: string | null;
  file_path: string | null;
  modified_secs_ago: number;
}
interface CurrentFocus {
  name: string;
  area: string;
  folder_path: string;
  file_path: string | null;
  updated_at: number;
}
interface NowEvent {
  app: string;
  title: string | null;
  secs_ago: number;
}
interface DashboardData {
  focus: FocusItem[];
  focus_extra: number;
  projects_software_count: number;
  projects_research_count: number;
  ideas_software: string[];
  ideas_research: string[];
  working_on: ActiveProject[];
  current_focus: CurrentFocus | null;
  now_event: NowEvent | null;
  insight: string;
}

interface CapturedItem {
  id: string;
  item_type: string;
  title: string;
  content: string;
  status: string;
  tags: string | null;
  related_project: string | null;
  created_at: string;
  updated_at: string;
}

type LoadState = "loading" | "ok" | "error";

const s = {
  shell: {
    minHeight: "100vh",
    backgroundColor: "#0a0a0a",
    padding: "2rem 2.5rem",
    display: "flex",
    flexDirection: "column",
    gap: "2rem",
  } as React.CSSProperties,

  headerRow: {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "flex-start",
    paddingBottom: "0.5rem",
  } as React.CSSProperties,

  greeting: {
    fontSize: "2rem",
    fontWeight: 700,
    color: "#f0f0f0",
    letterSpacing: "-0.03em",
    lineHeight: 1.2,
    margin: 0,
  } as React.CSSProperties,

  greetingSubtitle: {
    fontSize: "1rem",
    color: "#555555",
    fontWeight: 400,
    marginTop: "0.35rem",
    marginBottom: 0,
  } as React.CSSProperties,

  refreshBtn: {
    background: "none",
    border: "none",
    cursor: "pointer",
    fontSize: "0.75rem",
    color: "#383838",
    padding: "0.25rem 0.5rem",
    borderRadius: "6px",
    letterSpacing: "0.05em",
    marginTop: "0.25rem",
    transition: "color 0.15s",
  } as React.CSSProperties,

  grid: {
    display: "grid",
    gridTemplateColumns: "1fr 1fr",
    gap: "1.25rem",
  } as React.CSSProperties,

  card: {
    backgroundColor: "#111111",
    border: "1px solid #1e1e1e",
    borderRadius: "14px",
    padding: "1.75rem",
    display: "flex",
    flexDirection: "column",
    gap: "0.5rem",
    minHeight: "160px",
  } as React.CSSProperties,

  cardLabel: {
    fontSize: "0.7rem",
    fontWeight: 600,
    letterSpacing: "0.1em",
    textTransform: "uppercase" as const,
    color: "#404040",
  } as React.CSSProperties,

  cardTitle: {
    fontSize: "1rem",
    fontWeight: 600,
    color: "#d0d0d0",
    letterSpacing: "-0.01em",
    marginBottom: "0.25rem",
  } as React.CSSProperties,

  cardBody: {
    fontSize: "0.875rem",
    color: "#333333",
    lineHeight: 1.6,
  } as React.CSSProperties,

  cardPrimary: {
    backgroundColor: "#141414",
    border: "1px solid #2a2a2a",
    borderRadius: "14px",
    padding: "2.25rem",
    display: "flex",
    flexDirection: "column",
    gap: "0.5rem",
    minHeight: "160px",
  } as React.CSSProperties,

  cardPrimaryLabel: {
    fontSize: "0.7rem",
    fontWeight: 600,
    letterSpacing: "0.1em",
    textTransform: "uppercase" as const,
    color: "#505050",
  } as React.CSSProperties,

  cardPrimaryTitle: {
    fontSize: "1.1rem",
    fontWeight: 700,
    color: "#ebebeb",
    letterSpacing: "-0.02em",
    marginBottom: "0.25rem",
  } as React.CSSProperties,

  cardDimmed: {
    backgroundColor: "#111111",
    border: "1px solid #1e1e1e",
    borderRadius: "14px",
    padding: "1.75rem",
    display: "flex",
    flexDirection: "column",
    gap: "0.5rem",
    minHeight: "160px",
    opacity: 0.55,
  } as React.CSSProperties,

  cardError: {
    fontSize: "0.8rem",
    color: "#5a2020",
    lineHeight: 1.5,
  } as React.CSSProperties,

  list: {
    listStyle: "none",
    margin: 0,
    padding: 0,
    display: "flex",
    flexDirection: "column",
    gap: "0.4rem",
  } as React.CSSProperties,

  listItem: {
    fontSize: "0.875rem",
    color: "#909090",
    lineHeight: 1.4,
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap" as const,
  } as React.CSSProperties,

  listItemRow: {
    fontSize: "0.875rem",
    color: "#909090",
    display: "flex",
    justifyContent: "space-between",
    alignItems: "baseline",
  } as React.CSSProperties,

  dim: {
    fontSize: "0.8rem",
    color: "#3a3a3a",
    flexShrink: 0,
    marginLeft: "0.5rem",
  } as React.CSSProperties,

  extra: {
    fontSize: "0.75rem",
    color: "#3a3a3a",
    marginTop: "0.1rem",
  } as React.CSSProperties,

  addBtn: {
    background: "none",
    border: "none",
    cursor: "pointer",
    fontSize: "0.75rem",
    color: "#2e2e2e",
    padding: 0,
    marginTop: "0.4rem",
    textAlign: "left" as const,
    letterSpacing: "0.04em",
  } as React.CSSProperties,

  inlineRow: {
    display: "flex",
    alignItems: "center",
    gap: "0.5rem",
    marginTop: "0.4rem",
  } as React.CSSProperties,

  inlineInput: {
    flex: 1,
    background: "#181818",
    border: "1px solid #2a2a2a",
    borderRadius: "7px",
    padding: "0.4rem 0.6rem",
    fontSize: "0.85rem",
    color: "#c0c0c0",
    outline: "none",
    caretColor: "#555",
  } as React.CSSProperties,

  inlineSubmit: {
    background: "none",
    border: "none",
    cursor: "pointer",
    fontSize: "0.75rem",
    color: "#404040",
    padding: "0.3rem 0.5rem",
    borderRadius: "6px",
    whiteSpace: "nowrap" as const,
  } as React.CSSProperties,

  doneBtn: {
    background: "none",
    border: "none",
    cursor: "pointer",
    fontSize: "0.75rem",
    color: "#2e2e2e",
    padding: "0 0 0 0.5rem",
    flexShrink: 0,
    lineHeight: 1,
    transition: "color 0.15s",
  } as React.CSSProperties,

  focusBlock: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.1rem",
    paddingBottom: "0.65rem",
    marginBottom: "0.25rem",
    borderBottom: "1px solid #1a1a1a",
  } as React.CSSProperties,

  focusLabel: {
    fontSize: "0.65rem",
    fontWeight: 600,
    letterSpacing: "0.1em",
    textTransform: "uppercase" as const,
    color: "#2e2e2e",
  } as React.CSSProperties,

  focusName: {
    fontSize: "0.875rem",
    color: "#808080",
    marginTop: "0.1rem",
  } as React.CSSProperties,

  focusMeta: {
    fontSize: "0.72rem",
    color: "#3a3a3a",
  } as React.CSSProperties,

  nextActionCard: {
    backgroundColor: "#131313",
    border: "1px solid #202020",
    borderRadius: "14px",
    padding: "2rem 2.25rem",
    cursor: "pointer",
    transition: "opacity 0.15s",
  } as React.CSSProperties,

  captureActionBtn: {
    background: "#161616",
    border: "1px solid #242424",
    borderRadius: "6px",
    padding: "0.25rem 0.65rem",
    fontSize: "0.7rem",
    color: "#686868",
    cursor: "pointer",
    letterSpacing: "0.03em",
    transition: "color 0.12s, border-color 0.12s",
    whiteSpace: "nowrap" as const,
  } as React.CSSProperties,

  workingOnBtn: {
    background: "none",
    border: "none",
    cursor: "pointer",
    fontSize: "0.68rem",
    color: "#2e2e2e",
    padding: 0,
    letterSpacing: "0.04em",
    textAlign: "left" as const,
    transition: "color 0.15s",
  } as React.CSSProperties,

  commandBar: {
    backgroundColor: "#111111",
    border: "1px solid #1e1e1e",
    borderRadius: "14px",
    padding: "1.25rem 1.75rem",
    display: "flex",
    alignItems: "flex-start",
    gap: "1rem",
  } as React.CSSProperties,

  commandLabel: {
    fontSize: "0.7rem",
    fontWeight: 600,
    letterSpacing: "0.1em",
    textTransform: "uppercase" as const,
    color: "#404040",
    whiteSpace: "nowrap" as const,
    paddingTop: "0.15rem",
  } as React.CSSProperties,

  commandInner: {
    flex: 1,
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.5rem",
  } as React.CSSProperties,

  commandInput: {
    background: "transparent",
    border: "none",
    outline: "none",
    fontSize: "0.9rem",
    color: "#c0c0c0",
    caretColor: "#555555",
    width: "100%",
  } as React.CSSProperties,

  commandResponse: {
    fontSize: "0.8rem",
    color: "#555555",
    lineHeight: 1.5,
  } as React.CSSProperties,
};

const CAPTURED_FILTERS: { key: string; label: string }[] = [
  { key: "all", label: "All" },
  { key: "task", label: "Tasks" },
  { key: "idea", label: "Ideas" },
  { key: "question", label: "Questions" },
  { key: "note", label: "Notes" },
  { key: "project_note", label: "Projects" },
  { key: "reminder_candidate", label: "Reminders" },
];

function trunc(str: string, n: number) {
  return str.length > n ? str.slice(0, n) + "…" : str;
}

function ago(secs: number): string {
  if (secs < 120) return "just now";
  if (secs < 3600) return `${Math.floor(secs / 60)}m ago`;
  if (secs < 86400) return `${Math.floor(secs / 3600)}h ago`;
  return `${Math.floor(secs / 86400)}d ago`;
}

function agoNow(secs: number): string {
  if (secs < 60) return `${secs}s ago`;
  if (secs < 3600) return `${Math.floor(secs / 60)}m ago`;
  return `${Math.floor(secs / 3600)}h ago`;
}

function itemTypeLabel(t: string): string {
  switch (t) {
    case "task": return "task";
    case "idea": return "idea";
    case "question": return "question";
    case "project_note": return "project";
    case "reminder_candidate": return "reminder";
    default: return "note";
  }
}

function itemTypeColor(t: string): string {
  switch (t) {
    case "task": return "#2a5a8a";
    case "idea": return "#5a4a9a";
    case "question": return "#7a5a1a";
    case "project_note": return "#1a5a3a";
    case "reminder_candidate": return "#6a2a2a";
    default: return "#404040";
  }
}

function timeAgoFromSqlite(ts: string): string {
  try {
    const d = new Date(ts.replace(" ", "T") + "Z");
    const secs = Math.floor((Date.now() - d.getTime()) / 1000);
    return ago(Math.max(0, secs));
  } catch {
    return "";
  }
}

function Loading() {
  return <p style={s.cardBody}>Loading…</p>;
}

function Empty({ text }: { text: string }) {
  return <p style={s.cardBody}>{text}</p>;
}

function CardError({ msg }: { msg: string }) {
  return <p style={s.cardError}>Could not load — {msg}</p>;
}


export default function Home() {
  const [data, setData] = useState<DashboardData | null>(null);
  const [loadState, setLoadState] = useState<LoadState>("loading");
  const [error, setError] = useState("");

  const [showAdd, setShowAdd] = useState(false);
  const [reminderText, setReminderText] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const [cmdInput, setCmdInput] = useState("");
  const [cmdLoading, setCmdLoading] = useState(false);
  const [cmdResponse, setCmdResponse] = useState<{ action: string; message: string } | null>(null);
  const [cmdFlash, setCmdFlash] = useState(false);
  const cmdInputRef = useRef<HTMLInputElement>(null);
  const [doneIds, setDoneIds] = useState<Set<string>>(new Set());
  const [recentItems, setRecentItems] = useState<CapturedItem[]>([]);
  const [capturedFilter, setCapturedFilter] = useState("all");
  const [selectedItemId, setSelectedItemId] = useState<string | null>(null);
  const [itemActionMsg, setItemActionMsg] = useState("");

  const fetchRecentItems = useCallback(() => {
    invoke<CapturedItem[]>("get_recent_items", { limit: 20 })
      .then(setRecentItems)
      .catch(() => {});
  }, []);

  const filteredItems = capturedFilter === "all"
    ? recentItems
    : recentItems.filter(item => item.item_type === capturedFilter);

  const handleMarkDone = async (itemId: string) => {
    try {
      await invoke("mark_item_done", { itemId });
      setSelectedItemId(null);
      setItemActionMsg("");
      fetchRecentItems();
    } catch {
      setItemActionMsg("Failed to mark done.");
    }
  };

  const handlePromoteToReminder = async (itemId: string) => {
    try {
      const res = await invoke<{ action: string; message: string }>("promote_item_to_reminder", { itemId });
      setItemActionMsg(res.message);
      fetchRecentItems();
      refresh();
    } catch {
      setItemActionMsg("Failed to promote.");
    }
  };

  const handleSetFocusByName = async (name: string) => {
    try {
      const res = await invoke<{ action: string; message: string }>("set_focus_by_name", { name });
      setItemActionMsg(res.message);
      refresh();
    } catch {
      setItemActionMsg("Failed to set focus.");
    }
  };

  const refresh = useCallback(() => {
    setLoadState("loading");
    invoke<DashboardData>("get_dashboard_data")
      .then((d) => {
        setData(d);
        setLoadState("ok");
        setError("");
      })
      .catch((e) => {
        setError(String(e));
        setLoadState("error");
      });
  }, []);

  const silentRefresh = useCallback(() => {
    invoke<DashboardData>("get_dashboard_data")
      .then((d) => {
        setData(d);
        setLoadState((s) => (s === "error" ? "ok" : s));
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    refresh();
    fetchRecentItems();
  }, [refresh, fetchRecentItems]);

  useEffect(() => {
    const id = setInterval(silentRefresh, 7000);
    return () => clearInterval(id);
  }, [silentRefresh]);

  useEffect(() => {
    const onFocus = () => silentRefresh();
    const onVisible = () => { if (document.visibilityState === "visible") silentRefresh(); };
    window.addEventListener("focus", onFocus);
    document.addEventListener("visibilitychange", onVisible);
    return () => {
      window.removeEventListener("focus", onFocus);
      document.removeEventListener("visibilitychange", onVisible);
    };
  }, [silentRefresh]);

  useEffect(() => {
    if (showAdd) {
      setTimeout(() => inputRef.current?.focus(), 0);
    }
  }, [showAdd]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.metaKey && e.shiftKey && e.code === "Space") {
        e.preventDefault();
        setCmdInput("");
        setCmdResponse(null);
        setCmdFlash(true);
        setTimeout(() => setCmdFlash(false), 500);
        setTimeout(() => cmdInputRef.current?.focus(), 0);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  const handleAddReminder = async () => {
    const title = reminderText.trim();
    if (!title || submitting) return;
    setSubmitting(true);
    try {
      await invoke("add_reminder", { title });
      setReminderText("");
      setShowAdd(false);
      refresh();
    } catch (e) {
      console.error(e);
    } finally {
      setSubmitting(false);
    }
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") handleAddReminder();
    if (e.key === "Escape") {
      setShowAdd(false);
      setReminderText("");
    }
  };

  const handleDone = async (id: string) => {
    setDoneIds((prev) => new Set(prev).add(id));
    try {
      await invoke("mark_reminder_done", { id });
      refresh();
    } catch (e) {
      setDoneIds((prev) => { const next = new Set(prev); next.delete(id); return next; });
      console.error(e);
    }
  };

  const handleCmdKeyDown = async (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key !== "Enter") return;
    const input = cmdInput.trim();
    if (!input || cmdLoading) return;
    setCmdLoading(true);
    setCmdResponse(null);
    try {
      const result = await invoke<{ action: string; message: string }>("process_command", { input });
      setCmdResponse(result);
      if (["add_reminder", "complete_from_summary", "log_activity", "set_focus", "refresh", "save_project"].includes(result.action)) {
        refresh();
      }
      if (result.action.startsWith("capture_")) {
        fetchRecentItems();
      }
      if (result.action !== "unknown") {
        setCmdInput("");
      }
    } catch (e) {
      setCmdResponse({ action: "unknown", message: String(e) });
    } finally {
      setCmdLoading(false);
    }
  };

  const isLoading = loadState === "loading";
  const isError = loadState === "error";

  const continueItem = (() => {
    if (!data || loadState !== "ok") return null;
    if (data.current_focus) {
      return {
        name: data.current_focus.name,
        area: data.current_focus.area,
        path: data.current_focus.file_path ?? data.current_focus.folder_path,
        secsAgo: Math.floor(Date.now() / 1000) - data.current_focus.updated_at,
      };
    }
    if (data.working_on.length > 0) {
      const w = data.working_on[0];
      return {
        name: w.name,
        area: w.area,
        path: w.file_path ?? w.folder_path,
        secsAgo: w.modified_secs_ago,
      };
    }
    return null;
  })();

  return (
    <div style={s.shell}>
      <header style={s.headerRow}>
        <div>
          <h1 style={s.greeting}>Hi Zay</h1>
          <p style={s.greetingSubtitle}>What are we working on today?</p>
        </div>
        <button
          style={s.refreshBtn}
          onClick={refresh}
          disabled={isLoading}
          title="Refresh"
        >
          {isLoading ? "…" : "↻ Refresh"}
        </button>
      </header>

      {continueItem && (
        <div
          style={s.nextActionCard}
          onClick={() => invoke("open_path", { path: continueItem.path })}
          onMouseEnter={e => (e.currentTarget.style.opacity = "0.82")}
          onMouseLeave={e => (e.currentTarget.style.opacity = "1")}
        >
          <span style={{ fontSize: "0.65rem", fontWeight: 600, letterSpacing: "0.12em", textTransform: "uppercase" as const, color: "#363636" }}>Next Action</span>
          <div style={{ fontSize: "1.7rem", fontWeight: 700, color: "#f2f2f2", marginTop: "0.3rem", letterSpacing: "-0.03em", lineHeight: 1.15 }}>
            {continueItem.name}
          </div>
          <div style={{ fontSize: "0.75rem", color: "#383838", marginTop: "0.3rem" }}>
            {continueItem.area} · {ago(continueItem.secsAgo)}
          </div>
          {data?.insight && (
            <p style={{ fontSize: "0.9rem", color: "#686868", margin: "0.9rem 0 0", lineHeight: 1.6 }}>
              {data.insight}
            </p>
          )}
        </div>
      )}

      {/* Command bar — primary interaction surface */}
      <div style={{ ...s.commandBar, transition: "border-color 0.3s", borderColor: cmdFlash ? "#404040" : "#1e1e1e" }}>
        <span style={s.commandLabel}>AI</span>
        <div style={s.commandInner}>
          <input
            ref={cmdInputRef}
            style={s.commandInput}
            type="text"
            placeholder="Ask or tell your system anything…"
            value={cmdInput}
            onChange={(e) => setCmdInput(e.target.value)}
            onKeyDown={handleCmdKeyDown}
            disabled={cmdLoading}
          />
          {cmdLoading && (
            <span style={s.commandResponse}>thinking…</span>
          )}
          {cmdResponse && !cmdLoading && (() => {
            if (cmdResponse.action === 'suggest_today') {
              try {
                const d = JSON.parse(cmdResponse.message);
                if (d.__decision__) return (
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.35rem' }}>
                    <span style={{ fontSize: '0.95rem', fontWeight: 700, color: '#e0e0e0', lineHeight: 1.3 }}>
                      {d.action}
                    </span>
                    <span style={{ fontSize: '0.75rem', color: '#505050', lineHeight: 1.4 }}>
                      {d.why}
                    </span>
                    {d.after && (
                      <span style={{ fontSize: '0.68rem', color: '#303030', lineHeight: 1.4 }}>
                        → {d.after}
                      </span>
                    )}
                  </div>
                );
              } catch {}
            }
            return <span style={s.commandResponse}>{cmdResponse.message}</span>;
          })()}
        </div>
      </div>

      {/* Captured — dynamic working surface */}
      {recentItems.length > 0 && (
        <div style={{
          backgroundColor: "#111111",
          border: "1px solid #252525",
          borderRadius: "14px",
          padding: "1.75rem",
          boxShadow: "0 0 0 1px #1e1e1e",
        }}>
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "1rem" }}>
            <div style={{ display: "flex", alignItems: "baseline", gap: "0.6rem" }}>
              <span style={{
                fontSize: "0.75rem", fontWeight: 700, letterSpacing: "0.1em",
                textTransform: "uppercase" as const, color: "#505050",
              }}>Captured</span>
              <span style={{ fontSize: "0.68rem", color: "#2a2a2a" }}>
                {filteredItems.length} {filteredItems.length === 1 ? "item" : "items"}
              </span>
            </div>
            <div style={{ display: "flex", gap: "0.2rem" }}>
              {CAPTURED_FILTERS.map(f => (
                <button
                  key={f.key}
                  onClick={() => { setCapturedFilter(f.key); setSelectedItemId(null); setItemActionMsg(""); }}
                  style={{
                    background: capturedFilter === f.key ? "#1c1c1c" : "transparent",
                    border: `1px solid ${capturedFilter === f.key ? "#2d2d2d" : "transparent"}`,
                    borderRadius: "6px",
                    padding: "0.18rem 0.5rem",
                    fontSize: "0.63rem",
                    fontWeight: capturedFilter === f.key ? 600 : 400,
                    color: capturedFilter === f.key ? "#787878" : "#363636",
                    cursor: "pointer",
                    letterSpacing: "0.04em",
                  }}
                >
                  {f.label}
                </button>
              ))}
            </div>
          </div>
          {filteredItems.length === 0 ? (
            <p style={{ fontSize: "0.8rem", color: "#2e2e2e", margin: 0 }}>
              No {capturedFilter === "all" ? "captured" : capturedFilter.replace("_", " ")} items.
            </p>
          ) : (
            <div style={{ display: "flex", flexDirection: "column" }}>
              {filteredItems.map(item => {
                const isSelected = selectedItemId === item.id;
                return (
                  <div key={item.id}>
                    <div
                      style={{
                        display: "flex", alignItems: "baseline", gap: "0.6rem",
                        padding: "0.3rem 0.4rem", margin: "0 -0.4rem", borderRadius: "6px",
                        cursor: "pointer",
                        background: isSelected ? "#161616" : "transparent",
                        transition: "background 0.1s",
                      }}
                      onClick={() => { setSelectedItemId(isSelected ? null : item.id); setItemActionMsg(""); }}
                      onMouseEnter={e => { if (!isSelected) e.currentTarget.style.background = "#141414"; }}
                      onMouseLeave={e => { if (!isSelected) e.currentTarget.style.background = "transparent"; }}
                    >
                      <span style={{
                        fontSize: "0.6rem", fontWeight: 700, color: itemTypeColor(item.item_type),
                        letterSpacing: "0.08em", textTransform: "uppercase" as const, flexShrink: 0, width: "68px",
                      }}>
                        {itemTypeLabel(item.item_type)}
                      </span>
                      <span style={{
                        fontSize: "0.875rem", color: isSelected ? "#b0b0b0" : "#888888",
                        overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" as const,
                        flex: 1, fontWeight: isSelected ? 500 : 400,
                      }}>
                        {trunc(item.title, 64)}
                      </span>
                      {item.related_project && (
                        <span style={{ fontSize: "0.7rem", color: "#303030", flexShrink: 0 }}>
                          {trunc(item.related_project, 20)}
                        </span>
                      )}
                      <span style={{ fontSize: "0.68rem", color: "#2e2e2e", flexShrink: 0, marginLeft: "0.25rem" }}>
                        {timeAgoFromSqlite(item.created_at)}
                      </span>
                    </div>
                    {isSelected && (
                      <div style={{
                        marginLeft: "72px", paddingLeft: "0.75rem", paddingTop: "0.4rem",
                        paddingBottom: "0.75rem", marginBottom: "0.1rem", borderLeft: "1px solid #1e1e1e",
                      }}>
                        {item.content && item.content !== item.title && (
                          <p style={{
                            fontSize: "0.82rem", color: "#555555", lineHeight: 1.6,
                            margin: "0 0 0.65rem", whiteSpace: "pre-wrap" as const,
                          }}>
                            {item.content}
                          </p>
                        )}
                        <div style={{ display: "flex", gap: "0.4rem", flexWrap: "wrap" as const, alignItems: "center" }}>
                          {item.item_type === "task" && (
                            <button style={s.captureActionBtn} onClick={e => { e.stopPropagation(); handleMarkDone(item.id); }}>
                              ✓ Mark done
                            </button>
                          )}
                          {item.item_type === "reminder_candidate" && (
                            <button style={s.captureActionBtn} onClick={e => { e.stopPropagation(); handlePromoteToReminder(item.id); }}>
                              + Add to reminders
                            </button>
                          )}
                          {item.related_project && (
                            <button style={s.captureActionBtn} onClick={e => { e.stopPropagation(); handleSetFocusByName(item.related_project!); }}>
                              ◎ Set focus
                            </button>
                          )}
                          {(item.item_type === "note" || item.item_type === "question" || item.item_type === "idea") && (
                            <button style={s.captureActionBtn} onClick={e => { e.stopPropagation(); navigator.clipboard.writeText(item.content || item.title); setItemActionMsg("Copied."); }}>
                              ⎘ Copy
                            </button>
                          )}
                          {itemActionMsg && (
                            <span style={{ fontSize: "0.68rem", color: "#484848", marginLeft: "0.25rem" }}>{itemActionMsg}</span>
                          )}
                        </div>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>
      )}

      <div style={s.grid}>
        {/* Now — subtle activity indicator */}
        {data?.now_event && (
          <div style={{ gridColumn: "1 / -1", display: "flex", alignItems: "baseline", gap: "0.5rem", paddingBottom: "0.5rem", marginBottom: "0.25rem", borderBottom: "1px solid #141414" }}>
            <span style={{ fontSize: "0.6rem", fontWeight: 600, letterSpacing: "0.1em", textTransform: "uppercase" as const, color: "#222222" }}>Now</span>
            <span style={{ fontSize: "0.8rem", color: "#505050", fontWeight: 500 }}>{data.now_event.app}</span>
            <span style={{ fontSize: "0.7rem", color: "#2a2a2a" }}>{agoNow(data.now_event.secs_ago)}</span>
          </div>
        )}

        {/* Today / Focus — dominant, full width */}
        <div style={{ gridColumn: "1 / -1" }}>
          <div style={s.cardPrimary}>
            <span style={s.cardPrimaryLabel}>Today</span>
            <span style={s.cardPrimaryTitle}>Priorities &amp; Focus</span>
            {isLoading ? (
              <Loading />
            ) : isError ? (
              <CardError msg={error} />
            ) : (
              <>
                {data!.current_focus && (
                  <div
                    style={{ ...s.focusBlock, cursor: "pointer" }}
                    onClick={() => invoke("open_path", { path: data!.current_focus!.folder_path })}
                    title="Open focus folder"
                  >
                    <span style={s.focusLabel}>current focus</span>
                    <span style={s.focusName}>{data!.current_focus.name}</span>
                    <div style={{ display: "flex", alignItems: "center", gap: "0.75rem" }}>
                      <span style={s.focusMeta}>{data!.current_focus.area}</span>
                      {data!.current_focus.file_path && (
                        <button
                          style={{ ...s.workingOnBtn, fontSize: "0.65rem" }}
                          onClick={(e) => {
                            e.stopPropagation();
                            invoke("open_path", { path: data!.current_focus!.file_path });
                          }}
                          title="Open focus file"
                        >
                          ↗ file
                        </button>
                      )}
                    </div>
                  </div>
                )}
                {data!.focus.length === 0 && !showAdd ? (
                  !data!.current_focus && <Empty text="No pending reminders." />
                ) : (
                  <>
                    {data!.focus.length > 0 && (
                      <ul style={s.list}>
                        {data!.focus.map((item, i) => {
                          const done = doneIds.has(item.id);
                          const color = done
                            ? "#3a3a3a"
                            : i === 0
                            ? "#e0e0e0"
                            : i === 1
                            ? "#a0a0a0"
                            : "#686868";
                          const weight = i === 0 ? 600 : 400;
                          return (
                            <li
                              key={item.id}
                              style={{
                                ...s.listItemRow,
                                opacity: done ? 0.35 : 1,
                                transition: "opacity 0.2s",
                                paddingTop: i === 0 ? "0.1rem" : 0,
                              }}
                            >
                              <span
                                style={{
                                  overflow: "hidden",
                                  textOverflow: "ellipsis",
                                  whiteSpace: "nowrap",
                                  color,
                                  fontWeight: weight,
                                  fontSize: i === 0 ? "0.95rem" : "0.875rem",
                                }}
                              >
                                {trunc(item.title, 48)}
                              </span>
                              <button
                                style={s.doneBtn}
                                onClick={() => handleDone(item.id)}
                                disabled={done}
                                title="Mark done"
                              >
                                ✓
                              </button>
                            </li>
                          );
                        })}
                      </ul>
                    )}
                    {data!.focus_extra > 0 && (
                      <span style={s.extra}>+{data!.focus_extra} more</span>
                    )}
                  </>
                )}
                {showAdd ? (
                  <div style={s.inlineRow}>
                    <input
                      ref={inputRef}
                      style={s.inlineInput}
                      value={reminderText}
                      onChange={(e) => setReminderText(e.target.value)}
                      onKeyDown={onKeyDown}
                      placeholder="New reminder…"
                      disabled={submitting}
                    />
                    <button
                      style={s.inlineSubmit}
                      onClick={handleAddReminder}
                      disabled={submitting || !reminderText.trim()}
                    >
                      Add
                    </button>
                  </div>
                ) : (
                  <button style={s.addBtn} onClick={() => setShowAdd(true)}>
                    + add reminder
                  </button>
                )}
              </>
            )}
          </div>
        </div>

        {/* Working Context — compact reference panel */}
        <div style={{ gridColumn: "1 / -1" }}>
          <div style={{
            backgroundColor: "#111111",
            border: "1px solid #191919",
            borderRadius: "14px",
            padding: "1.1rem 1.75rem",
            display: "flex",
            flexDirection: "column",
            gap: "0.3rem",
            opacity: 0.6,
          }}>
            <span style={s.cardLabel}>Activity · Working Context</span>
            {isLoading ? (
              <Loading />
            ) : isError ? (
              <CardError msg={error} />
            ) : data!.working_on.length === 0 ? (
              <p style={{ fontSize: "0.8rem", color: "#333", margin: 0 }}>No recent activity.</p>
            ) : (
              <ul style={{ ...s.list, gap: "0.1rem" }}>
                {[...data!.working_on]
                  .sort((a, b) => a.modified_secs_ago - b.modified_secs_ago)
                  .slice(0, 4)
                  .map((item, i) => (
                    <li
                      key={i}
                      style={{ ...s.listItemRow, cursor: "pointer", borderRadius: "5px", padding: "0.1rem 0.3rem", margin: "0 -0.3rem", transition: "background 0.12s" }}
                      onClick={() => invoke("open_path", { path: item.file_path ?? item.folder_path })}
                      onMouseEnter={e => (e.currentTarget.style.background = "#181818")}
                      onMouseLeave={e => (e.currentTarget.style.background = "transparent")}
                    >
                      <span style={{ color: "#585858", fontWeight: 400, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", fontSize: "0.82rem" }}>
                        {trunc(item.name, 40)}
                      </span>
                      <span style={{ ...s.dim, flexShrink: 0, marginLeft: "1rem", fontSize: "0.7rem" }}>
                        {item.area} · {ago(item.modified_secs_ago)}
                      </span>
                    </li>
                  ))}
              </ul>
            )}
          </div>
        </div>

        {/* Projects — de-emphasized */}
        <div style={s.cardDimmed}>
          <span style={s.cardLabel}>Projects</span>
          <span style={{ ...s.cardTitle, fontSize: "0.9rem" }}>Finished Work</span>
          {isLoading ? (
            <Loading />
          ) : isError ? (
            <CardError msg={error} />
          ) : (
            <ul style={s.list}>
              <li
                style={{ ...s.listItemRow, cursor: "pointer" }}
                onClick={() => invoke("open_path", { path: "/Users/zay/Desktop/Software and     tools/finishedprojects.txt" })}
                title="Open software projects file"
              >
                <span>Software & tools</span>
                <span style={s.dim}>{data!.projects_software_count}</span>
              </li>
              <li
                style={{ ...s.listItemRow, cursor: "pointer" }}
                onClick={() => invoke("open_path", { path: "/Users/zay/Desktop/Research and writing/finishedprojectssearch.txt" })}
                title="Open research projects file"
              >
                <span>Research & writing</span>
                <span style={s.dim}>{data!.projects_research_count}</span>
              </li>
            </ul>
          )}
        </div>

        {/* Ideas — de-emphasized */}
        <div style={s.cardDimmed}>
          <span style={s.cardLabel}>Ideas</span>
          <span style={{ ...s.cardTitle, fontSize: "0.9rem" }}>Captured Ideas</span>
          {isLoading ? (
            <Loading />
          ) : isError ? (
            <CardError msg={error} />
          ) : data!.ideas_software.length === 0 &&
            data!.ideas_research.length === 0 ? (
            <Empty text="No ideas found." />
          ) : (
            <ul style={s.list}>
              {data!.ideas_software.map((idea, i) => (
                <li
                  key={`sw-${i}`}
                  style={{ ...s.listItem, cursor: "pointer" }}
                  onClick={() => invoke("open_path", { path: "/Users/zay/Desktop/Software and     tools/ideas.txt" })}
                  title="Open software ideas"
                >
                  {trunc(idea, 52)}
                </li>
              ))}
              {data!.ideas_research.map((idea, i) => (
                <li
                  key={`rw-${i}`}
                  style={{ ...s.listItem, cursor: "pointer" }}
                  onClick={() => invoke("open_path", { path: "/Users/zay/Desktop/Research and writing/Humans and relationships/ideas.txt" })}
                  title="Open research ideas"
                >
                  {trunc(idea, 52)}
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>

    </div>
  );
}
