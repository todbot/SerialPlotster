# SerialPlotster — TODO

## Phase 0 — Scaffold ✅

- [x] `npm create tauri-app` — React + TypeScript + Vite
- [x] Fix identifier (`com.todbot.serial-plotster`), productName, window title
- [x] `.gitignore` present (+ `src-tauri/target/` added)
- [x] `README.md` stub present
- [x] Add Tailwind CSS v4 via `@tailwindcss/vite`
- [x] Add serial plugin: `cargo tauri add serialplugin`
- [x] Verify `src-tauri/capabilities/default.json` includes `"serialplugin:default"`
- [x] Add `#[tauri::command]` IPC ping ("Hello from Rust") and call it from the frontend on load

**Acceptance:** ✅ `npm run tauri dev` opens a window, IPC ping returns successfully, serial plugin compiles.

---

## Phase 1 — Rust side ✅

- [x] `list_ports() -> Vec<String>`
- [x] `connect(path, baud, data_bits, parity, stop_bits, flow_control) -> Result<()>`
- [x] `disconnect() -> Result<()>`
- [x] `send_line(text, line_ending) -> Result<()>`
- [x] `connection_status() -> String`
- [x] Read loop: `std::thread` + `Arc<AtomicBool>` stop flag, 50 ms timeout poll, partial-line buffer
- [x] Emit `serial://sample`, `serial://raw`, `serial://status` events
- [x] Line parser (`,` → `\t` → ` ` precedence, `label:value`, `# header`)
- [x] `start_mock_stream(rate_hz, shape)` debug command (sin / cos / sincos / all)
- [x] 20 parser unit tests passing; `cargo clippy -D warnings` clean

**Acceptance:** ✅ `list_ports` works; connecting to a loopback fires `serial://sample`; yanking USB fires `serial://status disconnected` within ~1s; `send_line` reaches device.

---

## Phase 2 — Frontend chart ⬜

- [ ] `types/serial.ts` — types matching Rust event payloads
- [ ] `store/RingStore.ts` — per-series Float32Array ring buffer + Float64Array timestamps; NaN discontinuity markers
- [ ] `store/ConsoleStore.ts` — bounded RX/TX line list
- [ ] `hooks/useSerialBackend.ts` — Tauri command wrappers + event subscriptions
- [ ] `hooks/useRingBuffer.ts` — ring store as React hook
- [ ] `components/Header.tsx` — port picker, baud, connect/disconnect, status pill
- [ ] `components/TabNav.tsx` — Chart | Console tabs
- [ ] `components/PlotCanvas.tsx` — rAF render loop, live/scrub modes, pan, ctrl+wheel zoom, double-click reset, NaN gap rendering, disconnect annotation
- [ ] `components/PlotToolsOverlay.tsx` — pause/resume, reset zoom, time-window selector
- [ ] `components/Legend.tsx` — series names + color swatches, toggle visibility
- [ ] `components/ConsolePane.tsx` — wraps ConsoleLog + ConsoleInput
- [ ] `components/ConsoleLog.tsx` — scrolling RX/TX list with timestamps
- [ ] `components/ConsoleInput.tsx` — text field + line-ending dropdown + send button

**Acceptance:** sin/cos at 100 Hz draws smoothly; disconnect shows gap + status flip; scrub freezes view while data keeps arriving; send reaches device; malformed lines appear in console only.

---

## Settled decisions

1. Default line ending for sent commands: **`\n`**
2. Default chart time window: **30 s**
3. Header line format: **both** `# label1 label2` and `label:value`
4. Y-axis: **auto-scale to visible data**
5. On reconnect: **continue buffer with NaN gap** (do not clear)

---

## Housekeeping

- `src-tauri/target/` in `.gitignore`
- `npm run clean` — removes `dist/` and `src-tauri/target/`
- `npm run tauribuild` — alias for `tauri build`

---

## Out of scope for v1

CSV/JSON/PNG export · multiple connections · binary protocols · FFT/signal processing · session save/load · iOS/Android · custom baud rate text input · statistics panel
