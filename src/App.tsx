import { useEffect, useMemo, useState, type FormEvent } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  addColumn,
  deleteCard,
  getSettings,
  hideNag,
  listCards,
  listColumns,
  removeColumn,
  saveCard,
  setAutostart,
  setStatus,
  setTheme,
  snoozeCard,
} from "./api";
import { DateField } from "./DateField";
import { applyUpdate, findUpdate, type UpdateOffer } from "./updates";
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
  showsOn,
  toLocalInput,
} from "./time";
import { DONE, TODO, WEEKDAYS, type Card, type CardInput, type Column, type Placement } from "./types";

type Draft = {
  id: string | null;
  title: string;
  description: string;
  status: string;
  startAt: string;
  deadlineAt: string;
  repeat: boolean;
  repeatFrom: string;
  repeatUntil: string;
  weekdays: number[];
};

const FALLBACK_COLUMNS: Column[] = [
  { id: TODO, title: "К выполнению" },
  { id: "doing", title: "В работе" },
  { id: DONE, title: "Готово" },
];

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
  const today = dayKey(new Date());
  return {
    id: card.id,
    title: card.title,
    description: card.description,
    status: card.status,
    startAt: toLocalInput(card.startAt),
    deadlineAt: toLocalInput(card.deadlineAt),
    repeat: Boolean(card.recurrence),
    repeatFrom: card.recurrence?.from ?? today,
    repeatUntil: card.recurrence?.until ?? today,
    weekdays: card.recurrence?.weekdays ?? [],
  };
}

function blankDraft(): Draft {
  const now = new Date();
  const weekday = now.getDay() === 0 ? 7 : now.getDay();
  return {
    id: null,
    title: "",
    description: "",
    status: TODO,
    startAt: "",
    deadlineAt: toLocalInput(now.toISOString()),
    repeat: false,
    repeatFrom: dayKey(now),
    repeatUntil: dayKey(now),
    weekdays: [weekday],
  };
}

function inputFromDraft(draft: Draft): CardInput {
  return {
    id: draft.id,
    title: draft.title,
    description: draft.description,
    status: draft.status,
    startAt: fromLocalInput(draft.startAt),
    deadlineAt: fromLocalInput(draft.deadlineAt),
    recurrence: draft.repeat
      ? { from: draft.repeatFrom, until: draft.repeatUntil, weekdays: draft.weekdays }
      : null,
  };
}

function windowLabel(): string {
  try {
    return getCurrentWindow().label;
  } catch {
    return "main";
  }
}

export default function App() {
  if (windowLabel() === "nag") return <NagPopup />;
  return <MainApp />;
}

function MainApp() {
  const [cards, setCards] = useState<Card[]>([]);
  const [view, setView] = useState<"calendar" | "board" | "settings">("calendar");
  const [month, setMonth] = useState(() => new Date());
  const [selected, setSelected] = useState(() => new Date());
  const [now, setNow] = useState(() => new Date());
  const [draft, setDraft] = useState<Draft | null>(null);
  const [error, setError] = useState("");
  const [autostart, setAutostartOn] = useState(true);
  const [focusId, setFocusId] = useState<string | null>(null);
  const [dropDay, setDropDay] = useState<string | null>(null);
  const [dropColumn, setDropColumn] = useState<string | null>(null);
  const [draggingId, setDraggingId] = useState<string | null>(null);
  const [dragPoint, setDragPoint] = useState<{ x: number; y: number } | null>(null);
  const [viewId, setViewId] = useState<string | null>(null);
  const [columns, setColumns] = useState<Column[]>(FALLBACK_COLUMNS);
  const [newStatus, setNewStatus] = useState("");
  const [theme, setThemeName] = useState<"dark" | "light">("dark");
  const [customSnooze, setCustomSnooze] = useState("30");
  const [offer, setOffer] = useState<UpdateOffer | null>(null);
  const [updating, setUpdating] = useState(false);

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
        const [settings, loadedColumns] = await Promise.all([getSettings(), listColumns()]);
        if (!alive) return;
        setAutostartOn(settings.autostart);
        setThemeName(settings.theme === "light" ? "light" : "dark");
        if (loadedColumns.length > 0) setColumns(loadedColumns);
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
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  useEffect(() => {
    let alive = true;
    async function look() {
      try {
        const next = await findUpdate();
        if (alive) setOffer(next);
      } catch {
        /* нет сети или релиз ещё без подписи */
      }
    }
    void look();
    const timer = window.setInterval(() => void look(), 6 * 60 * 60 * 1000);
    return () => {
      alive = false;
      window.clearInterval(timer);
    };
  }, []);

  async function onUpdate() {
    if (!offer || updating) return;
    setUpdating(true);
    try {
      await applyUpdate(offer.kind);
      if (offer.kind === "browser") setUpdating(false);
    } catch (updateError) {
      setUpdating(false);
      setError(explain(updateError));
    }
  }

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (event.key !== "Escape") return;
      if (draft) setDraft(null);
      else if (viewId) setViewId(null);
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [draft, viewId]);

  useEffect(() => {
    if (!focusId) return;
    const card = cards.find((item) => item.id === focusId);
    if (!card) return;
    setViewId(card.id);
    setFocusId(null);
  }, [cards, focusId]);

  const nagging = useMemo(
    () => cards.filter((card) => isNagging(card, now)),
    [cards, now],
  );
  const days = monthGrid(month);
  const selectedKey = dayKey(selected);
  const agenda = cards
    .filter((card) => showsOn(card, selected))
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
    setDraggingId(card.id);
    event.dataTransfer.setData("text/plain", JSON.stringify({ id: card.id, kind }));
    event.dataTransfer.effectAllowed = "move";
  }

  function columnTitle(id: string): string {
    return columns.find((column) => column.id === id)?.title ?? id;
  }

  function openView(cardId: string) {
    setViewId(cardId);
  }

  function beginBoardDrag(event: React.PointerEvent, cardId: string) {
    if (event.button !== 0) return;
    const target = event.target as HTMLElement;
    if (target.closest("button, a, input, textarea, select")) return;
    event.preventDefault();
    const startX = event.clientX;
    const startY = event.clientY;
    let moved = false;
    function move(pointer: PointerEvent) {
      if (!moved && Math.hypot(pointer.clientX - startX, pointer.clientY - startY) < 6) return;
      moved = true;
      const column = document.elementFromPoint(pointer.clientX, pointer.clientY)?.closest("[data-column]");
      setDraggingId(cardId);
      setDragPoint({ x: pointer.clientX, y: pointer.clientY });
      setDropColumn(column?.getAttribute("data-column") ?? null);
    }
    function up(pointer: PointerEvent) {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      const column = document.elementFromPoint(pointer.clientX, pointer.clientY)?.closest("[data-column]");
      const status = column?.getAttribute("data-column");
      setDraggingId(null);
      setDragPoint(null);
      setDropColumn(null);
      if (moved && status) void run(() => setStatus(cardId, status));
      else if (!moved) openView(cardId);
    }
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  }

  function chooseTheme(next: "dark" | "light") {
    setThemeName(next);
    void setTheme(next).catch((themeError) => setError(explain(themeError)));
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
      description: card.description,
      status: card.status,
      recurrence: card.recurrence,
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
          <b>inzh's</b>
          <span>work calendar</span>
        </div>
        <div className="switch" data-view={view}>
          <span className="thumb" />
          <button type="button" aria-pressed={view === "calendar"} onClick={() => setView("calendar")}>
            Календарь
          </button>
          <button type="button" aria-pressed={view === "board"} onClick={() => setView("board")}>
            Доска
          </button>
          <button type="button" aria-pressed={view === "settings"} onClick={() => setView("settings")}>
            Настройки
          </button>
        </div>
        <div className="top-actions">
          <div className="switch theme" data-theme={theme}>
            <span className="thumb" />
            <button type="button" aria-label="Светлая тема" aria-pressed={theme === "light"} onClick={() => chooseTheme("light")}>
              <SunIcon />
            </button>
            <button type="button" aria-label="Тёмная тема" aria-pressed={theme === "dark"} onClick={() => chooseTheme("dark")}>
              <MoonIcon />
            </button>
          </div>
          <button type="button" className="solid" onClick={() => setDraft(blankDraft())}>
            Новая задача
          </button>
        </div>
      </header>

      {error && <p className="banner">{error}</p>}

      <main className="work">
        {view === "calendar" ? (
          <div className="calendar">
            <section>
              <div className="month-head">
                <h2>{monthLabel(month)}</h2>
                <div className="month-tools">
                  <button
                    type="button"
                    className="ghost icon-button"
                    onClick={() => setMonth(new Date(month.getFullYear(), month.getMonth() - 1, 1))}
                  >
                    ←
                  </button>
                  <button type="button" className="ghost" onClick={() => chooseDay(new Date())}>
                    Сегодня
                  </button>
                  <button
                    type="button"
                    className="ghost icon-button"
                    onClick={() => setMonth(new Date(month.getFullYear(), month.getMonth() + 1, 1))}
                  >
                    →
                  </button>
                </div>
              </div>
              <div className="week">
                {WEEK.map((name) => (
                  <span key={name}>{name}</span>
                ))}
              </div>
              <div className="grid">
                {days.map((day) => {
                  const key = dayKey(day);
                  const items = cards.filter((card) => showsOn(card, day));
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
              <DueStrip cards={nagging} now={now} run={run} />
            </section>
            <aside className="agenda">
              <div className="day-head">
                <h2>
                  {selected.toLocaleDateString("ru-RU", { day: "numeric", month: "long" })}
                </h2>
                <button type="button" className="tiny" onClick={() => setDraft(blankDraft())}>
                  Добавить
                </button>
              </div>
              <div className="list">
                {agenda.length === 0 && <p className="empty">На этот день пусто. Задача без даты живёт на доске.</p>}
                {agenda.map((card) => (
                    <article
                      key={card.id}
                      className={["row", card.status === "done" ? "done" : isDeadlineDue(card, now) ? "late" : ""].join(" ")}
                      onClick={(event) => {
                        if ((event.target as HTMLElement).closest("button")) return;
                        openView(card.id);
                      }}
                    >
                      <strong>{card.title}</strong>
                      {card.description.trim() && <p className="preview">{preview(card.description)}</p>}
                      <span className="status">{columnTitle(card.status)}</span>
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
            </aside>
          </div>
        ) : view === "board" ? (
          <div className="board-wrap">
          <div className="board">
            {columns.map((column) => {
              const items = cards
                .filter((card) => card.status === column.id)
                .sort((a, b) => rank(a, now) - rank(b, now));
              return (
                <section
                  key={column.id}
                  data-column={column.id}
                  className={dropColumn === column.id ? "column drop" : "column"}
                  onDragOver={(event) => {
                    event.preventDefault();
                    setDropColumn(column.id);
                  }}
                  onDragLeave={() => setDropColumn(null)}
                  onDrop={(event) => {
                    event.preventDefault();
                    const payload = readDrag(event);
                    const id = payload?.id ?? draggingId;
                    setDropColumn(null);
                    setDraggingId(null);
                    if (!id || id === "") return;
                    void run(() => setStatus(id, column.id));
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
                        className={["card", draggingId === card.id ? "dragging" : "", card.status === "done" ? "done" : isDeadlineDue(card, now) ? "late" : ""].join(" ")}
                        onPointerDown={(event) => beginBoardDrag(event, card.id)}
                      >
                        <strong>{card.title}</strong>
                        {card.description.trim() && <p className="preview">{preview(card.description)}</p>}
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
          <DueStrip cards={nagging} now={now} run={run} />
          </div>
        ) : (
          <SettingsView
            autostart={autostart}
            offer={offer}
            updating={updating}
            onUpdate={() => void onUpdate()}
            columns={columns}
            newStatus={newStatus}
            onNewStatus={setNewStatus}
            onAutostart={(enabled) => {
              setAutostartOn(enabled);
              void setAutostart(enabled)
                .then((settings) => setAutostartOn(settings.autostart))
                .catch((autostartError) => {
                  setAutostartOn(!enabled);
                  setError(explain(autostartError));
                });
            }}
            onAdd={() => {
              const title = newStatus.trim();
              if (!title) return;
              void addColumn(title)
                .then((next) => {
                  setColumns(next);
                  setNewStatus("");
                  setError("");
                })
                .catch((addError) => setError(explain(addError)));
            }}
            onRemove={(id) => {
              void removeColumn(id)
                .then((next) => {
                  setColumns(next.columns);
                  setCards(next.cards);
                  setError("");
                })
                .catch((removeError) => setError(explain(removeError)));
            }}
          />
        )}
      </main>

      <p className="foot">
        Начало напоминает только в «К выполнению». Дедлайн — в любом статусе, кроме «Готово», каждые 10 минут. Отложить дедлайн можно не больше чем на сутки.
        Закрытие прячет окно в трей. Задачу можно продиктовать в чат — она попадёт сюда, если приложение запущено.
      </p>

      {draggingId && dragPoint && (
        <div className="drag-chip" style={{ left: dragPoint.x, top: dragPoint.y }}>
          {cards.find((card) => card.id === draggingId)?.title}
        </div>
      )}

      {viewId && cards.some((card) => card.id === viewId) && (
        <TaskView
          card={cards.find((card) => card.id === viewId) as Card}
          statusTitle={columnTitle(cards.find((card) => card.id === viewId)?.status ?? "")}
          onClose={() => setViewId(null)}
          onEdit={() => {
            const card = cards.find((item) => item.id === viewId);
            if (!card) return;
            setDraft(draftFromCard(card));
            setViewId(null);
          }}
        />
      )}

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
              Описание
              <textarea
                rows={4}
                value={draft.description}
                onChange={(event) => setDraft({ ...draft, description: event.target.value })}
              />
            </label>
            <label>
              Начало, если надо напомнить приступить
              <DateField
                key={`${draft.id ?? "new"}-start`}
                value={draft.startAt}
                onChange={(startAt) => setDraft({ ...draft, startAt })}
              />
            </label>
            <label>
              Дедлайн
              <DateField
                key={`${draft.id ?? "new"}-deadline`}
                value={draft.deadlineAt}
                onChange={(deadlineAt) => setDraft({ ...draft, deadlineAt })}
              />
            </label>
            <label className="check">
              <input
                type="checkbox"
                checked={draft.repeat}
                onChange={(event) => setDraft({ ...draft, repeat: event.target.checked })}
              />
              Повтор по дням недели
            </label>
            {draft.repeat && (
              <>
                <label>
                  С даты
                  <DateField
                    key={`${draft.id ?? "new"}-from`}
                    withTime={false}
                    value={draft.repeatFrom}
                    onChange={(repeatFrom) => setDraft({ ...draft, repeatFrom })}
                  />
                </label>
                <label>
                  По дату
                  <DateField
                    key={`${draft.id ?? "new"}-until`}
                    withTime={false}
                    value={draft.repeatUntil}
                    onChange={(repeatUntil) => setDraft({ ...draft, repeatUntil })}
                  />
                </label>
                <div className="weekdays">
                  {WEEKDAYS.map((day) => (
                    <button
                      key={day.id}
                      type="button"
                      aria-pressed={draft.weekdays.includes(day.id)}
                      onClick={() =>
                        setDraft({
                          ...draft,
                          weekdays: draft.weekdays.includes(day.id)
                            ? draft.weekdays.filter((item) => item !== day.id)
                            : [...draft.weekdays, day.id],
                        })
                      }
                    >
                      {day.label}
                    </button>
                  ))}
                </div>
              </>
            )}
            <label>
              Статус
              <select
                value={draft.status}
                onChange={(event) => setDraft({ ...draft, status: event.target.value })}
              >
                {columns.map((column) => (
                  <option key={column.id} value={column.id}>
                    {column.title}
                  </option>
                ))}
              </select>
            </label>
            <p className="hint">
              Начало напоминает каждые 10 минут, только пока задача в «К выполнению». Дедлайн напоминает в остальных статусах тоже, пока она не в «Готово». Delete в дате стирает текущие цифры и переходит к следующим.
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

function SettingsView({
  autostart,
  offer,
  updating,
  onUpdate,
  columns,
  newStatus,
  onNewStatus,
  onAutostart,
  onAdd,
  onRemove,
}: {
  autostart: boolean;
  offer: UpdateOffer | null;
  updating: boolean;
  onUpdate: () => void;
  columns: Column[];
  newStatus: string;
  onNewStatus: (value: string) => void;
  onAutostart: (enabled: boolean) => void;
  onAdd: () => void;
  onRemove: (id: string) => void;
}) {
  return (
    <section className="settings">
      <h2>Настройки</h2>
      {offer && (
        <article className="setting">
          <div>
            <strong>Доступна версия {offer.version}</strong>
            <p>
              {offer.kind === "browser"
                ? "Пакет deb ставит система. Новая версия на странице релиза."
                : "Скачать и перезапустить календарь"}
            </p>
          </div>
          <button type="button" className="solid" disabled={updating} onClick={onUpdate}>
            {updating
              ? "Скачиваем…"
              : offer.kind === "browser"
                ? "Открыть страницу релиза"
                : "Обновить и перезапустить"}
          </button>
        </article>
      )}
      <article className="setting">
        <div>
          <strong>Автозапуск</strong>
          <p>Открывать вместе с системой</p>
        </div>
        <button
          type="button"
          className={autostart ? "tumbler on" : "tumbler"}
          aria-pressed={autostart}
          aria-label="Автозапуск"
          onClick={() => onAutostart(!autostart)}
        >
          <span />
        </button>
      </article>
      <article className="setting column-setting">
        <div>
          <strong>Статусы доски</strong>
          <p>«К выполнению» и «Готово» остаются всегда. Остальные можно убрать или добавить.</p>
        </div>
        <div className="list">
          {columns.map((column) => (
            <div key={column.id} className="status-row">
              <strong>{column.title}</strong>
              {column.id !== TODO && column.id !== DONE ? (
                <button type="button" className="tiny late" onClick={() => onRemove(column.id)}>
                  Убрать
                </button>
              ) : (
                <span className="meta">всегда</span>
              )}
            </div>
          ))}
        </div>
        <form
          className="status-add"
          onSubmit={(event) => {
            event.preventDefault();
            onAdd();
          }}
        >
          <input
            value={newStatus}
            placeholder="Новый статус"
            onChange={(event) => onNewStatus(event.target.value)}
          />
          <button type="submit" className="solid">
            Добавить
          </button>
        </form>
      </article>
    </section>
  );
}

function NagPopup() {
  const [items, setItems] = useState<Card[]>([]);

  function apply(all: Card[]) {
    const due = all.filter((card) => isNagging(card, new Date()));
    setItems(due);
    if (due.length === 0) void hideNag();
  }

  async function load() {
    apply(await listCards());
  }

  useEffect(() => {
    void load();
    const timer = window.setInterval(() => void load(), 5000);
    let unlistenRefresh: (() => void) | undefined;
    let unlistenCards: (() => void) | undefined;
    void listen("nag-refresh", () => void load()).then((stop) => {
      unlistenRefresh = stop;
    });
    void listen<Card[]>("cards-changed", (event) => apply(event.payload)).then((stop) => {
      unlistenCards = stop;
    });
    return () => {
      window.clearInterval(timer);
      unlistenRefresh?.();
      unlistenCards?.();
    };
  }, []);

  function dragWindow(event: React.PointerEvent) {
    if ((event.target as HTMLElement).closest("button")) return;
    void getCurrentWindow().startDragging();
  }

  return (
    <div className="nag">
      <header className="nag-bar">
        <strong data-tauri-drag-region onPointerDown={dragWindow}>
          inzh's work calendar
        </strong>
        <button type="button" className="nag-close" onClick={() => void hideNag()}>
          Закрыть
        </button>
      </header>
      {items.map((card) => (
        <article key={card.id}>
          <b>{card.title}</b>
          <span>
            {isDeadlineDue(card, new Date()) ? "Дедлайн" : "Пора начать"}
            {card.deadlineAt ? ` · ${formatDayTime(card.deadlineAt)}` : ""}
          </span>
          <div className="actions">
            <button type="button" className="solid" onClick={() => void setStatus(card.id, "done").then(load)}>
              Готово
            </button>
            {isDeadlineDue(card, new Date()) &&
              SNOOZE.map((option) => (
                <button
                  key={option.minutes}
                  type="button"
                  className="tiny"
                  onClick={() => void snoozeCard(card.id, option.minutes).then(load)}
                >
                  {option.label}
                </button>
              ))}
          </div>
        </article>
      ))}
    </div>
  );
}

function preview(text: string): string {
  const clean = text.replace(/\s+/g, " ").trim();
  if (clean.length <= 180) return clean;
  return `${clean.slice(0, 180).trimEnd()}…`;
}

function DueStrip({
  cards,
  now,
  run,
}: {
  cards: Card[];
  now: Date;
  run: (action: () => Promise<Card[]>) => Promise<boolean>;
}) {
  if (cards.length === 0) return null;
  return (
    <section className="attention">
      {cards.map((card) => (
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
  );
}

function TaskView({
  card,
  statusTitle,
  onClose,
  onEdit,
}: {
  card: Card;
  statusTitle: string;
  onClose: () => void;
  onEdit: () => void;
}) {
  return (
    <div className="shade" onMouseDown={onClose}>
      <article className="viewer" onMouseDown={(event) => event.stopPropagation()}>
        <h2>{card.title}</h2>
        <p className="meta">{statusTitle}</p>
        {card.description.trim() ? <p className="preview">{card.description}</p> : <p className="empty">Без описания</p>}
        <p className="meta">{whenLine(card, new Date())}</p>
        {card.recurrence && (
          <p className="meta">
            Повтор {card.recurrence.from} — {card.recurrence.until},{" "}
            {card.recurrence.weekdays
              .map((id) => WEEKDAYS.find((day) => day.id === id)?.label ?? String(id))
              .join(" ")}
          </p>
        )}
        <div className="actions">
          <button type="button" className="ghost" onClick={onClose}>
            Закрыть
          </button>
          <button type="button" className="solid" onClick={onEdit}>
            Изменить
          </button>
        </div>
      </article>
    </div>
  );
}

function SunIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <circle cx="12" cy="12" r="4" />
      <path d="M12 2.5v2.5M12 19v2.5M2.5 12H5M19 12h2.5M5 5l1.8 1.8M17.2 17.2 19 19M19 5l-1.8 1.8M6.8 17.2 5 19" />
    </svg>
  );
}

function MoonIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M16 3.2A8.5 8.5 0 1 0 20.8 15 7 7 0 0 1 16 3.2Z" />
    </svg>
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
    description: draft.description,
    status: draft.status,
    startAt: fromLocalInput(draft.startAt),
    deadlineAt: fromLocalInput(draft.deadlineAt),
    snoozeUntil: null,
    lastStartNotifiedAt: null,
    lastDeadlineNotifiedAt: null,
    createdAt: "",
    completedAt: null,
    recurrence: null,
  };
}
