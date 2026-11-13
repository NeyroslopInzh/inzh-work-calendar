import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { openUrl } from "@tauri-apps/plugin-opener";
import { updateKind, type UpdateKind } from "./api";

const RELEASES = "https://github.com/NeyroslopInzh/inzh-work-calendar/releases/latest";

export type UpdateOffer = {
  version: string;
  kind: UpdateKind;
};

export async function findUpdate(): Promise<UpdateOffer | null> {
  const [kind, found] = await Promise.all([updateKind(), check()]);
  if (!found) return null;
  const offer = { version: found.version, kind };
  await found.close();
  return offer;
}

export async function applyUpdate(kind: UpdateKind): Promise<void> {
  if (kind === "browser") {
    await openUrl(RELEASES);
    return;
  }
  const found = await check();
  if (!found) return;
  await found.downloadAndInstall();
  if (kind === "appimage") await relaunch();
}
