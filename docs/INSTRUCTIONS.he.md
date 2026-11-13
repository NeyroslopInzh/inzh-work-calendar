# הוראות מלאות

## מבנה הפרויקט

| תיקייה | תפקיד |
|--------|--------|
| `src/` | ממשק: React ו-TypeScript |
| `src-tauri/` | אפליקציה: Rust, Tauri, מגש, תזכורות, קליטת משימות מקומית |
| `docs/` | תיעוד בחמש שפות |
| `.github/workflows/` | בנייה של `.exe`, `.deb` ו-AppImage לפי תגית `v*` |

---

## Windows

### מתקין

[Releases](https://github.com/NeyroslopInzh/inzh-work-calendar/releases) → `inzh.work.calendar_*_x64-setup.exe`.

`x64` הוא מעבד 64-ביט רגיל, Intel או AMD. על ARM המתקין לא ירוץ.

### קוד מקור / בנייה

נדרשים Node.js ו-Rust.

```powershell
npm install
npm run tauri build
```

המתקין יופיע ב-`src-tauri/target/release/bundle/nsis/`.

---

## Linux

### דרישות בנייה

WebKitGTK 4.1 וספריות Tauri:

```sh
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
npm install
npm run tauri build
```

ה-`.deb` נמצא ב-`src-tauri/target/release/bundle/deb/`, ה-AppImage ב-`src-tauri/target/release/bundle/appimage/`.

### קבצים מוכנים

```bash
sudo apt install ./inzh.work.calendar_*_amd64.deb
```

```bash
chmod +x inzh.work.calendar_*_amd64.AppImage
./inzh.work.calendar_*_amd64.AppImage
```

`amd64` הוא אותו מעבד 64-ביט כמו `x64`. הבינארי לא ירוץ על ARM.

אם AppImage מדווח שחסר FUSE, התקינו `libfuse2`.

---

## חלון ומגש

סגירת החלון מסתירה את היומן במגש. לחיצה שמאלית על הסמל, או «Открыть», מחזירה את החלון. יציאה — «Выход». בהפעלה הראשונה נדלקת הפעלה אוטומטית; אפשר לכבות אותה בכותרת. שם גם מחליפים בין ערכת כהה לבהירה.

---

## משימות

| שדה | משמעות |
|-----|--------|
| שם | חובה |
| תיאור | אופציונלי |
| התחלה | אופציונלי. תזכורת רק בסטטוס «К выполнению» |
| דדליין | אופציונלי. מתריע עד שהמשימה ב-«Готово» |
| חזרה | ימי שבוע בטווח תאריכים, עד שלוש שנים |
| סטטוס | «К выполнению», «В работе», «Готово», או סטטוס משלך ביניהם לבין «Готово» |

תזכורת מגיעה לכל היותר פעם ב-10 דקות. אחרי שהדדליין עבר, תזכורת ההתחלה שותקת ורק הדדליין מתריע. דחייה: 15 דקות, שעה, 3 שעות, יממה, או מספר דקות משלך, ולא יותר מיממה.

אפשר לגרור כרטיס ליום אחר או לעמודה אחרת. אי אפשר להסיר את «К выполнению» ואת «Готово». מחיקת סטטוס משלך מחזירה את המשימות שלו ל-«К выполнению».

ימי חזרה: 1 — יום שני … 7 — יום ראשון. בטווח שנבחר חייב ליפול לפחות יום אחד כזה.

---

## נתונים

| מערכת | תיקייה |
|-------|--------|
| Windows | `%APPDATA%\com.inzh.workcalendar\` |
| Linux | `~/.local/share/com.inzh.workcalendar/` |

אם ב-Linux מוגדר `XDG_DATA_HOME`, התיקייה היא `$XDG_DATA_HOME/com.inzh.workcalendar/`.

| קובץ | תוכן |
|------|------|
| `cards.json` | משימות |
| `settings.json` | ערכת נושא, הפעלה אוטומטית, עמודות |
| `endpoint.json` | `base`, `port`, `token` |
| `inbox.jsonl` | משימות שנכתבו כשהאפליקציה הייתה כבויה |
| `inbox.failed.jsonl` | שורות שלא ניתן היה לפענח |

---

## קליטת משימות מקומית

האפליקציה מאזינה רק ל-`127.0.0.1`. הפורט נבחר מתוך `47321`–`47330` ונכתב ל-`endpoint.json` יחד עם הטוקן.

כל בקשה דורשת את הכותרת `Authorization: Bearer {token}`.

| בקשה | פעולה |
|------|--------|
| `GET {base}/health` | `{ "ok": true }` |
| `GET {base}/tasks` | רשימת משימות |
| `POST {base}/tasks` | יצירה |
| `PATCH {base}/tasks/{id}` | עדכון. `null` מוחק תאריך או חזרה |
| `DELETE {base}/tasks/{id}` | מחיקה |

דוגמת יצירה:

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

הזמן מקומי: `YYYY-MM-DDTHH:MM`, `YYYY-MM-DD HH:MM`, `YYYY-MM-DDTHH:MM:SS` או RFC3339. אפשר לא לשלוח את `startAt`, `deadlineAt`, `description` ו-`recurrence`. ברירת המחדל של `status` היא `todo`.

אם האפליקציה כבויה, הוסיפו שורת JSON אחת ל-`inbox.jsonl` באותה תיקייה. בהפעלה היומן יאסוף את הקובץ. שורה שבורה נשארת ב-`inbox.failed.jsonl`, והשאר מוחלות.

---

## מקרים נפוצים

### החלון «נסגר» אבל התהליך חי

זה המגש. יציאה — «Выход» בתפריט הסמל.

### הכתובת המקומית לא נפתחה

הפורטים `47321`–`47330` תפוסים. כל עוד האפליקציה רצה, `endpoint.json` לא יתעדכן. עדיין אפשר להוסיף משימה ל-`inbox.jsonl`.

### לא עולה על ARM

בריליז יש רק `amd64` / `x64`.

### AppImage לא עולה

לרוב חסר FUSE. התקינו `libfuse2` והריצו שוב.

---

## בניית ריליז

```bash
git tag v0.2.0
git push origin v0.2.0
```

GitHub Actions יבנה Windows ‏(NSIS) ו-Linux ‏(`.deb`, AppImage). ה-workflow הוא `.github/workflows/release.yml`.

---

## רישיון

[GNU GPL v3](../LICENSE) — copyleft. עותק ששיניתם ומסרתם למישהו צריך להיפתח באותם תנאים.
