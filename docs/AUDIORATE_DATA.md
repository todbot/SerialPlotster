# Changes since v0.2 (audiorate_fixes branch)

## High-rate data support

### Problem
At audio-rate sample rates (10 kHz+), USB CDC on macOS delivers data in ~52 ms bursts rather than one event per line. The original architecture emitted one `serial://sample` IPC event per parsed line, causing massive IPC overhead and chart stutter.

### Batch IPC (`serial://sample-batch`)
- Read loop now accumulates all samples from a single `read()` call into a `Vec<SampleEvent>` and emits one `serial://sample-batch` event instead of N `serial://sample` events.
- Reduces IPC crossings from O(samples/sec) to O(read-calls/sec) — ~50× reduction at 10 kHz.
- Frontend `useSerialBackend.ts` subscribes to `serial://sample-batch` and calls `store.addSamples(batch)`.

### Monotonic timestamps
- Each parsed line now receives `t = max(real_now, last_t + 1)` — strictly increasing even when multiple lines arrive in one burst.
- Spreads burst data evenly over time so the chart doesn't compress all burst samples to a single x-pixel.

### Console raw event throttle
- `serial://raw` events capped at 100/sec via a token-bucket `RawThrottle`.
- The console pane can't usefully display more than ~100 lines/sec anyway; throttling prevents IPC saturation at high rates.

### Chart rendering optimisations
Three-layer improvement to keep the chart smooth at 500 k samples in the ring buffer:

1. **Binary-search entry point** (`RingStore.forEachSampleFrom`): skips O(N) samples older than the visible window before iterating; timestamps are monotonically increasing so binary search is valid.
2. **Merged multi-series pass**: replaced 3 separate `forEachSampleFrom` calls (one per visible series) with a single pass that accumulates a `Path2D` per series, cutting the iteration cost by ~3×.
3. **Bucket scan range**: pixel-bucketed min-max downsampling now scans only `[startMs, endMs+200]` (matching the visible window) rather than an extended lookback.

## Correctness fixes

### Disconnect race
Two races fixed that caused stale data to appear after disconnect:

- **Mock stream**: `serial://status { connected }` and the `ActiveConnection` entry are now set *before* the worker thread is spawned, matching the fix already applied to `connect()`.
- **Batch emit after stop**: read loop now checks `stop_flag` (SeqCst ordering) before emitting a batch; any in-flight samples are discarded rather than delivered after the status event.

### Auto-pause on disconnect
- `App.tsx` now auto-pauses the chart on `disconnected`/`error` status and auto-resumes on `connected`.
- Prevents the live view from advancing wall-clock time while disconnected, which previously made old data appear to scroll as if live for minutes after disconnect.

### Blank chart after tab switch
- When the chart is paused and mounted fresh (e.g. switching console → chart tab), `viewRef.endMs` was 0 and `drawFrame` never initialised the viewport.
- Fixed with an `else if (view.endMs === 0)` branch that seeds `endMs = Date.now()` on first paint.

## Mock stream improvements

### Available in production builds
- Removed all `#[cfg(debug_assertions)]` guards from `start_mock_stream` command registration, Tauri menu item creation, and menu builder.
- Mock controls in `Header.tsx` no longer gated behind `import.meta.env.DEV`.
- The Mock submenu and `▶ Mock` button are now available in release builds.

### Noise waveform fix
- Old formula: `((t * 1234.567).sin() * 999.0).fract()` — output range `[0, 1)`, not centred, amplitude shrank to near-zero at high `t` values (10 kHz rate).
- New formula: `(t * 7.31).sin() * 0.7 + (t * 3.17).cos() * 0.3` — range `[−1, 1]`, rate-independent, incommensurate frequencies give aperiodic appearance.

## Code structure

### Parser extracted to `parser.rs`
- `LineResult` enum and `parse_line()` function moved from `lib.rs` to `src-tauri/src/parser.rs`.
- `lib.rs` re-exports via `mod parser; use parser::{parse_line, LineResult};`.
- `tests.rs` unchanged — `use super::*` picks up the re-export.

### Native OS menu
- Added OS-native menu bar with keyboard shortcuts (Cmd/Ctrl+1/2 for tabs, Cmd+R for connect/disconnect, Space for pause, Escape for back-to-live, Cmd+L to clear console).
- macOS: App + Edit submenus; Windows/Linux: File + Edit submenus.
- Menu actions emit `menu://action` Tauri events; `App.tsx` handler mirrors the keyboard shortcut logic.

### First partial-line discard
- Read loop now discards the first line fragment after connecting (the tail of a line already in flight when the port opened).
- Prevents a garbage partial value from corrupting Y-axis auto-scale on connect.
