# Testing

## Parser unit tests

```bash
cd src-tauri && cargo test
```

## Tauri commands — via devtools console

Run the app:

```bash
npm run tauri dev
```

Open devtools in the Tauri window (right-click → Inspect).

In dev builds, `invoke` and `listen` are exposed on `window.__ipc` for easy console access.

Set up event listeners **first** (before connecting/starting the stream):

```js
const { invoke, listen } = window.__ipc;
await listen('serial://sample', e => console.log(e.payload));
await listen('serial://status', e => console.log(e.payload));
await listen('serial://raw',    e => console.log(e.payload));
```

Then invoke commands:

```js
await invoke('list_ports')
// → ["<port names>"]

await invoke('start_mock_stream', { rateHz: 10.0, shape: 'sincos' })
// → null  (null = Ok(()) from Rust, meaning success)
// serial://sample events will now appear in the console

await invoke('connection_status')
// → "connected"

await invoke('disconnect')
// → null
```

## Sending control characters

Control characters survive the JS→JSON→Rust→port pipeline. Pass them as hex escapes with an empty line ending:

```js
await invoke('send_line', { text: '\x03', lineEnding: '' })  // Ctrl-C
await invoke('send_line', { text: '\x04', lineEnding: '' })  // Ctrl-D
await invoke('send_line', { text: '\x1a', lineEnding: '' })  // Ctrl-Z
await invoke('send_line', { text: '\x1b', lineEnding: '' })  // ESC
```

`lineEnding: ''` prevents the default `\n` from being appended.

## Real serial port — with a loopback

With a USB-serial adapter, short TX→RX, then:

```js
const { invoke } = window.__ipc;
await invoke('connect', {
  path: '/dev/cu.usbserial-XXXX',
  baud: 115200,
  dataBits: 8,
  parity: 'none',
  stopBits: 1,
  flowControl: 'none'
})
await invoke('send_line', { text: '1.0,2.0,3.0', lineEnding: '\n' })
// serial://raw fires for tx; the rx echo triggers serial://sample
```

## Virtual serial loopback (no hardware)

`socat` creates a linked pair of virtual ports:

```bash
socat -d -d pty,raw,echo=0 pty,raw,echo=0
# prints two paths, e.g. /dev/ttys004 and /dev/ttys005
```

Connect the app to one port, then write to the other from a second terminal:

```bash
echo "temp:23.5,hum:45.2" > /dev/ttys005
```
