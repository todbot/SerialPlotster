import type { RawEvent } from '../types/serial';

const MAX_LINES = 1000;

export class ConsoleStore {
  private _lines: RawEvent[] = [];
  private _listeners = new Set<() => void>();

  get lines(): readonly RawEvent[] { return this._lines; }

  add(entry: RawEvent): void {
    this._lines.push(entry);
    if (this._lines.length > MAX_LINES) this._lines.shift();
    for (const cb of this._listeners) cb();
  }

  subscribe(cb: () => void): () => void {
    this._listeners.add(cb);
    return () => this._listeners.delete(cb);
  }

  clear(): void {
    this._lines = [];
    for (const cb of this._listeners) cb();
  }
}
