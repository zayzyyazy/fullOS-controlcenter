import React, { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface FocusItem {
  id: string;
  title: string;
}
interface AppUsage {
  app: string;
  minutes: number;
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
  activity: AppUsage[];
  projects_software_count: number;
  projects_research_count: number;
  ideas_software: string[];
  ideas_research: string[];
  working_on: ActiveProject[];
  current_focus: CurrentFocus | null;
  now_event: NowEvent | null;
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

  workingOnName: {
    fontSize: "0.875rem",
    color: "#909090",
    lineHeight: 1.3,
  } as React.CSSProperties,

  workingOnMeta: {
    fontSize: "0.72rem",
    color: "#383838",
    lineHeight: 1.3,
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

  workingOnActions: {
    display: "flex",
    gap: "0.6rem",
    marginTop: "0.25rem",
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

  workingOnBtnDisabled: {
    background: "none",
    border: "none",
    fontSize: "0.68rem",
    color: "#222222",
    padding: 0,
    letterSpacing: "0.04em",
    cursor: "default",
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

function Card({
  label,
  title,
  children,
}: {
  label: string;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div style={s.card}>
      <span style={s.cardLabel}>{label}</span>
      <span style={s.cardTitle}>{title}</span>
      {children}
    </div>
  );
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
  const [doneIds, setDoneIds] = useState<Set<string>>(new Set());

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
  }, [refresh]);

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

  const handleSetFocus = async (item: ActiveProject) => {
    await invoke("set_focus", {
      name: item.name,
      area: item.area,
      folderPath: item.folder_path,
      filePath: item.file_path ?? null,
    });
    refresh();
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
      if (["add_reminder", "complete_from_summary", "log_activity", "set_focus", "refresh"].includes(result.action)) {
        refresh();
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

      {data?.now_event && (
        <div style={{ display: "flex", alignItems: "baseline", gap: "0.6rem" }}>
          <span style={{
            fontSize: "0.65rem",
            fontWeight: 600,
            letterSpacing: "0.1em",
            textTransform: "uppercase" as const,
            color: "#2e2e2e",
          }}>Now</span>
          <span style={{ fontSize: "0.875rem", color: "#707070", fontWeight: 500 }}>
            {data.now_event.app}
          </span>
          <span style={{ fontSize: "0.75rem", color: "#333333" }}>
            {agoNow(data.now_event.secs_ago)}
          </span>
        </div>
      )}

      <div style={s.grid}>
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

        {/* Working On */}
        <div style={{ gridColumn: "1 / -1" }}>
          <Card label="Now" title="Working On">
            {isLoading ? (
              <Loading />
            ) : isError ? (
              <CardError msg={error} />
            ) : data!.working_on.length === 0 ? (
              <Empty text="No recent folders found." />
            ) : (
              <ul style={{ ...s.list, flexDirection: "row", flexWrap: "wrap", gap: "0.75rem" }}>
                {data!.working_on.map((item, i) => (
                  <li
                    key={i}
                    style={{
                      display: "flex",
                      flexDirection: "column",
                      gap: "0.15rem",
                      minWidth: "140px",
                      flex: "1 1 140px",
                    }}
                  >
                    <div style={s.listItemRow}>
                      <span style={s.workingOnName}>{trunc(item.name, 28)}</span>
                      <span style={s.dim}>{ago(item.modified_secs_ago)}</span>
                    </div>
                    <span style={s.workingOnMeta}>
                      {item.area}{item.recent_file ? ` · ${trunc(item.recent_file, 22)}` : ""}
                    </span>
                    <div style={s.workingOnActions}>
                      <button
                        style={s.workingOnBtn}
                        onClick={() => invoke("open_path", { path: item.folder_path })}
                      >
                        ↗ folder
                      </button>
                      {item.file_path ? (
                        <button
                          style={s.workingOnBtn}
                          onClick={() => invoke("open_path", { path: item.file_path! })}
                        >
                          ↗ file
                        </button>
                      ) : (
                        <span style={s.workingOnBtnDisabled}>↗ file</span>
                      )}
                      <button
                        style={{
                          ...s.workingOnBtn,
                          color: data!.current_focus?.folder_path === item.folder_path
                            ? "#5a5a5a"
                            : "#2e2e2e",
                        }}
                        onClick={() => handleSetFocus(item)}
                      >
                        {data!.current_focus?.folder_path === item.folder_path ? "★ focused" : "★ focus"}
                      </button>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </Card>
        </div>

        {/* Activity */}
        <Card label="Activity" title="Today's Apps">
          {isLoading ? (
            <Loading />
          ) : isError ? (
            <CardError msg={error} />
          ) : data!.activity.length === 0 ? (
            <Empty text="No activity tracked today." />
          ) : (
            <ul style={s.list}>
              {data!.activity.map((app, i) => (
                <li key={i} style={s.listItemRow}>
                  <span>{app.app}</span>
                  <span style={s.dim}>{app.minutes}m</span>
                </li>
              ))}
            </ul>
          )}
        </Card>

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

      <div style={s.commandBar}>
        <span style={s.commandLabel}>AI</span>
        <div style={s.commandInner}>
          <input
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
          {cmdResponse && !cmdLoading && (
            <span style={s.commandResponse}>{cmdResponse.message}</span>
          )}
        </div>
      </div>
    </div>
  );
}
