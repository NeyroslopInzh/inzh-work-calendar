# inzh work calendar

Календарь и канбан для Windows и Linux. У задачи есть необязательное начало и дедлайн. Пока она не в колонке «Готово», система напоминает каждые 10 минут. Дедлайн можно отложить не больше чем на сутки.

Copyright (C) 2026 NeyroslopInzh

Это свободная программа под GNU GPL версии 3 или новее. Если вы её меняете и отдаёте кому-то копию, исходники этих изменений нужно отдать на тех же условиях. Текст лицензии — в файле `LICENSE`.

## Сборка

Нужны Node.js, Rust и системные библиотеки Tauri.

Linux (Debian/Ubuntu):

```sh
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
npm install
npm run tauri build
```

Windows:

```powershell
npm install
npm run tauri build
```

Окно закрывается в трей. Выход — из меню трея. При первом запуске включается автозапуск.

## Данные

Задачи лежат рядом с `endpoint.json`:

- Linux: `~/.local/share/com.inzh.workcalendar/`
- Windows: `%APPDATA%\com.inzh.workcalendar\`

Пока приложение запущено, оно принимает задачи только с `127.0.0.1`. Порт и токен — в `endpoint.json`.

Создать задачу: `POST {base}/tasks` с заголовком `Authorization: Bearer {token}`.

```json
{ "title": "помыться", "startAt": "2026-11-13T21:00", "deadlineAt": "2026-11-14T09:00", "status": "todo" }
```

`startAt` и `deadlineAt` необязательны. Время — местное `YYYY-MM-DDTHH:MM` или RFC3339. `status`: `todo`, `doing`, `done`.

Список: `GET {base}/tasks`. Изменить: `PATCH {base}/tasks/{id}` (`null` снимает дату). Удалить: `DELETE {base}/tasks/{id}`.

Если приложение выключено, допишите одну JSON-строку в `inbox.jsonl` в той же папке. Календарь заберёт её при запуске.
