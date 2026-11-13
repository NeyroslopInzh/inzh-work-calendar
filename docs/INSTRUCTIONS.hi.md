# पूरी गाइड

## प्रोजेक्ट की बनावट

| फ़ोल्डर | काम |
|---------|------|
| `src/` | UI: React और TypeScript |
| `src-tauri/` | ऐप: Rust, Tauri, ट्रे, याद दिलाना, लोकल टास्क इनटेक |
| `docs/` | पाँच भाषाओं में दस्तावेज़ |
| `.github/workflows/` | `v*` टैग पर `.exe`, `.deb` और AppImage बिल्ड |

---

## Windows

### इंस्टॉलर

[Releases](https://github.com/NeyroslopInzh/inzh-work-calendar/releases) → `inzh.work.calendar_*_x64-setup.exe`।

`x64` सामान्य 64-बिट प्रोसेसर है, Intel या AMD। ARM पर इंस्टॉलर नहीं चलेगा।

### सोर्स / बिल्ड

Node.js और Rust चाहिए।

```powershell
npm install
npm run tauri build
```

इंस्टॉलर `src-tauri/target/release/bundle/nsis/` में बनेगा।

---

## Linux

### बिल्ड की ज़रूरतें

WebKitGTK 4.1 और Tauri की लाइब्रेरी:

```sh
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
npm install
npm run tauri build
```

`.deb` — `src-tauri/target/release/bundle/deb/` में, AppImage — `src-tauri/target/release/bundle/appimage/` में।

### तैयार फ़ाइलें

```bash
sudo apt install ./inzh.work.calendar_*_amd64.deb
```

```bash
chmod +x inzh.work.calendar_*_amd64.AppImage
./inzh.work.calendar_*_amd64.AppImage
```

`amd64` वही 64-बिट प्रोसेसर है जो `x64`। ARM पर बाइनरी नहीं चलेगी।

अगर AppImage कहे कि FUSE नहीं है, तो `libfuse2` लगाओ।

---

## विंडो और ट्रे

विंडो बंद करने पर कैलेंडर ट्रे में छिप जाता है। आइकन पर बायाँ क्लिक या «Открыть» विंडो वापस लाता है। बाहर निकलना — «Выход»। पहली बार चलाने पर ऑटोस्टार्ट चालू होता है; हेडर से उसे बंद किया जा सकता है। वहीं डार्क और लाइट थीम बदलती है।

---

## टास्क

| फ़ील्ड | मतलब |
|--------|--------|
| नाम | ज़रूरी |
| विवरण | वैकल्पिक |
| शुरुआत | वैकल्पिक। याद सिर्फ स्टेटस «К выполнению» में |
| डेडलाइन | वैकल्पिक। «Готово» तक याद दिलाती है |
| दोहराव | तारीखों की सीमा में हफ़्ते के दिन, ज़्यादा से ज़्यादा तीन साल |
| स्टेटस | «К выполнению», «В работе», «Готово», या अपना, पहले दोनों और «Готово» के बीच |

याद ज़्यादा से ज़्यादा हर 10 मिनट में एक बार आती है। डेडलाइन बीत जाए तो शुरुआत की याद चुप हो जाती है और सिर्फ डेडलाइन याद दिलाती है। टालना: 15 मिनट, 1 घंटा, 3 घंटे, एक दिन, या अपने मिनट — एक दिन से ज़्यादा नहीं।

कार्ड को दूसरे दिन या दूसरी कॉलम पर खींचा जा सकता है। «К выполнению» और «Готово» हटाए नहीं जा सकते। अपना स्टेटस मिटाने पर उसके टास्क «К выполнению» में लौट आते हैं।

दोहराव के दिन: 1 — सोमवार … 7 — रविवार। चुनी हुई सीमा में ऐसा कम से कम एक दिन आना चाहिए।

---

## डेटा

| OS | डायरेक्टरी |
|----|-------------|
| Windows | `%APPDATA%\com.inzh.workcalendar\` |
| Linux | `~/.local/share/com.inzh.workcalendar/` |

Linux पर `XDG_DATA_HOME` सेट हो तो डायरेक्टरी `$XDG_DATA_HOME/com.inzh.workcalendar/` है।

| फ़ाइल | अंदर क्या है |
|-------|---------------|
| `cards.json` | टास्क |
| `settings.json` | थीम, ऑटोस्टार्ट, कॉलम |
| `endpoint.json` | `base`, `port`, `token` |
| `inbox.jsonl` | ऐप बंद होने पर लिखे टास्क |
| `inbox.failed.jsonl` | जो पंक्तियाँ पढ़ी नहीं गईं |

---

## लोकल टास्क इनटेक

ऐप सिर्फ `127.0.0.1` सुनता है। पोर्ट `47321`–`47330` में से चुना जाता है और टोकन के साथ `endpoint.json` में लिखा जाता है।

हर अनुरोध को हेडर `Authorization: Bearer {token}` चाहिए।

| अनुरोध | काम |
|---------|------|
| `GET {base}/health` | `{ "ok": true }` |
| `GET {base}/tasks` | टास्क की सूची |
| `POST {base}/tasks` | बनाना |
| `PATCH {base}/tasks/{id}` | बदलना। `null` तारीख या दोहराव हटाता है |
| `DELETE {base}/tasks/{id}` | मिटाना |

बनाने का उदाहरण:

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

समय स्थानीय है: `YYYY-MM-DDTHH:MM`, `YYYY-MM-DD HH:MM`, `YYYY-MM-DDTHH:MM:SS` या RFC3339। `startAt`, `deadlineAt`, `description` और `recurrence` छोड़े जा सकते हैं। `status` का डिफ़ॉल्ट `todo` है।

ऐप बंद हो तो उसी डायरेक्टरी की `inbox.jsonl` में एक JSON पंक्ति जोड़ो। चालू होते ही कैलेंडर फ़ाइल ले लेगा। टूटी पंक्ति `inbox.failed.jsonl` में रह जाएगी, बाकी लग जाएँगी।

---

## आम मामले

### विंडो «बंद» हुई, पर प्रोसेस ज़िंदा है

यह ट्रे है। बाहर निकलना आइकन मेनू का «Выход» है।

### लोकल पता नहीं खुला

पोर्ट `47321`–`47330` भरे हैं। ऐप चलते समय `endpoint.json` नया नहीं होगा। टास्क फिर भी `inbox.jsonl` में जोड़ा जा सकता है।

### ARM पर नहीं चलता

रिलीज़ में सिर्फ `amd64` / `x64` है।

### AppImage नहीं चलता

अक्सर FUSE नहीं होता। `libfuse2` लगाओ और फ़ाइल फिर चलाओ।

---

## रिलीज़ बिल्ड

```bash
git tag v0.2.0
git push origin v0.2.0
```

GitHub Actions Windows (NSIS) और Linux (`.deb`, AppImage) बनाएगा। Workflow — `.github/workflows/release.yml`।

---

## लाइसेंस

[GNU GPL v3](../LICENSE) — copyleft। बदली हुई कॉपी जो किसी को दी जाए, उन्हीं शर्तों पर खोलनी होगी।
