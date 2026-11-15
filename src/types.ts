export type Column = {
  id: string;
  title: string;
};

export type Recurrence = {
  from: string;
  until: string;
  weekdays: number[];
};

export type TelegramRecipient = {
  id: string;
  chatId: number;
  name: string;
  username: string;
};

export type Card = {
  id: string;
  title: string;
  description: string;
  status: string;
  startAt: string | null;
  deadlineAt: string | null;
  snoozeUntil: string | null;
  lastStartNotifiedAt: string | null;
  lastDeadlineNotifiedAt: string | null;
  createdAt: string;
  completedAt: string | null;
  recurrence: Recurrence | null;
  recipientIds: string[];
};

export type CardInput = {
  id: string | null;
  title: string;
  description: string;
  status: string;
  startAt: string | null;
  deadlineAt: string | null;
  recurrence: Recurrence | null;
  recipientIds: string[];
};

export type Settings = {
  autostart: boolean;
  theme: "dark" | "light";
  telegramToken: string;
  telegramBot: string;
  telegramInvite: string;
  telegramAlways: boolean;
  recipients: TelegramRecipient[];
};

export type Placement = "start" | "deadline" | "both";

export const TODO = "todo";
export const DONE = "done";

export const WEEKDAYS: { id: number; label: string }[] = [
  { id: 1, label: "пн" },
  { id: 2, label: "вт" },
  { id: 3, label: "ср" },
  { id: 4, label: "чт" },
  { id: 5, label: "пт" },
  { id: 6, label: "сб" },
  { id: 7, label: "вс" },
];
