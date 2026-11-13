# Full instructions

## Project layout

| Folder | Role |
|--------|------|
| `src/` | UI: React and TypeScript |
| `src-tauri/` | App: Rust, Tauri, tray, reminders, local task intake |
| `docs/` | Docs in five languages |
| `.github/workflows/` | Builds `.exe`, `.deb`, and AppImage on a `v*` tag |

---

## Windows

### Installer

[Releases](https://github.com/NeyroslopInzh/inzh-work-calendar/releases) → `inzh.work.calendar_*_x64-setup.exe`.

`x64` is a normal 64-bit CPU, Intel or AMD. The installer will not run on ARM.

### Source / build

Node.js and Rust are required.

```powershell
npm install
npm run tauri build
```

The installer lands in `src-tauri/target/release/bundle/nsis/`.

---

## Linux

### Build requirements

WebKitGTK 4.1 and the Tauri libraries:

```sh
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
npm install
npm run tauri build
```

The `.deb` is in `src-tauri/target/release/bundle/deb/`, the AppImage in `src-tauri/target/release/bundle/appimage/`.

### Ready files

```bash
sudo apt install ./inzh.work.calendar_*_amd64.deb
```

```bash
chmod +x inzh.work.calendar_*_amd64.AppImage
./inzh.work.calendar_*_amd64.AppImage
```

`amd64` is the same 64-bit CPU as `x64`. The binary will not run on ARM.

If the AppImage says FUSE is missing, install `libfuse2`.

---

## Window and tray

Closing the window hides the calendar in the tray. A left click on the icon, or “Открыть”, brings the window back. Quit is “Выход”. The first launch turns autostart on; the header can turn it off. The header also switches the dark and light theme.

---

## Tasks

| Field | Meaning |
|-------|---------|
| Title | Required |
| Description | Optional |
| Start | Optional. Reminder only while the status is “К выполнению” |
| Deadline | Optional. Nags until the task is in “Готово” |
| Repeat | Weekdays inside a date range, at most three years |
| Status | “К выполнению”, “В работе”, “Готово”, or a custom one between the first two and “Готово” |

A reminder fires at most once every 10 minutes. Once the deadline has passed, the start reminder goes quiet and only the deadline nags. Snooze can be 15 minutes, 1 hour, 3 hours, 1 day, or a custom number of minutes, and never more than one day.

A card can be dragged onto another day or another column. “К выполнению” and “Готово” cannot be removed. Deleting a custom status sends its tasks back to “К выполнению”.

Repeat days: 1 is Monday … 7 is Sunday. The chosen range must contain at least one of those days.

---

## Data

| OS | Directory |
|----|-----------|
| Windows | `%APPDATA%\com.inzh.workcalendar\` |
| Linux | `~/.local/share/com.inzh.workcalendar/` |

If `XDG_DATA_HOME` is set on Linux, the directory is `$XDG_DATA_HOME/com.inzh.workcalendar/`.

| File | Contents |
|------|----------|
| `cards.json` | Tasks |
| `settings.json` | Theme, autostart, columns |
| `endpoint.json` | `base`, `port`, `token` |
| `inbox.jsonl` | Tasks written while the app was closed |
| `inbox.failed.jsonl` | Lines that could not be parsed |

---

## Local task intake

The app listens only on `127.0.0.1`. The port is chosen from `47321`–`47330` and written to `endpoint.json` with the token.

Every request needs the header `Authorization: Bearer {token}`.

| Request | Action |
|---------|--------|
| `GET {base}/health` | `{ "ok": true }` |
| `GET {base}/tasks` | List tasks |
| `POST {base}/tasks` | Create |
| `PATCH {base}/tasks/{id}` | Update. `null` clears a date or the repeat |
| `DELETE {base}/tasks/{id}` | Delete |

Create example:

```json
{
  "title": "помыться",
  "description": "",
  "startAt": "2026-11-13T21:00",
  "deadlineAt": "2026-11-14T09:00",
  "status": "todo",
  "recurrence": {
    "from": "2026-11-13",
    "until": "2026-12-13",
    "weekdays": [1, 3, 5]
  }
}
```

Time is local `YYYY-MM-DDTHH:MM`, `YYYY-MM-DD HH:MM`, `YYYY-MM-DDTHH:MM:SS`, or RFC3339. `startAt`, `deadlineAt`, `description`, and `recurrence` may be omitted. The default `status` is `todo`.

If the app is closed, append one JSON line to `inbox.jsonl` in the same directory. On launch the calendar consumes the file. A broken line stays in `inbox.failed.jsonl`; the rest are applied.

---

## Common cases

### The window “closed” but the process is still there

That is the tray. Quit with “Выход” on the icon menu.

### The local address did not open

Ports `47321`–`47330` are taken. While the app is running, `endpoint.json` will not be refreshed. A task can still be appended to `inbox.jsonl`.

### It does not start on ARM

Releases are `amd64` / `x64` only.

### The AppImage will not start

FUSE is often missing. Install `libfuse2` and run the file again.

---

## Release build

```bash
git tag v0.2.2
git push origin v0.2.2
```

GitHub Actions builds Windows (NSIS) and Linux (`.deb`, AppImage). The workflow is `.github/workflows/release.yml`.

---

## License

[GNU GPL v3](../LICENSE) — copyleft. A changed copy you give to someone else must be released under the same terms.
