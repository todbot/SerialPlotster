# SerialPlotster

A cross-platform desktop serial plotter built with [Tauri 2](https://tauri.app). Graphs line-oriented numeric data from a serial port as a real-time strip chart, with scrub/zoom that does not pause data collection, plus a console pane for bidirectional communication with the device.

![status: early development](https://img.shields.io/badge/status-early%20development-yellow)

<img src="./docs/screenshot1.png" width="700">

## Features

- **Live strip chart** — canvas-based renderer driven by `requestAnimationFrame`, no charting library
- **Non-blocking scrub/zoom** — drag or trackpad-swipe to pan through history while data keeps arriving; Ctrl+scroll or pinch to zoom; double-click to snap back to live
- **Auto or fixed Y axis** — auto-scales to the visible window, or lock to a manual min/max range
- **Multi-series** — colour-coded series; click legend swatches to toggle visibility
- **Flexible parser** — comma, tab, or space delimited; Arduino-style `label:value` pairs; Python tuple/list syntax; `# header` lines for series names; malformed lines silently skipped
- **Device restart detection** — a new `# header` line mid-stream clears the old series and starts fresh with the new names
- **Console pane** — scrolling RX/TX log with timestamps; send field with configurable line ending; one-click ^C / ^D buttons
- **Mock stream** — built-in synthetic sin/cos/noise source for development without hardware


## Supported data formats

Any line-oriented text where each line contains numbers:

```
1.23,4.56,7.89          # comma-separated
1.23  4.56  7.89        # space-separated
1.23\t4.56\t7.89        # tab-separated
temp:23.5, hum:45.2     # Arduino label:value pairs (sets series names)
(1.23, 4.56, 7.89)      # Python tuple
[1.23, 4.56, 7.89]      # Python list
# temperature humidity  # header line — sets series names
```

In general it tries to be very lenient in parsing. 

Header lines name the series; a new header mid-stream (e.g. after a device restart) 
clears the old series and replots under the new names.

## Chart controls

### Navigation

| Action | Result |
|---|---|
| Click + drag | Pan in time (enters scrub mode) |
| Horizontal trackpad swipe | Pan in time (enters scrub mode) |
| Ctrl + scroll wheel | Zoom in/out, pivoting around the mouse cursor |
| Ctrl + trackpad pinch | Zoom in/out |
| Double-click | Exit scrub mode, snap back to live |
| **back to live** button | Exit scrub mode, snap back to live |

Data continues accumulating in the buffer while scrubbing — you never miss samples.

### Y axis

| Control | Result |
|---|---|
| **Y: Auto** button | Auto-scales Y to the visible data in the current window |
| **Y: Fixed** button | Lock Y to a manual range; edit the min/max fields and press Enter |

### Other controls

| Control | Result |
|---|---|
| Time window selector | Sets the width of the visible time window (1 s – 5 m) |
| **⏸ Pause** / **▶ Resume** | Freeze/resume the live view (data still accumulates) |
| Legend swatches | Click to toggle individual series visibility |


## Stack

| Concern | Choice |
|---|---|
| Framework | Tauri 2 |
| Frontend | React 19 + TypeScript + Vite |
| Styling | Tailwind CSS v4 |
| Charting | Custom canvas rendering |
| Serial | `tauri-plugin-serialplugin` + `serialport` crate |

## Development

### Prerequisites

- [Rust](https://rustup.rs) (stable)
- [Node.js](https://nodejs.org) 18+
- Tauri CLI prerequisites for your platform — see [Tauri docs](https://tauri.app/start/prerequisites/)

### Run

```bash
npm install
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

### Clean

```bash
npm run clean        # removes dist/ and src-tauri/target/
```

### Tests

```bash
cd src-tauri && cargo test     # Rust parser unit tests
```

See [TESTING.md](TESTING.md) for how to exercise the Tauri commands and events from the browser devtools console.

## CI / releases

GitHub Actions workflow at `.github/workflows/build.yml` builds for macOS (arm64 + x86_64), Windows, and Linux on `workflow_dispatch`. Optional checkboxes enable:

- macOS code signing and notarization (Developer ID certificate)
- Windows signing via Azure Trusted Signing

See [codesigning.md](codesigning.md) for the required secrets and setup steps.

## Project structure

```
src/
  App.tsx
  components/
    Header.tsx           — port picker, baud, connect/disconnect, status pill
    TabNav.tsx           — Chart | Console tabs
    PlotCanvas.tsx       — canvas rendering, rAF loop, scrub/zoom
    PlotToolsOverlay.tsx — Y-axis mode, pause/resume, time-window selector
    Legend.tsx           — series names + colour swatches, toggle visibility
    ConsolePane.tsx      — wraps ConsoleLog + ConsoleInput
    ConsoleLog.tsx       — scrolling RX/TX list with timestamps
    ConsoleInput.tsx     — ^C/^D buttons, text field, line-ending picker, Send
  hooks/
    useSerialBackend.ts  — Tauri command wrappers + event subscriptions
    useRingBuffer.ts     — ring store as a React hook
  store/
    RingStore.ts         — per-series Float32Array ring buffer + Float64Array timestamps
    ConsoleStore.ts      — bounded list of RX/TX lines
  types/
    serial.ts            — TypeScript types matching Rust event payloads

src-tauri/src/
  lib.rs                — serial commands, read loop, line parser, mock stream, app menu
```

## Links

- Inspired by:
  - [atomic14 web-serial-plotter](https://github.com/atomic14/web-serial-plotter)
  - [Arduino Serial Plotter tool](https://docs.arduino.cc/software/ide-v2/tutorials/ide-v2-serial-plotter/)
  - [Mu serial plotter](https://codewith.mu/en/tutorials/1.0/plotter)
- Built with help from [Claude Code](https://claude.ai/code)

## License

GPL-3.0-only — see [LICENSE](LICENSE).
