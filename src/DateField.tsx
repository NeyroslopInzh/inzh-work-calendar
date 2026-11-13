import { useRef, useState } from "react";
import { monthGrid } from "./time";

const KEYS = ["day", "month", "year", "hour", "minute"] as const;
type Key = (typeof KEYS)[number];

const LABEL: Record<Key, string> = {
  day: "день",
  month: "месяц",
  year: "год",
  hour: "часы",
  minute: "минуты",
};

const PLACE: Record<Key, string> = {
  day: "дд",
  month: "мм",
  year: "гггг",
  hour: "чч",
  minute: "мм",
};

type Parts = Record<Key, string>;

const EMPTY: Parts = { day: "", month: "", year: "", hour: "", minute: "" };

function split(value: string): Parts {
  if (!value) return { ...EMPTY };
  const [date, time = ""] = value.split("T");
  const [year = "", month = "", day = ""] = date.split("-");
  const [hour = "", minute = ""] = time.split(":");
  return { day, month, year, hour, minute };
}

function join(parts: Parts, withTime: boolean): string {
  const needed: Key[] = withTime ? [...KEYS] : ["day", "month", "year"];
  if (needed.some((key) => parts[key].length !== (key === "year" ? 4 : 2))) return "";
  const day = Number(parts.day);
  const month = Number(parts.month);
  const year = Number(parts.year);
  const hour = withTime ? Number(parts.hour) : 0;
  const minute = withTime ? Number(parts.minute) : 0;
  if (month < 1 || month > 12 || hour > 23 || minute > 59 || year < 1970) return "";
  const date = new Date(year, month - 1, day, hour, minute);
  if (date.getFullYear() !== year || date.getMonth() !== month - 1 || date.getDate() !== day) return "";
  const pad = (n: number) => String(n).padStart(2, "0");
  const stamp = `${year}-${pad(month)}-${pad(day)}`;
  return withTime ? `${stamp}T${pad(hour)}:${pad(minute)}` : stamp;
}

export function DateField({
  value,
  onChange,
  withTime = true,
}: {
  value: string;
  onChange: (value: string) => void;
  withTime?: boolean;
}) {
  const keys = withTime ? KEYS : KEYS.slice(0, 3);
  const [parts, setParts] = useState<Parts>(() => split(value));
  const [open, setOpen] = useState(false);
  const [cursor, setCursor] = useState(() => new Date());
  const refs = useRef<Array<HTMLInputElement | null>>([]);

  function commit(next: Parts, focusIndex?: number) {
    setParts(next);
    onChange(join(next, withTime));
    if (focusIndex === undefined) return;
    const node = refs.current[focusIndex];
    if (!node) return;
    node.focus();
    node.select();
  }

  function pick(day: Date) {
    const pad = (n: number) => String(n).padStart(2, "0");
    commit({
      ...parts,
      day: pad(day.getDate()),
      month: pad(day.getMonth() + 1),
      year: String(day.getFullYear()),
    });
    setOpen(false);
  }

  const monthTitle = cursor.toLocaleDateString("ru-RU", { month: "long", year: "numeric" }).replace(" г.", "");

  return (
    <div className="segments">
      {keys.map((key, index) => (
        <span key={key} className="segment">
          {index === 1 || index === 2 ? <i>.</i> : null}
          {index === 4 ? <i>:</i> : null}
          {index === 3 ? <i className="gap" /> : null}
          <input
            ref={(node) => {
              refs.current[index] = node;
            }}
            inputMode="numeric"
            aria-label={LABEL[key]}
            placeholder={PLACE[key]}
            value={parts[key]}
            size={key === "year" ? 4 : 2}
            onChange={(event) => {
              const max = key === "year" ? 4 : 2;
              const digits = event.target.value.replace(/\D/g, "").slice(0, max);
              const next = { ...parts, [key]: digits };
              commit(next, digits.length === max ? index + 1 : undefined);
            }}
            onKeyDown={(event) => {
              if (event.key !== "Delete" && event.key !== "Backspace") return;
              event.preventDefault();
              commit({ ...parts, [key]: "" }, index + 1);
            }}
          />
        </span>
      ))}
      <button type="button" className="cal-button" aria-label="Открыть календарь" onClick={() => setOpen((value) => !value)}>
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <rect x="4" y="5" width="16" height="15" rx="2" />
          <path d="M8 3.5v3M16 3.5v3M4 9.5h16" />
        </svg>
      </button>
      {open && (
        <div className="picker">
          <div className="picker-head">
            <button type="button" onClick={() => setCursor(new Date(cursor.getFullYear(), cursor.getMonth() - 1, 1))}>
              ←
            </button>
            <b>{monthTitle}</b>
            <button type="button" onClick={() => setCursor(new Date(cursor.getFullYear(), cursor.getMonth() + 1, 1))}>
              →
            </button>
          </div>
          <div className="picker-grid">
            {monthGrid(cursor).map((day) => (
              <button
                key={day.toISOString()}
                type="button"
                className={day.getMonth() === cursor.getMonth() ? "" : "outside"}
                onClick={() => pick(day)}
              >
                {day.getDate()}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
