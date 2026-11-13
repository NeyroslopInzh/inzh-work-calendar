# inzh work calendar

<details open>
<summary>Русский</summary>

Календарь и канбан для Windows и Linux на одних и тех же карточках. У задачи есть название, необязательные начало, дедлайн и описание, повтор по дням недели. Напоминание приходит тостом системы и отдельным окном поверх остальных: окно можно сдвинуть, закрыть или отложить. Просроченное лежит полоской под календарём и под доской, пока задачу не закроют или не отложат.

**Лицензия:** [GNU GPL v3](LICENSE) — copyleft, форки только под GPL.

Сборка `amd64` / `x64` — обычный 64-битный процессор: и Intel, и AMD. На ARM она не запустится.

Copyright (C) 2026 NeyroslopInzh

</details>

<details>
<summary>English</summary>

A calendar and kanban for Windows and Linux, on the same cards. A task has a title, an optional start, deadline and description, and a weekday repeat. A reminder arrives as a system toast and as its own window above the others: that window can be moved, closed, or snoozed. Overdue work sits in a strip under the calendar and under the board until the task is finished or snoozed.

**License:** [GNU GPL v3](LICENSE) — copyleft; forks must stay under GPL.

The `amd64` / `x64` build is a normal 64-bit CPU, Intel or AMD. It will not run on ARM.

Copyright (C) 2026 NeyroslopInzh

</details>

<details>
<summary>עברית</summary>

<div dir="rtl">

יומן וקאנבן ל-Windows ול-Linux, על אותם כרטיסים. למשימה יש שם, והתחלה, דדליין, תיאור וחזרה לפי ימי השבוע — כולם אופציונליים. תזכורת מגיעה כהתראת מערכת וכחלון נפרד מעל השאר: אפשר להזיז אותו, לסגור או לדחות. משימה באיחור נשארת בפס מתחת ליומן ומתחת ללוח עד שסוגרים אותה או דוחים.

**רישיון:** [GNU GPL v3](LICENSE) — copyleft; forks חייבים להישאר תחת GPL.

בניית `amd64` / `x64` היא מעבד 64-ביט רגיל, Intel או AMD. על ARM זה לא ירוץ.

Copyright (C) 2026 NeyroslopInzh

</div>

</details>

<details>
<summary>हिन्दी</summary>

Windows और Linux के लिए कैलेंडर और कानबान, उन्हीं कार्डों पर। टास्क में नाम होता है, और वैकल्पिक शुरुआत, डेडलाइन, विवरण और हफ़्ते के दिनों का दोहराव। याद सिस्टम टोस्ट से आती है और बाकी विंडो के ऊपर एक अलग विंडो में: उसे खिसकाया, बंद किया या टाला जा सकता है। देर हुई टास्क कैलेंडर और बोर्ड के नीचे पट्टी में रहती है, जब तक उसे पूरा या टाल न दिया जाए।

**लाइसेंस:** [GNU GPL v3](LICENSE) — copyleft; forks सिर्फ GPL के तहत।

`amd64` / `x64` बिल्ड सामान्य 64-बिट प्रोसेसर है, Intel या AMD। ARM पर नहीं चलेगा।

Copyright (C) 2026 NeyroslopInzh

</details>

<details>
<summary>Oʻzbekcha</summary>

Windows va Linux uchun kalendar va kanban, bir xil kartochkalarda. Vazifada nom bor, ixtiyoriy boshlanish, dedlayn, tavsif va hafta kunlari bo‘yicha takror. Eslatma tizim tosti va qolgan oynalar ustidagi alohida oyna orqali keladi: oynani surish, yopish yoki kechiktirish mumkin. Muddati o‘tgan vazifa kalendar va doska ostidagi tasmada turadi, toki uni yopishmasa yoki surishmasa.

**Litsenziya:** [GNU GPL v3](LICENSE) — copyleft; forklar faqat GPL ostida.

`amd64` / `x64` yig‘imi oddiy 64-bit protsessor: Intel yoki AMD. ARM da ishlamaydi.

Copyright (C) 2026 NeyroslopInzh

</details>

---

## Структура репозитория

<details open>
<summary>Русский</summary>

```
├── src/                 # интерфейс: React и TypeScript
├── src-tauri/           # приложение: Rust и Tauri
├── docs/                # инструкции (5 языков)
├── .github/workflows/   # сборка Windows и Linux
└── LICENSE
```

Бинарники — в [Releases](https://github.com/NeyroslopInzh/inzh-work-calendar/releases).

</details>

<details>
<summary>English</summary>

```
├── src/                 # UI: React and TypeScript
├── src-tauri/           # app: Rust and Tauri
├── docs/                # docs (5 languages)
├── .github/workflows/   # Windows and Linux builds
└── LICENSE
```

Binaries are in [Releases](https://github.com/NeyroslopInzh/inzh-work-calendar/releases).

</details>

<details>
<summary>עברית</summary>

<div dir="rtl">

```
├── src/                 # ממשק: React ו-TypeScript
├── src-tauri/           # אפליקציה: Rust ו-Tauri
├── docs/                # הוראות (5 שפות)
├── .github/workflows/   # בניות Windows ו-Linux
└── LICENSE
```

בינאריים — ב-[Releases](https://github.com/NeyroslopInzh/inzh-work-calendar/releases).

</div>

</details>

<details>
<summary>हिन्दी</summary>

```
├── src/                 # UI: React और TypeScript
├── src-tauri/           # ऐप: Rust और Tauri
├── docs/                # निर्देश (5 भाषाएँ)
├── .github/workflows/   # Windows और Linux बिल्ड
└── LICENSE
```

बाइनरी — [Releases](https://github.com/NeyroslopInzh/inzh-work-calendar/releases) में।

</details>

<details>
<summary>Oʻzbekcha</summary>

```
├── src/                 # interfeys: React va TypeScript
├── src-tauri/           # ilova: Rust va Tauri
├── docs/                # ko‘rsatmalar (5 til)
├── .github/workflows/   # Windows va Linux yig‘imlari
└── LICENSE
```

Binariylar — [Releases](https://github.com/NeyroslopInzh/inzh-work-calendar/releases) da.

</details>

---

## Что скачать

<details open>
<summary>Русский</summary>

| Нужно | Windows | Linux |
|-------|---------|-------|
| **Установщик** | `inzh.work.calendar_*_x64-setup.exe` | `inzh.work.calendar_*_amd64.deb` |
| **Один файл** | — | `inzh.work.calendar_*_amd64.AppImage` |

`*` — номер версии, как у тега релиза. `amd64` и `x64` — один и тот же 64-битный процессор.

Окно закрывается в трей. Выход — пункт «Выход» в меню трея. При первом запуске включается автозапуск.

</details>

<details>
<summary>English</summary>

| Need | Windows | Linux |
|------|---------|-------|
| **Installer** | `inzh.work.calendar_*_x64-setup.exe` | `inzh.work.calendar_*_amd64.deb` |
| **Single file** | — | `inzh.work.calendar_*_amd64.AppImage` |

`*` is the version, matching the release tag. `amd64` and `x64` are the same 64-bit CPU.

Closing the window sends it to the tray. Quit is the “Выход” item in the tray menu. The first launch turns autostart on.

</details>

<details>
<summary>עברית</summary>

<div dir="rtl">

| צורך | Windows | Linux |
|------|---------|-------|
| **מתקין** | `inzh.work.calendar_*_x64-setup.exe` | `inzh.work.calendar_*_amd64.deb` |
| **קובץ אחד** | — | `inzh.work.calendar_*_amd64.AppImage` |

`*` הוא מספר הגרסה, כמו בתגית הריליז. `amd64` ו-`x64` הם אותו מעבד 64-ביט.

סגירת החלון שולחת אותו למגש. יציאה — הפריט «Выход» בתפריט המגש. בהפעלה הראשונה נדלקת הפעלה אוטומטית.

</div>

</details>

<details>
<summary>हिन्दी</summary>

| ज़रूरत | Windows | Linux |
|--------|---------|-------|
| **इंस्टॉलर** | `inzh.work.calendar_*_x64-setup.exe` | `inzh.work.calendar_*_amd64.deb` |
| **एक फ़ाइल** | — | `inzh.work.calendar_*_amd64.AppImage` |

`*` वर्शन नंबर है, रिलीज़ टैग जैसा। `amd64` और `x64` एक ही 64-बिट प्रोसेसर हैं।

विंडो बंद करने पर ऐप ट्रे में चला जाता है। बाहर निकलना ट्रे मेनू का «Выход» है। पहली बार चलाने पर ऑटोस्टार्ट चालू होता है।

</details>

<details>
<summary>Oʻzbekcha</summary>

| Kerak | Windows | Linux |
|-------|---------|-------|
| **O‘rnatuvchi** | `inzh.work.calendar_*_x64-setup.exe` | `inzh.work.calendar_*_amd64.deb` |
| **Bitta fayl** | — | `inzh.work.calendar_*_amd64.AppImage` |

`*` — versiya raqami, reliz tegi bilan bir xil. `amd64` va `x64` — bitta 64-bit protsessor.

Oyna yopilsa, ilova treyga tushadi. Chiqish — trey menyusidagi «Выход». Birinchi ishga tushirishda avtostart yoqiladi.

</details>

---

## Windows

<details open>
<summary>Русский</summary>

### Установщик

[Releases](https://github.com/NeyroslopInzh/inzh-work-calendar/releases) → `inzh.work.calendar_*_x64-setup.exe` → двойной клик.

### Из исходников

Нужны Node.js и Rust.

```powershell
npm install
npm run tauri build
```

</details>

<details>
<summary>English</summary>

### Installer

[Releases](https://github.com/NeyroslopInzh/inzh-work-calendar/releases) → `inzh.work.calendar_*_x64-setup.exe` → double-click.

### From source

Node.js and Rust are required.

```powershell
npm install
npm run tauri build
```

</details>

<details>
<summary>עברית</summary>

<div dir="rtl">

### מתקין

[Releases](https://github.com/NeyroslopInzh/inzh-work-calendar/releases) → `inzh.work.calendar_*_x64-setup.exe` → לחיצה כפולה.

### מקוד מקור

נדרשים Node.js ו-Rust.

```powershell
npm install
npm run tauri build
```

</div>

</details>

<details>
<summary>हिन्दी</summary>

### इंस्टॉलर

[Releases](https://github.com/NeyroslopInzh/inzh-work-calendar/releases) → `inzh.work.calendar_*_x64-setup.exe` → डबल क्लिक।

### सोर्स से

Node.js और Rust चाहिए।

```powershell
npm install
npm run tauri build
```

</details>

<details>
<summary>Oʻzbekcha</summary>

### O‘rnatuvchi

[Releases](https://github.com/NeyroslopInzh/inzh-work-calendar/releases) → `inzh.work.calendar_*_x64-setup.exe` → ikki marta bosing.

### Manbadan

Node.js va Rust kerak.

```powershell
npm install
npm run tauri build
```

</details>

---

## Linux

<details open>
<summary>Русский</summary>

**Для сборки из исходников** нужны WebKitGTK 4.1 и остальные библиотеки Tauri. Готовые `.deb` и AppImage их приносят с собой, кроме FUSE у AppImage: на части дистрибутивов нужен пакет `libfuse2`.

### Готовые файлы

```bash
sudo apt install ./inzh.work.calendar_*_amd64.deb
```

```bash
chmod +x inzh.work.calendar_*_amd64.AppImage
./inzh.work.calendar_*_amd64.AppImage
```

### Из исходников

```sh
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
npm install
npm run tauri build
```

</details>

<details>
<summary>English</summary>

**Building from source** needs WebKitGTK 4.1 and the other Tauri libraries. The `.deb` and AppImage already include them, except AppImage FUSE: some distros need the `libfuse2` package.

### Ready files

```bash
sudo apt install ./inzh.work.calendar_*_amd64.deb
```

```bash
chmod +x inzh.work.calendar_*_amd64.AppImage
./inzh.work.calendar_*_amd64.AppImage
```

### From source

```sh
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
npm install
npm run tauri build
```

</details>

<details>
<summary>עברית</summary>

<div dir="rtl">

**בנייה מקוד מקור** דורשת WebKitGTK 4.1 ושאר ספריות Tauri. קובצי `.deb` ו-AppImage כבר כוללים אותן, חוץ מ-FUSE של AppImage: בחלק מההפצות צריך את החבילה `libfuse2`.

### קבצים מוכנים

```bash
sudo apt install ./inzh.work.calendar_*_amd64.deb
```

```bash
chmod +x inzh.work.calendar_*_amd64.AppImage
./inzh.work.calendar_*_amd64.AppImage
```

### מקוד מקור

```sh
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
npm install
npm run tauri build
```

</div>

</details>

<details>
<summary>हिन्दी</summary>

**सोर्स से बिल्ड** के लिए WebKitGTK 4.1 और बाकी Tauri लाइब्रेरी चाहिए। `.deb` और AppImage उन्हें साथ लाते हैं, AppImage के FUSE को छोड़कर: कुछ डिस्ट्रो पर `libfuse2` पैकेज चाहिए।

### तैयार फ़ाइलें

```bash
sudo apt install ./inzh.work.calendar_*_amd64.deb
```

```bash
chmod +x inzh.work.calendar_*_amd64.AppImage
./inzh.work.calendar_*_amd64.AppImage
```

### सोर्स से

```sh
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
npm install
npm run tauri build
```

</details>

<details>
<summary>Oʻzbekcha</summary>

**Manbadan yig‘ish** uchun WebKitGTK 4.1 va qolgan Tauri kutubxonalari kerak. `.deb` va AppImage ularni o‘zi olib keladi, AppImage FUSE dan tashqari: ba’zi distributivlarda `libfuse2` paketi kerak.

### Tayyor fayllar

```bash
sudo apt install ./inzh.work.calendar_*_amd64.deb
```

```bash
chmod +x inzh.work.calendar_*_amd64.AppImage
./inzh.work.calendar_*_amd64.AppImage
```

### Manbadan

```sh
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
npm install
npm run tauri build
```

</details>

---

## Данные

<details open>
<summary>Русский</summary>

Задачи лежат рядом с `endpoint.json`:

| ОС | Каталог |
|----|---------|
| **Windows** | `%APPDATA%\com.inzh.workcalendar\` |
| **Linux** | `~/.local/share/com.inzh.workcalendar/` |

На Linux при заданном `XDG_DATA_HOME` каталог — `$XDG_DATA_HOME/com.inzh.workcalendar/`.

Пока приложение запущено, оно принимает задачи только с `127.0.0.1`. Порт (из диапазона `47321`–`47330`) и токен — в `endpoint.json`.

Создать задачу: `POST {base}/tasks` с заголовком `Authorization: Bearer {token}`.

```json
{
  "title": "помыться",
  "startAt": "2026-11-13T21:00",
  "deadlineAt": "2026-11-14T09:00",
  "status": "todo"
}
```

`startAt`, `deadlineAt`, `description` и `recurrence` необязательны. Время — местное `YYYY-MM-DDTHH:MM` или RFC3339. `status`: `todo`, `doing`, `done` или id своей колонки.

Список: `GET {base}/tasks`. Изменить: `PATCH {base}/tasks/{id}` (`null` снимает дату или повтор). Удалить: `DELETE {base}/tasks/{id}`.

Если приложение выключено, допишите одну JSON-строку в `inbox.jsonl` в той же папке. Календарь заберёт её при запуске.

Подробности — в [docs/INSTRUCTIONS.md](docs/INSTRUCTIONS.md).

</details>

<details>
<summary>English</summary>

Tasks live next to `endpoint.json`:

| OS | Directory |
|----|-----------|
| **Windows** | `%APPDATA%\com.inzh.workcalendar\` |
| **Linux** | `~/.local/share/com.inzh.workcalendar/` |

On Linux, if `XDG_DATA_HOME` is set, the directory is `$XDG_DATA_HOME/com.inzh.workcalendar/`.

While the app is running it accepts tasks only from `127.0.0.1`. The port (from `47321`–`47330`) and the token are in `endpoint.json`.

Create a task: `POST {base}/tasks` with header `Authorization: Bearer {token}`.

```json
{
  "title": "помыться",
  "startAt": "2026-11-13T21:00",
  "deadlineAt": "2026-11-14T09:00",
  "status": "todo"
}
```

`startAt`, `deadlineAt`, `description`, and `recurrence` are optional. Time is local `YYYY-MM-DDTHH:MM` or RFC3339. `status`: `todo`, `doing`, `done`, or a custom column id.

List: `GET {base}/tasks`. Update: `PATCH {base}/tasks/{id}` (`null` clears a date or the repeat). Delete: `DELETE {base}/tasks/{id}`.

If the app is closed, append one JSON line to `inbox.jsonl` in the same folder. The calendar picks it up on launch.

Details: [docs/INSTRUCTIONS.en.md](docs/INSTRUCTIONS.en.md).

</details>

<details>
<summary>עברית</summary>

<div dir="rtl">

המשימות נשמרות ליד `endpoint.json`:

| מערכת | תיקייה |
|-------|--------|
| **Windows** | `%APPDATA%\com.inzh.workcalendar\` |
| **Linux** | `~/.local/share/com.inzh.workcalendar/` |

ב-Linux עם `XDG_DATA_HOME` התיקייה היא `$XDG_DATA_HOME/com.inzh.workcalendar/`.

כשהאפליקציה רצה היא מקבלת משימות רק מ-`127.0.0.1`. הפורט (מתוך `47321`–`47330`) והטוקן נמצאים ב-`endpoint.json`.

יצירת משימה: `POST {base}/tasks` עם הכותרת `Authorization: Bearer {token}`.

```json
{
  "title": "помыться",
  "startAt": "2026-11-13T21:00",
  "deadlineAt": "2026-11-14T09:00",
  "status": "todo"
}
```

`startAt`, `deadlineAt`, `description` ו-`recurrence` אופציונליים. הזמן מקומי: `YYYY-MM-DDTHH:MM` או RFC3339. `status`: `todo`, `doing`, `done` או מזהה עמודה משלך.

רשימה: `GET {base}/tasks`. עדכון: `PATCH {base}/tasks/{id}` (`null` מוחק תאריך או חזרה). מחיקה: `DELETE {base}/tasks/{id}`.

אם האפליקציה כבויה, הוסיפו שורת JSON אחת ל-`inbox.jsonl` באותה תיקייה. היומן יאסוף אותה בהפעלה.

פירוט: [docs/INSTRUCTIONS.he.md](docs/INSTRUCTIONS.he.md).

</div>

</details>

<details>
<summary>हिन्दी</summary>

टास्क `endpoint.json` के पास रहते हैं:

| OS | डायरेक्टरी |
|----|-------------|
| **Windows** | `%APPDATA%\com.inzh.workcalendar\` |
| **Linux** | `~/.local/share/com.inzh.workcalendar/` |

Linux पर `XDG_DATA_HOME` सेट हो तो डायरेक्टरी `$XDG_DATA_HOME/com.inzh.workcalendar/` है।

ऐप चलते समय टास्क सिर्फ `127.0.0.1` से लेता है। पोर्ट (`47321`–`47330` में से) और टोकन `endpoint.json` में हैं।

टास्क बनाना: `POST {base}/tasks`, हेडर `Authorization: Bearer {token}`।

```json
{
  "title": "помыться",
  "startAt": "2026-11-13T21:00",
  "deadlineAt": "2026-11-14T09:00",
  "status": "todo"
}
```

`startAt`, `deadlineAt`, `description` और `recurrence` वैकल्पिक हैं। समय स्थानीय `YYYY-MM-DDTHH:MM` या RFC3339। `status`: `todo`, `doing`, `done`, या अपनी कॉलम की id।

सूची: `GET {base}/tasks`। बदलना: `PATCH {base}/tasks/{id}` (`null` तारीख या दोहराव हटाता है)। मिटाना: `DELETE {base}/tasks/{id}`।

ऐप बंद हो तो उसी फोल्डर की `inbox.jsonl` में एक JSON पंक्ति जोड़ो। कैलेंडर उसे चालू होते ही ले लेगा।

विवरण: [docs/INSTRUCTIONS.hi.md](docs/INSTRUCTIONS.hi.md)।

</details>

<details>
<summary>Oʻzbekcha</summary>

Vazifalar `endpoint.json` yonida yotadi:

| OT | Katalog |
|----|---------|
| **Windows** | `%APPDATA%\com.inzh.workcalendar\` |
| **Linux** | `~/.local/share/com.inzh.workcalendar/` |

Linux da `XDG_DATA_HOME` berilgan bo‘lsa, katalog — `$XDG_DATA_HOME/com.inzh.workcalendar/`.

Ilova ishlayotganda vazifalarni faqat `127.0.0.1` dan oladi. Port (`47321`–`47330` oralig‘idan) va token `endpoint.json` da.

Vazifa yaratish: `POST {base}/tasks`, sarlavha `Authorization: Bearer {token}`.

```json
{
  "title": "помыться",
  "startAt": "2026-11-13T21:00",
  "deadlineAt": "2026-11-14T09:00",
  "status": "todo"
}
```

`startAt`, `deadlineAt`, `description` va `recurrence` ixtiyoriy. Vaqt mahalliy `YYYY-MM-DDTHH:MM` yoki RFC3339. `status`: `todo`, `doing`, `done` yoki o‘z ustunning id si.

Ro‘yxat: `GET {base}/tasks`. O‘zgartirish: `PATCH {base}/tasks/{id}` (`null` sana yoki takrorni olib tashlaydi). O‘chirish: `DELETE {base}/tasks/{id}`.

Ilova o‘chiq bo‘lsa, shu papkadagi `inbox.jsonl` ga bitta JSON qator qo‘shing. Kalendar uni ishga tushganda oladi.

Batafsil: [docs/INSTRUCTIONS.uz.md](docs/INSTRUCTIONS.uz.md).

</details>

---

## Возможности

<details open>
<summary>Русский</summary>

- Календарь на месяц и доска со статусами, одни и те же карточки; на доске карточку перетаскивают в другую колонку
- Тост и окно поверх других окон: «Готово», «15 мин», «1 час», «3 часа», «Сутки». Окно двигается за заголовок и само закрывается, когда напоминать нечего
- Полоска просрочки под календарём и под доской снимается вместе с откладыванием
- Начало, дедлайн и описание необязательны
- Повтор по выбранным дням недели, диапазон не длиннее трёх лет
- Свои статусы между «К выполнению» и «Готово»
- Напоминание о начале — только в «К выполнению», каждые 10 минут
- Просроченный дедлайн заменяет напоминание о начале и стучит каждые 10 минут, пока задача не в «Готово»
- Напоминание можно отложить не больше чем на сутки
- Тёмная и светлая тема, трей, автозапуск
- Локальный вход задач на `127.0.0.1`, пока приложение запущено

</details>

<details>
<summary>English</summary>

- A month calendar and a status board of the same cards; a board card is dragged into another column
- A toast and a window above other windows: Done, 15 min, 1 hour, 3 hours, 1 day. The window moves by its title and closes itself when nothing is due
- The overdue strip under the calendar and the board clears together with a snooze
- Start, deadline, and description are optional
- Repeat on chosen weekdays, spanning at most three years
- Custom statuses between “К выполнению” and “Готово”
- Start reminder only while the task is in “К выполнению”, every 10 minutes
- A passed deadline replaces the start reminder and nags every 10 minutes until the task is in “Готово”
- A reminder can be snoozed for at most one day
- Dark and light theme, tray, autostart
- Local task intake on `127.0.0.1` while the app is running

</details>

<details>
<summary>עברית</summary>

<div dir="rtl">

- יומן חודשי ולוח סטטוסים של אותם כרטיסים; כרטיס בלוח נגרר לעמודה אחרת
- התראת מערכת וחלון מעל חלונות אחרים: «Готово», «15 мин», «1 час», «3 часа», «Сутки». החלון זז לפי הכותרת ונסגר מעצמו כשאין מה להזכיר
- פס האיחור מתחת ליומן ומתחת ללוח נעלם יחד עם הדחייה
- התחלה, דדליין ותיאור אופציונליים
- חזרה בימי שבוע נבחרים, עד שלוש שנים
- סטטוסים משלך בין «К выполнению» ל-«Готово»
- תזכורת התחלה רק כשהמשימה ב-«К выполнению», כל 10 דקות
- דדליין שעבר מחליף את תזכורת ההתחלה ומתריע כל 10 דקות עד שהמשימה ב-«Готово»
- אפשר לדחות תזכורת לכל היותר ליממה
- ערכת כהה ובהירה, מגש, הפעלה אוטומטית
- קליטת משימות מקומית ב-`127.0.0.1` כל עוד האפליקציה רצה

</div>

</details>

<details>
<summary>हिन्दी</summary>

- महीने का कैलेंडर और स्टेटस बोर्ड, वही कार्ड; बोर्ड का कार्ड दूसरी कॉलम में खींचा जाता है
- टोस्ट और दूसरी विंडो के ऊपर एक विंडो: «Готово», «15 мин», «1 час», «3 часа», «Сутки»। विंडो शीर्षक से हिलती है और याद न बचे तो खुद बंद हो जाती है
- कैलेंडर और बोर्ड के नीचे की देरी वाली पट्टी टालने पर साथ हट जाती है
- शुरुआत, डेडलाइन और विवरण वैकल्पिक
- चुने हुए हफ़्ते के दिनों पर दोहराव, ज़्यादा से ज़्यादा तीन साल
- «К выполнению» और «Готово» के बीच अपने स्टेटस
- शुरुआत की याद सिर्फ «К выполнению» में, हर 10 मिनट
- बीती डेडलाइन शुरुआत की याद की जगह लेती है और «Готово» तक हर 10 मिनट याद दिलाती है
- याद को ज़्यादा से ज़्यादा एक दिन के लिए टाला जा सकता है
- डार्क और लाइट थीम, ट्रे, ऑटोस्टार्ट
- ऐप चलते समय `127.0.0.1` पर लोकल टास्क इनटेक

</details>

<details>
<summary>Oʻzbekcha</summary>

- Oylik kalendar va statuslar doskasi, bir xil kartochkalar; doskadagi kartochka boshqa ustunga sudraladi
- Tost va boshqa oynalar ustidagi oyna: «Готово», «15 мин», «1 час», «3 часа», «Сутки». Oyna sarlavhasidan suriladi va eslatadigan narsa qolmasa o‘zi yopiladi
- Kalendar va doska ostidagi muddati o‘tgan tasma surish bilan birga yo‘qoladi
- Boshlanish, dedlayn va tavsif ixtiyoriy
- Tanlangan hafta kunlarida takror, oralig‘i uch yildan oshmaydi
- «К выполнению» va «Готово» orasida o‘z statuslaringiz
- Boshlanish eslatmasi faqat «К выполнению» da, har 10 daqiqada
- O‘tgan dedlayn boshlanish eslatmasining o‘rnini egallaydi va vazifa «Готово» ga tushmaguncha har 10 daqiqada eslatadi
- Eslatmani ko‘pi bilan bir sutkaga surish mumkin
- Qorong‘i va yorug‘ mavzu, trey, avtostart
- Ilova ishlayotganda `127.0.0.1` da mahalliy vazifa qabul qilish

</details>

---

## Документация

<details open>
<summary>Русский</summary>

- Подробная инструкция: [docs/INSTRUCTIONS.md](docs/INSTRUCTIONS.md)

</details>

<details>
<summary>English</summary>

- Full guide: [docs/INSTRUCTIONS.en.md](docs/INSTRUCTIONS.en.md)

</details>

<details>
<summary>עברית</summary>

<div dir="rtl">

- מדריך מלא: [docs/INSTRUCTIONS.he.md](docs/INSTRUCTIONS.he.md)

</div>

</details>

<details>
<summary>हिन्दी</summary>

- पूरी गाइड: [docs/INSTRUCTIONS.hi.md](docs/INSTRUCTIONS.hi.md)

</details>

<details>
<summary>Oʻzbekcha</summary>

- To‘liq qo‘llanma: [docs/INSTRUCTIONS.uz.md](docs/INSTRUCTIONS.uz.md)

</details>

---

## Стек

<details open>
<summary>Русский</summary>

Rust · Tauri 2 · React · TypeScript · Vite

</details>

<details>
<summary>English</summary>

Rust · Tauri 2 · React · TypeScript · Vite

</details>

<details>
<summary>עברית</summary>

Rust · Tauri 2 · React · TypeScript · Vite

</details>

<details>
<summary>हिन्दी</summary>

Rust · Tauri 2 · React · TypeScript · Vite

</details>

<details>
<summary>Oʻzbekcha</summary>

Rust · Tauri 2 · React · TypeScript · Vite

</details>
