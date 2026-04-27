const DEFAULT_CAPACITY = 100_000;

export class RingStore {
  private _t: Float64Array;
  private _data = new Map<string, Float32Array>();
  private _keys: string[] = [];
  private _head = 0; // next write slot
  private _size = 0; // number of valid samples (≤ capacity)
  readonly capacity: number;

  constructor(capacity = DEFAULT_CAPACITY) {
    this.capacity = capacity;
    this._t = new Float64Array(capacity);
  }

  get length(): number { return this._size; }
  get seriesKeys(): readonly string[] { return this._keys; }

  /** Add a sample. Returns true if a new series was created. */
  addSample(t_ms: number, values: number[], labels?: string[]): boolean {
    const keys =
      labels && labels.length === values.length
        ? labels
        : values.map((_, i) => String(i));

    let newSeries = false;
    for (const k of keys) {
      if (!this._data.has(k)) {
        const arr = new Float32Array(this.capacity);
        arr.fill(NaN);
        this._data.set(k, arr);
        this._keys.push(k);
        newSeries = true;
      }
    }

    this._t[this._head] = t_ms;
    for (const [k, arr] of this._data) {
      const idx = keys.indexOf(k);
      arr[this._head] = idx >= 0 ? values[idx] : NaN;
    }

    this._advance();
    return newSeries;
  }

  /** Insert a NaN break on all series (visual disconnect gap). */
  addGap(): void {
    if (this._size === 0) return;
    const prevT = this._t[(this._head - 1 + this.capacity) % this.capacity];
    this._t[this._head] = prevT;
    for (const arr of this._data.values()) arr[this._head] = NaN;
    this._advance();
  }

  /**
   * Iterate samples in chronological order.
   * Return `false` from the callback to stop early.
   */
  forEachSample(cb: (t: number, ri: number) => boolean | void): void {
    const start = this._size < this.capacity ? 0 : this._head;
    for (let i = 0; i < this._size; i++) {
      const ri = (start + i) % this.capacity;
      if (cb(this._t[ri], ri) === false) break;
    }
  }

  getValue(key: string, ri: number): number {
    return this._data.get(key)?.[ri] ?? NaN;
  }

  clear(): void {
    this._head = 0;
    this._size = 0;
    this._data.clear();
    this._keys = [];
  }

  private _advance(): void {
    this._head = (this._head + 1) % this.capacity;
    if (this._size < this.capacity) this._size++;
  }
}
