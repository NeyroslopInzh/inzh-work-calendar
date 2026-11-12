import type { Card, Placement } from "./types";

const pad = (value: number) => String(value).padStart(2, "0");

export function dayKey(date: Date): string {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

export function parseDay(iso: string | null): string | null {
  if (!iso) return null;
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return null;
  return dayKey(date);
}

export function toLocalInput(iso: string | null): string {
  if (!iso) return "";
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return "";
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

export function fromLocalInput(value: string): string | null {
  if (!value) return null;
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return null;
  return date.toISOString();
}

export function shiftToDay(iso: string, target: Date): string {
  const source = new Date(iso);
  const next = new Date(target);
  next.setHours(source.getHours(), source.getMinutes(), 0, 0);
  return next.toISOString();
}

export function monthGrid(anchor: Date): Date[] {
  const first = new Date(anchor.getFullYear(), anchor.getMonth(), 1);
  const offset = (first.getDay() + 6) % 7;
  const start = new Date(first);
  start.setDate(first.getDate() - offset);
  return Array.from({ length: 42 }, (_, index) => {
    const day = new Date(start);
    day.setDate(start.getDate() + index);
    return day;
  });
}

export function monthLabel(anchor: Date): string {
  const label = new Intl.DateTimeFormat("ru-RU", {
    month: "long",
    year: "numeric",
  }).format(anchor);
  return label.replace(" г.", "");
}

export function formatTime(iso: string): string {
  return new Intl.DateTimeFormat("ru-RU", {
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(iso));
}

export function formatDayTime(iso: string): string {
  return new Intl.DateTimeFormat("ru-RU", {
    day: "numeric",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(iso));
}

export function formatLate(iso: string, now: Date): string {
  const minutes = Math.max(0, Math.round((now.getTime() - new Date(iso).getTime()) / 60000));
  if (minutes < 1) return "меньше минуты";
  if (minutes < 60) return `${minutes} мин`;
  if (minutes < 60 * 48) return `${Math.floor(minutes / 60)} ч`;
  return `${Math.floor(minutes / (60 * 24))} д`;
}

export function placement(card: Card, day: Date): Placement | null {
  const key = dayKey(day);
  const start = parseDay(card.startAt) === key;
  const deadline = parseDay(card.deadlineAt) === key;
  if (start && deadline) return "both";
  if (start) return "start";
  if (deadline) return "deadline";
  return null;
}

export function isSnoozed(card: Card, now: Date): boolean {
  return Boolean(card.snoozeUntil && new Date(card.snoozeUntil).getTime() > now.getTime());
}

export function isDeadlineDue(card: Card, now: Date): boolean {
  return (
    card.status !== "done" &&
    Boolean(card.deadlineAt && new Date(card.deadlineAt).getTime() <= now.getTime())
  );
}

export function isStartDue(card: Card, now: Date): boolean {
  if (card.status === "done" || !card.startAt) return false;
  if (new Date(card.startAt).getTime() > now.getTime()) return false;
  if (card.deadlineAt && new Date(card.deadlineAt).getTime() <= now.getTime()) return false;
  return true;
}

export function isNagging(card: Card, now: Date): boolean {
  if (isSnoozed(card, now)) return false;
  return isDeadlineDue(card, now) || isStartDue(card, now);
}

export function agendaStamp(card: Card, day: Date): string | null {
  const where = placement(card, day);
  if (!where) return null;
  if (where === "start" && card.startAt) return card.startAt;
  if (where === "deadline" && card.deadlineAt) return card.deadlineAt;
  const times = [card.startAt, card.deadlineAt].filter((value): value is string => Boolean(value));
  times.sort((a, b) => new Date(a).getTime() - new Date(b).getTime());
  return times[0] ?? null;
}
