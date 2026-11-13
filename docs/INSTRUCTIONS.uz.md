# To‘liq qo‘llanma

## Loyiha tuzilishi

| Papka | Vazifasi |
|-------|----------|
| `src/` | Interfeys: React va TypeScript |
| `src-tauri/` | Ilova: Rust, Tauri, trey, eslatmalar, mahalliy vazifa qabul qilish |
| `docs/` | Besh tildagi hujjatlar |
| `.github/workflows/` | `v*` tegi bo‘yicha `.exe`, `.deb` va AppImage yig‘ish |

---

## Windows

### O‘rnatuvchi

[Releases](https://github.com/NeyroslopInzh/inzh-work-calendar/releases) → `inzh.work.calendar_*_x64-setup.exe`.

`x64` — oddiy 64-bit protsessor, Intel yoki AMD. ARM da o‘rnatuvchi ishlamaydi.

### Manba / yig‘ish

Node.js va Rust kerak.

```powershell
npm install
npm run tauri build
```

O‘rnatuvchi `src-tauri/target/release/bundle/nsis/` da paydo bo‘ladi.

---

## Linux

### Yig‘ish talablari

WebKitGTK 4.1 va Tauri kutubxonalari:

```sh
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
npm install
npm run tauri build
```

`.deb` — `src-tauri/target/release/bundle/deb/` da, AppImage — `src-tauri/target/release/bundle/appimage/` da.

### Tayyor fayllar

```bash
sudo apt install ./inzh.work.calendar_*_amd64.deb
```

```bash
chmod +x inzh.work.calendar_*_amd64.AppImage
./inzh.work.calendar_*_amd64.AppImage
```

`amd64` — `x64` bilan bir xil 64-bit protsessor. ARM da binar ishlamaydi.

AppImage FUSE yetishmayapti desa, `libfuse2` o‘rnating.

---

## Oyna va trey

Oynani yopish kalendarni treyga yashiradi. Belgi ustiga chap sichqoncha yoki «Открыть» oynani qaytaradi. Chiqish — «Выход». Birinchi ishga tushirishda avtostart yoqiladi; uni sarlavhadan o‘chirish mumkin. O‘sha yerda qorong‘i va yorug‘ mavzu almashadi.

---

## Vazifalar

| Maydon | Ma’nosi |
|--------|---------|
| Nom | Majburiy |
| Tavsif | Ixtiyoriy |
| Boshlanish | Ixtiyoriy. Eslatma faqat «К выполнению» statusida |
| Dedlayn | Ixtiyoriy. Vazifa «Готово» ga tushmaguncha eslatadi |
| Takror | Sana oralig‘idagi hafta kunlari, ko‘pi bilan uch yil |
| Status | «К выполнению», «В работе», «Готово» yoki o‘zingizniki, birinchi ikkisi va «Готово» orasida |

Eslatma 10 daqiqada bir martadan ko‘p kelmaydi. Dedlayn o‘tib ketgach, boshlanish eslatmasi jim bo‘ladi va faqat dedlayn eslatadi. Surish: 15 daqiqa, 1 soat, 3 soat, sutka yoki o‘z daqiqalaringiz, lekin sutkadan oshmasin.

Kartochkani boshqa kunga yoki boshqa ustunga sudrash mumkin. «К выполнению» va «Готово» ni olib tashlab bo‘lmaydi. O‘z statusingizni o‘chirsangiz, uning vazifalari «К выполнению» ga qaytadi.

Takror kunlari: 1 — dushanba … 7 — yakshanba. Tanlangan oralig‘da shunday kamida bitta kun bo‘lishi kerak.

---

## Ma’lumotlar

| OT | Katalog |
|----|---------|
| Windows | `%APPDATA%\com.inzh.workcalendar\` |
| Linux | `~/.local/share/com.inzh.workcalendar/` |

Linux da `XDG_DATA_HOME` berilgan bo‘lsa, katalog — `$XDG_DATA_HOME/com.inzh.workcalendar/`.

| Fayl | Ichida |
|------|--------|
| `cards.json` | Vazifalar |
| `settings.json` | Mavzu, avtostart, ustunlar |
| `endpoint.json` | `base`, `port`, `token` |
| `inbox.jsonl` | Ilova o‘chiq paytda yozilgan vazifalar |
| `inbox.failed.jsonl` | O‘qib bo‘lmagan qatorlar |

---

## Mahalliy vazifa qabul qilish

Ilova faqat `127.0.0.1` ni tinglaydi. Port `47321`–`47330` oralig‘idan olinadi va token bilan birga `endpoint.json` ga yoziladi.

Har bir so‘rovga `Authorization: Bearer {token}` sarlavhasi kerak.

| So‘rov | Ish |
|--------|-----|
| `GET {base}/health` | `{ "ok": true }` |
| `GET {base}/tasks` | Vazifalar ro‘yxati |
| `POST {base}/tasks` | Yaratish |
| `PATCH {base}/tasks/{id}` | O‘zgartirish. `null` sana yoki takrorni olib tashlaydi |
| `DELETE {base}/tasks/{id}` | O‘chirish |

Yaratish namunasi:

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

Vaqt mahalliy: `YYYY-MM-DDTHH:MM`, `YYYY-MM-DD HH:MM`, `YYYY-MM-DDTHH:MM:SS` yoki RFC3339. `startAt`, `deadlineAt`, `description` va `recurrence` yuborilmasa ham bo‘ladi. `status` sukut bo‘yicha `todo`.

Ilova o‘chiq bo‘lsa, shu katalogdagi `inbox.jsonl` ga bitta JSON qator qo‘shing. Ishga tushganda kalendar faylni oladi. Buzuq qator `inbox.failed.jsonl` da qoladi, qolganlari qo‘llanadi.

---

## Odatiy holatlar

### Oyna «yopildi», lekin jarayon tirik

Bu trey. Chiqish — belgi menyusidagi «Выход».

### Mahalliy manzil ochilmadi

`47321`–`47330` portlari band. Ilova ishlayotganda `endpoint.json` yangilanmaydi. Vazifani baribir `inbox.jsonl` ga qo‘shish mumkin.

### ARM da ishga tushmaydi

Relizda faqat `amd64` / `x64` bor.

### AppImage ishga tushmaydi

Ko‘pincha FUSE yetishmaydi. `libfuse2` o‘rnating va faylni qayta ishga tushiring.

---

## Reliz yig‘ish

```bash
git tag v0.2.2
git push origin v0.2.2
```

GitHub Actions Windows (NSIS) va Linux (`.deb`, AppImage) yig‘adi. Workflow — `.github/workflows/release.yml`.

---

## Litsenziya

[GNU GPL v3](../LICENSE) — copyleft. O‘zgartirib kimga berilgan nusxa shu shartlarda ochiq bo‘lishi kerak.
