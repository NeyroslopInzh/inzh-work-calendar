import { useEffect, useMemo, useState, type FormEvent } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  deleteCard,
  getSettings,
  listCards,
  saveCard,
  setAutostart,
  setStatus,
  snoozeCard,
} from "./api";
import {
  agendaStamp,
  dayKey,
  formatDayTime,
  formatLate,
  formatTime,
  fromLocalInput,
  isDeadlineDue,
  isNagging,
  isSnoozed,
  isStartDue,
  monthGrid,
  monthLabel,
  placement,
  shiftToDay,
  toLocalInput,
} from "./time";
import { COLUMNS, type Card, type CardInput, type Placement, type Status } from "./types";

type Draft = {
  id: string | null;
  title: string;
  status: Status;
  startAt: string;
  deadlineAt: string;
};

type DragPayload = { id: string; kind: Placement };

const WEEK = ["пн", "вт", "ср", "чт", "пт", "сб", "вс"];
const SNOOZE = [
  { minutes: 15, label: "15 мин" },
  { minutes: 60, label: "1 час" },
  { minutes: 180, label: "3 часа" },
  { minutes: 1440, label: "Сутки" },
];

function explain(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "Не получилось сохранить";
}

function draftFromCard(card: Card): Draft {
  return {
    id: card.id,
    title: card.title,
    status: card.status,
    startAt: toLocalInput(card.startAt),
    deadlineAt: toLocalInput(card.deadlineAt),
  };
}

function blankDraft(day: Date): Draft {
  const start = new Date(day);
  const now = new Date();
  if (dayKey(day) === dayKey(now)) start.setHours(now.getHours() + 1, 0, 0, 0);
  else start.setHours(9, 0, 0, 0);
  return {
    id: null,
    title: "",
    status: "todo",
    startAt: toLocalInput(start.toISOString()),
    deadlineAt: "",
  };
}

function inputFromDraft(draft: Draft): CardInput {
  return {
    id: draft.id,
    title: draft.title,
    status: draft.status,
    startAt: fromLocalInput(draft.startAt),
    deadlineAt: fromLocalInput(draft.deadlineAt),
  };
}

export default function App() {
  const [cards, setCards] = useState<Card[]>([]);
  const [view, setView] = useState<"calendar" | "board">("calendar");
  const [month, setMonth] = useState(() => new Date());
  const [selected, setSelected] = useState(() => new Date());
  const [now, setNow] = useState(() => new Date());
  const [draft, setDraft] = useState<Draft | null>(null);
  const [error, setError] = useState("");
  const [autostart, setAutostartOn] = useState(true);
  const [focusId, setFocusId] = useState<string | null>(null);
  const [dropDay, setDropDay] = useState<string | null>(null);
  const [dropColumn, setDropColumn] = useState<Status | null>(null);
  const [customSnooze, setCustomSnooze] = useState("30");

  useEffect(() => {
    let unlistenCards: (() => void) | undefined;
    let unlistenFocus: (() => void) | undefined;
    let alive = true;
    const timer = window.setInterval(() => setNow(new Date()), 30000);
    void (async () => {
      try {
        const loaded = await listCards();
        if (alive) setCards(loaded);
      } catch (loadError) {
        if (alive) setError(explain(loadError));
      }
      try {
        const settings = await getSettings();
        if (alive) setAutostartOn(settings.autostart);
      } catch (loadError) {
        if (alive) setError(explain(loadError));
      }
      const cardsUn = await listen<Card[]>("cards-changed", (event) => setCards(event.payload));
      const focusUn = await listen<string>("focus-card", (event) => setFocusId(event.payload));
      if (!alive) {
        cardsUn();
        focusUn();
        return;
      }
      unlistenCards = cardsUn;
      unlistenFocus = focusUn;
    })();
    return () => {
      alive = false;
      unlistenCards?.();
      unlistenFocus?.();
      window.clearInterval(timer);
    };
  }, []);

  useEffect(() => {
    if (!focusId) return;
    const card = cards.find((item) => item.id === focusId);
    if (!card) return;
    setDraft(draftFromCard(card));
    setFocusId(null);
  }, [cards, focusId]);

  const nagging = useMemo(
    () => cards.filter((card) => isNagging(card, now)),
    [cards, now],
  );
  const days = monthGrid(month);
  const selectedKey = dayKey(selected);
  const agenda = cards
    .filter((card) => placement(card, selected))
    .sort((a, b) => {
      const aTime = agendaStamp(a, selected);
      const bTime = agendaStamp(b, selected);
      return (aTime ? Date.parse(aTime) : 0) - (bTime ? Date.parse(bTime) : 0);
    });

  async function run(action: () => Promise<Card[]>): Promise<boolean> {
    try {
      setCards(await action());
      setError("");
      return true;
    } catch (actionError) {
      setError(explain(actionError));
      return false;
    }
  }

  async function onSave(event: FormEvent) {
    event.preventDefault();
    if (!draft) return;
    if (await run(() => saveCard(inputFromDraft(draft)))) setDraft(null);
  }

  function readDrag(event: React.DragEvent): DragPayload | null {
    const raw = event.dataTransfer.getData("text/plain");
    if (!raw) return null;
    try {
      return JSON.parse(raw) as DragPayload;
    } catch {
      return null;
    }
  }

  function startDrag(event: React.DragEvent, card: Card, kind: Placement) {
    event.dataTransfer.setData("text/plain", JSON.stringify({ id: card.id, kind }));
    event.dataTransfer.effectAllowed = "move";
  }

  async function dropOnDay(event: React.DragEvent, day: Date) {
    event.preventDefault();
    setDropDay(null);
    const payload = readDrag(event);
    const card = payload && cards.find((item) => item.id === payload.id);
    if (!payload || !card) return;
    const next: CardInput = {
      id: card.id,
      title: card.title,
      status: card.status,
      startAt:
        payload.kind === "deadline"
          ? card.startAt
          : card.startAt
            ? shiftToDay(card.startAt, day)
            : null,
      deadlineAt:
        payload.kind === "start"
          ? card.deadlineAt
          : card.deadlineAt
            ? shiftToDay(card.deadlineAt, day)
            : null,
    };
    await run(() => saveCard(next));
  }

  function chooseDay(day: Date) {
    setSelected(day);
    if (day.getMonth() !== month.getMonth() || day.getFullYear() !== month.getFullYear()) {
      setMonth(new Date(day.getFullYear(), day.getMonth(), 1));
    }
  }

  return (
    <div className="app">
      <header className="top">
        <div className="brand">
          <b>inzh</b>
          <span>work calendar</span>
        </div>
        <div className="switch">
          <button type="button" aria-pressed={view === "calendar"} onClick={() => setView("calendar")}>
            Календарь
          </button>
          <button type="button" aria-pressed={view === "board"} onClick={() => setView("board")}>
            Доска
          </button>
        </div>
        <div className="grow" />
        <label className="check">
          <input
            type="checkbox"
            checked={autostart}
            onChange={(event) => {
              const enabled = event.target.checked;
              setAutostartOn(enabled);
              void setAutostart(enabled)
                .then((settings) => setAutostartOn(settings.autostart))
                .catch((autostartError) => {
                  setAutostartOn(!enabled);
                  setError(explain(autostartError));
                });
            }}
          />
          Автозапуск
        </label>
        <button type="button" className="solid" onClick={() => setDraft(blankDraft(selected))}>
          Новая задача
        </button>
      </header>

      {error && <p className="banner">{error}</p>}

      {nagging.length > 0 && (
        <section className="attention">
          {nagging.map((card) => (
            <article key={card.id}>
              <strong>{card.title}</strong>
              <span className="meta">
                {isDeadlineDue(card, now) && card.deadlineAt
                  ? `дедлайн просрочен на ${formatLate(card.deadlineAt, now)}`
                  : card.startAt
                    ? `пора начать, с ${formatTime(card.startAt)}`
                    : "ждёт"}
              </span>
              <div className="actions">
                <button type="button" className="tiny" onClick={() => void run(() => setStatus(card.id, "done"))}>
                  Готово
                </button>
                {isDeadlineDue(card, now) &&
                  SNOOZE.map((option) => (
                    <button
                      key={option.minutes}
                      type="button"
                      className="tiny late"
                      onClick={() => void run(() => snoozeCard(card.id, option.minutes))}
                    >
                      {option.label}
                    </button>
                  ))}
              </div>
            </article>
          ))}
        </section>
      )}

      <main className="work">
        {view === "calendar" ? (
          <div className="calendar">
            <section>
              <div className="month-head">
                <button
                  type="button"
                  className="ghost"
                  onClick={() => setMonth(new Date(month.getFullYear(), month.getMonth() - 1, 1))}
                >
                  ←
                </button>
                <h2>{monthLabel(month)}</h2>
                <button
                  type="button"
                  className="ghost"
                  onClick={() => setMonth(new Date(month.getFullYear(), month.getMonth() + 1, 1))}
                >
                  →
                </button>
                <button type="button" className="ghost" onClick={() => chooseDay(new Date())}>
                  Сегодня
                </button>
              </div>
              <div className="week">
                {WEEK.map((name) => (
                  <span key={name}>{name}</span>
                ))}
              </div>
              <div className="grid">
                {days.map((day) => {
                  const key = dayKey(day);
                  const items = cards.filter((card) => placement(card, day));
                  return (
                    <div
                      key={key}
                      className={[
                        "day",
                        day.getMonth() === month.getMonth() ? "" : "outside",
                        key === selectedKey ? "selected" : "",
                        key === dayKey(now) ? "today" : "",
                        dropDay === key ? "drop" : "",
                      ].join(" ")}
                      onClick={() => chooseDay(day)}
                      onDragOver={(event) => {
                        event.preventDefault();
                        setDropDay(key);
                      }}
                      onDragLeave={() => setDropDay(null)}
                      onDrop={(event) => void dropOnDay(event, day)}
                    >
                      <span className="num">{day.getDate()}</span>
                      {items.slice(0, 3).map((card) => {
                        const where = placement(card, day) ?? "both";
                        const stamp = where === "deadline" ? card.deadlineAt : card.startAt ?? card.deadlineAt;
                        return (
                          <button
                            key={`${card.id}-${where}`}
                            type="button"
                            draggable
                            className={[
                              "chip",
                              card.status === "done" ? "done" : isDeadlineDue(card, now) && where !== "start" ? "late" : where === "start" ? "start" : "",
                            ].join(" ")}
                            onDragStart={(event) => startDrag(event, card, where)}
                            onClick={(event) => {
                              event.stopPropagation();
                              setDraft(draftFromCard(card));
                            }}
                          >
                            {stamp ? `${formatTime(stamp)} ` : ""}
                            {card.title}
                          </button>
                        );
                      })}
                      {items.length > 3 && <span className="count">+{items.length - 3}</span>}
                    </div>
                  );
                })}
              </div>
            </section>
            <aside className="agenda">
              <div className="day-head">
                <h2>
                  {selected.toLocaleDateString("ru-RU", { day: "numeric", month: "long" })}
                </h2>
                <button type="button" className="tiny" onClick={() => setDraft(blankDraft(selected))}>
                  Добавить
                </button>
              </div>
              <div className="list">
                {agenda.length === 0 && <p className="empty">На этот день пусто. Задача без даты живёт на доске.</p>}
                {agenda.map((card) => {
                  const where = placement(card, selected) ?? "both";
                  return (
                    <article
                      key={card.id}
                      className={["row", card.status === "done" ? "done" : isDeadlineDue(card, now) ? "late" : ""].join(" ")}
                      draggable
                      onDragStart={(event) => startDrag(event, card, where)}
                    >
                      <strong>{card.title}</strong>
                      <span className="meta">{whenLine(card, now)}</span>
                      <div className="actions">
                        <button type="button" className="tiny" onClick={() => setDraft(draftFromCard(card))}>
                          Изменить
                        </button>
                        {card.status !== "done" && (
                          <button type="button" className="tiny" onClick={() => void run(() => setStatus(card.id, "done"))}>
                            Готово
                          </button>
                        )}
                      </div>
                    </article>
                  );
                })}
              </div>
            </aside>
          </div>
        ) : (
          <div className="board">
            {COLUMNS.map((column) => {
              const items = cards
                .filter((card) => card.status === column.id)
                .sort((a, b) => rank(a, now) - rank(b, now));
              return (
                <section
                  key={column.id}
                  className={dropColumn === column.id ? "column drop" : "column"}
                  onDragOver={(event) => {
                    event.preventDefault();
                    setDropColumn(column.id);
                  }}
                  onDragLeave={() => setDropColumn(null)}
                  onDrop={(event) => {
                    event.preventDefault();
                    setDropColumn(null);
                    const payload = readDrag(event);
                    if (!payload) return;
                    void run(() => setStatus(payload.id, column.id));
                  }}
                >
                  <header>
                    <b>
                      <i className={`dot ${column.id}`} />
                      {column.title}
                    </b>
                    <span className="count">{items.length}</span>
                  </header>
                  <div className="cards">
                    {items.map((card) => (
                      <article
                        key={card.id}
                        className={["card", card.status === "done" ? "done" : isDeadlineDue(card, now) ? "late" : ""].join(" ")}
                        draggable
                        onDragStart={(event) => startDrag(event, card, placement(card, selected) ?? "both")}
                      >
                        <strong>{card.title}</strong>
                        <span className="meta">{whenLine(card, now)}</span>
                        <div className="actions">
                          <button type="button" className="tiny" onClick={() => setDraft(draftFromCard(card))}>
                            Изменить
                          </button>
                          {card.status !== "done" && (
                            <button type="button" className="tiny" onClick={() => void run(() => setStatus(card.id, "done"))}>
                              Готово
                            </button>
                          )}
                        </div>
                      </article>
                    ))}
                  </div>
                </section>
              );
            })}
          </div>
        )}
      </main>

      <p className="foot">
        Напоминание каждые 10 минут, пока задача не в «Готово». Дедлайн можно отложить не больше чем на сутки.
        Закрытие прячет окно в трей. Задачу можно продиктовать в чат — она попадёт сюда, если приложение запущено.
      </p>

      {draft && (
        <div className="shade" onMouseDown={() => setDraft(null)}>
          <form
            className="editor"
            onMouseDown={(event) => event.stopPropagation()}
            onSubmit={(event) => void onSave(event)}
          >
            <h2>{draft.id ? "Задача" : "Новая задача"}</h2>
            <label>
              Название
              <input
                autoFocus
                value={draft.title}
                onChange={(event) => setDraft({ ...draft, title: event.target.value })}
              />
            </label>
            <label>
              Начало, если надо напомнить приступить
              <input
                type="datetime-local"
                value={draft.startAt}
                onChange={(event) => setDraft({ ...draft, startAt: event.target.value })}
              />
            </label>
            <label>
              Дедлайн
              <input
                type="datetime-local"
                value={draft.deadlineAt}
                onChange={(event) => setDraft({ ...draft, deadlineAt: event.target.value })}
              />
            </label>
            <label>
              Статус
              <select
                value={draft.status}
                onChange={(event) => setDraft({ ...draft, status: event.target.value as Status })}
              >
                {COLUMNS.map((column) => (
                  <option key={column.id} value={column.id}>
                    {column.title}
                  </option>
                ))}
              </select>
            </label>
            <p className="hint">
              После начала напоминания идут каждые 10 минут, пока задача не закрыта. У дедлайна то же самое, но его можно отложить.
            </p>
            {draft.id && isDeadlineDue(inputAsCard(draft), now) && (
              <div className="field">
                Отложить дедлайн
                <div className="actions">
                  {SNOOZE.map((option) => (
                    <button
                      key={option.minutes}
                      type="button"
                      className="tiny"
                      onClick={() => {
                        const id = draft.id;
                        if (id) void run(() => snoozeCard(id, option.minutes));
                      }}
                    >
                      {option.label}
                    </button>
                  ))}
                  <input
                    type="number"
                    min={1}
                    max={1440}
                    value={customSnooze}
                    onChange={(event) => setCustomSnooze(event.target.value)}
                  />
                  <button
                    type="button"
                    className="tiny"
                    onClick={() => {
                      const minutes = Number(customSnooze);
                      if (draft.id && minutes >= 1 && minutes <= 1440) {
                        void run(() => snoozeCard(draft.id as string, minutes));
                      }
                    }}
                  >
                    мин
                  </button>
                </div>
              </div>
            )}
            {(() => {
              const current = cards.find((card) => card.id === draft.id);
              return current && isSnoozed(current, now) && current.snoozeUntil ? (
                <p className="hint">Тишина до {formatDayTime(current.snoozeUntil)}.</p>
              ) : null;
            })()}
            <div className="actions">
              {draft.id ? (
                <button
                  type="button"
                  className="danger"
                  onClick={() => {
                    const id = draft.id;
                    if (!id || !window.confirm("Удалить задачу?")) return;
                    void run(() => deleteCard(id)).then((ok) => {
                      if (ok) setDraft(null);
                    });
                  }}
                >
                  Удалить
                </button>
              ) : (
                <span />
              )}
              <div className="actions">
                <button type="button" className="ghost" onClick={() => setDraft(null)}>
                  Закрыть
                </button>
                <button type="submit" className="solid">
                  Сохранить
                </button>
              </div>
            </div>
          </form>
        </div>
      )}
    </div>
  );
}

function whenLine(card: Card, now: Date): string {
  const parts: string[] = [];
  if (card.startAt) parts.push(`начало ${formatDayTime(card.startAt)}`);
  if (card.deadlineAt) parts.push(`дедлайн ${formatDayTime(card.deadlineAt)}`);
  if (isSnoozed(card, now) && card.snoozeUntil) parts.push(`тишина до ${formatTime(card.snoozeUntil)}`);
  if (isStartDue(card, now)) parts.push("пора начать");
  if (parts.length === 0) return "без срока";
  return parts.join(" · ");
}

function rank(card: Card, now: Date): number {
  if (isDeadlineDue(card, now) && card.deadlineAt) return Date.parse(card.deadlineAt);
  if (card.deadlineAt) return Date.parse(card.deadlineAt);
  if (card.startAt) return Date.parse(card.startAt);
  return Number.MAX_SAFE_INTEGER;
}

function inputAsCard(draft: Draft): Card {
  return {
    id: draft.id ?? "",
    title: draft.title,
    status: draft.status,
    startAt: fromLocalInput(draft.startAt),
    deadlineAt: fromLocalInput(draft.deadlineAt),
    snoozeUntil: null,
    lastStartNotifiedAt: null,
    lastDeadlineNotifiedAt: null,
    createdAt: "",
    completedAt: null,
  };
}
