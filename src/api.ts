import { invoke } from "@tauri-apps/api/core";
import type { Card, CardInput, Settings, Status } from "./types";

export function listCards(): Promise<Card[]> {
  return invoke("list_cards");
}

export function saveCard(input: CardInput): Promise<Card[]> {
  return invoke("save_card", { input });
}

export function setStatus(id: string, status: Status): Promise<Card[]> {
  return invoke("set_status", { id, status });
}

export function deleteCard(id: string): Promise<Card[]> {
  return invoke("delete_card", { id });
}

export function snoozeCard(id: string, minutes: number): Promise<Card[]> {
  return invoke("snooze_card", { id, minutes });
}

export function getSettings(): Promise<Settings> {
  return invoke("get_settings");
}

export function setAutostart(enabled: boolean): Promise<Settings> {
  return invoke("set_autostart", { enabled });
}
