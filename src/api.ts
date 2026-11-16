import { invoke } from "@tauri-apps/api/core";
import type { Card, CardInput, Column, Settings } from "./types";

export function listCards(): Promise<Card[]> {
  return invoke("list_cards");
}

export function saveCard(input: CardInput): Promise<Card[]> {
  return invoke("save_card", { input });
}

export function setStatus(id: string, status: string): Promise<Card[]> {
  return invoke("set_status", { id, status });
}

export function listColumns(): Promise<Column[]> {
  return invoke("list_columns");
}

export function addColumn(title: string): Promise<Column[]> {
  return invoke("add_column", { title });
}

export function removeColumn(id: string): Promise<{ columns: Column[]; cards: Card[] }> {
  return invoke("remove_column", { id });
}

export function deleteCard(id: string): Promise<Card[]> {
  return invoke("delete_card", { id });
}

export function snoozeCard(id: string, minutes: number): Promise<Card[]> {
  return invoke("snooze_card", { id, minutes });
}

export function hideNag(): Promise<void> {
  return invoke("hide_nag");
}

export function getSettings(): Promise<Settings> {
  return invoke("get_settings");
}

export function setAutostart(enabled: boolean): Promise<Settings> {
  return invoke("set_autostart", { enabled });
}

export function setTheme(theme: "dark" | "light"): Promise<Settings> {
  return invoke("set_theme", { theme });
}

export function setTelegramToken(token: string): Promise<Settings> {
  return invoke("set_telegram_token", { token });
}

export function setRecipientAlways(id: string, enabled: boolean): Promise<Settings> {
  return invoke("set_recipient_always", { id, enabled });
}

export function removeTelegramRecipient(id: string): Promise<Settings> {
  return invoke("remove_telegram_recipient", { id });
}

export function prepareTelegramInvite(): Promise<Settings> {
  return invoke("prepare_telegram_invite");
}

export type UpdateKind = "windows" | "appimage" | "browser";

export function updateKind(): Promise<UpdateKind> {
  return invoke("update_kind");
}
