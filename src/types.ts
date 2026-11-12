export type Status = "todo" | "doing" | "done";

export type Card = {
  id: string;
  title: string;
  status: Status;
  startAt: string | null;
  deadlineAt: string | null;
  snoozeUntil: string | null;
  lastStartNotifiedAt: string | null;
  lastDeadlineNotifiedAt: string | null;
  createdAt: string;
  completedAt: string | null;
};

export type CardInput = {
  id: string | null;
  title: string;
  status: Status;
  startAt: string | null;
  deadlineAt: string | null;
};

export type Settings = {
  autostart: boolean;
};

export type Placement = "start" | "deadline" | "both";

export const COLUMNS: { id: Status; title: string }[] = [
  { id: "todo", title: "К выполнению" },
  { id: "doing", title: "В работе" },
  { id: "done", title: "Готово" },
];
