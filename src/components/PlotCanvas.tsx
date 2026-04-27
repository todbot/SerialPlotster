import { forwardRef, useEffect, useImperativeHandle, useRef } from 'react';
import type { RingStore } from '../store/RingStore';
import { seriesColor } from './Legend';

// ── types ─────────────────────────────────────────────────────────────────────

export interface PlotCanvasHandle {
  resetToLive(): void;
}

interface PlotCanvasProps {
  store: RingStore;
  seriesKeys: string[];
  visible: Set<string>;
  windowMs: number;
  paused: boolean;
  onScrubChange: (scrubbing: boolean) => void;
}

interface ViewState {
  mode: 'live' | 'scrub';
  startMs: number;
  endMs: number;
}

// ── drawing constants ─────────────────────────────────────────────────────────

const PAD = { top: 12, right: 16, bottom: 32, left: 56 };
const BG        = '#111827'; // gray-900
const GRID      = '#1f2937'; // gray-800
const AXIS_LINE = '#374151'; // gray-700
const AXIS_TEXT = '#6b7280'; // gray-500

// ── standalone draw function (no closures over React state) ───────────────────

type DrawProps = {
  seriesKeys: string[];
  visible: Set<string>;
  windowMs: number;
  paused: boolean;
};

function drawFrame(
  ctx: CanvasRenderingContext2D,
  canvas: HTMLCanvasElement,
  dpr: number,
  store: RingStore,
  view: ViewState,
  props: DrawProps,
): void {
  if (view.mode === 'live' && !props.paused) {
    view.endMs = Date.now();
    view.startMs = view.endMs - props.windowMs;
  }

  const W = canvas.width / dpr;
  const H = canvas.height / dpr;

  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.fillStyle = BG;
  ctx.fillRect(0, 0, W, H);

  const plotL = PAD.left;
  const plotT = PAD.top;
  const plotW = W - PAD.left - PAD.right;
  const plotH = H - PAD.top - PAD.bottom;
  if (plotW <= 0 || plotH <= 0) return;

  const { startMs, endMs } = view;
  const spanMs = endMs - startMs || 1;

  const xOf = (t: number) => plotL + ((t - startMs) / spanMs) * plotW;
  const yOf = (v: number, yMin: number, range: number) =>
    plotT + (1 - (v - yMin) / range) * plotH;

  // ── Y range ────────────────────────────────────────────────────────────────
  let yMin = Infinity, yMax = -Infinity;
  // Look back one extra window so a line entering from the left looks continuous
  const scanStart = startMs - spanMs;
  for (const key of props.seriesKeys) {
    if (!props.visible.has(key)) continue;
    store.forEachSample((t, ri) => {
      if (t < scanStart) return;
      if (t > endMs) return false;
      const v = store.getValue(key, ri);
      if (!isNaN(v) && t >= startMs) {
        if (v < yMin) yMin = v;
        if (v > yMax) yMax = v;
      }
    });
  }
  if (!isFinite(yMin)) { yMin = -1; yMax = 1; }
  if (yMin === yMax) { yMin -= 1; yMax += 1; }
  const yPad = (yMax - yMin) * 0.06;
  yMin -= yPad; yMax += yPad;
  const yRange = yMax - yMin;

  // ── Y grid + labels ────────────────────────────────────────────────────────
  ctx.font = '10px monospace';
  ctx.textBaseline = 'middle';
  ctx.textAlign = 'right';
  for (const y of niceTicks(yMin, yMax, 5)) {
    const cy = yOf(y, yMin, yRange);
    ctx.strokeStyle = GRID;
    ctx.lineWidth = 1;
    ctx.beginPath(); ctx.moveTo(plotL, cy); ctx.lineTo(plotL + plotW, cy); ctx.stroke();
    ctx.fillStyle = AXIS_TEXT;
    ctx.fillText(fmtVal(y), plotL - 4, cy);
  }

  // ── X grid + labels ────────────────────────────────────────────────────────
  ctx.textAlign = 'center';
  ctx.textBaseline = 'top';
  for (const t of niceTimeTicks(startMs, endMs, Math.max(2, Math.floor(plotW / 70)))) {
    const cx = xOf(t);
    if (cx < plotL || cx > plotL + plotW) continue;
    ctx.strokeStyle = GRID;
    ctx.lineWidth = 1;
    ctx.beginPath(); ctx.moveTo(cx, plotT); ctx.lineTo(cx, plotT + plotH); ctx.stroke();
    ctx.fillStyle = AXIS_TEXT;
    ctx.fillText(fmtRelTime(t, endMs), cx, plotT + plotH + 4);
  }

  // ── plot border ────────────────────────────────────────────────────────────
  ctx.strokeStyle = AXIS_LINE;
  ctx.lineWidth = 1;
  ctx.strokeRect(plotL, plotT, plotW, plotH);

  // ── series lines (clipped) ─────────────────────────────────────────────────
  ctx.save();
  ctx.beginPath();
  ctx.rect(plotL, plotT, plotW, plotH);
  ctx.clip();

  ctx.lineWidth = 1.5;
  ctx.lineJoin = 'round';

  for (let si = 0; si < props.seriesKeys.length; si++) {
    const key = props.seriesKeys[si];
    if (!props.visible.has(key)) continue;
    ctx.strokeStyle = seriesColor(si);
    ctx.beginPath();
    let pen = false;

    store.forEachSample((t, ri) => {
      if (t > endMs + 200) return false; // stop well past right edge
      const v = store.getValue(key, ri);
      if (isNaN(v)) { pen = false; return; }
      if (t < scanStart) { pen = false; return; } // way too old, skip
      const x = xOf(t);
      const y = yOf(v, yMin, yRange);
      if (pen) ctx.lineTo(x, y); else { ctx.moveTo(x, y); pen = true; }
    });

    ctx.stroke();
  }

  ctx.restore();
}

// ── component ─────────────────────────────────────────────────────────────────

export const PlotCanvas = forwardRef<PlotCanvasHandle, PlotCanvasProps>(
  function PlotCanvas({ store, seriesKeys, visible, windowMs, paused, onScrubChange }, ref) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const viewRef = useRef<ViewState>({ mode: 'live', startMs: 0, endMs: 0 });
    const dragRef = useRef({ active: false, x0: 0, startMs0: 0, endMs0: 0 });
    // Keep a stable ref to current props so the rAF closure never goes stale.
    const propsRef = useRef<DrawProps>({ seriesKeys, visible, windowMs, paused });
    propsRef.current = { seriesKeys, visible, windowMs, paused };
    const scrubCbRef = useRef(onScrubChange);
    scrubCbRef.current = onScrubChange;

    useImperativeHandle(ref, () => ({
      resetToLive() {
        viewRef.current.mode = 'live';
      },
    }));

    // rAF loop + ResizeObserver — runs once, reads everything through refs
    useEffect(() => {
      const canvas = canvasRef.current!;
      const ctx = canvas.getContext('2d')!;
      let rafId: number;
      let dpr = window.devicePixelRatio || 1;

      const ro = new ResizeObserver(() => {
        dpr = window.devicePixelRatio || 1;
        const r = canvas.getBoundingClientRect();
        canvas.width = Math.round(r.width * dpr);
        canvas.height = Math.round(r.height * dpr);
      });
      ro.observe(canvas);

      function tick() {
        drawFrame(ctx, canvas, dpr, store, viewRef.current, propsRef.current);
        rafId = requestAnimationFrame(tick);
      }
      rafId = requestAnimationFrame(tick);
      return () => { cancelAnimationFrame(rafId); ro.disconnect(); };
    }, [store]); // store is a stable ref

    // ── mouse / wheel handlers ───────────────────────────────────────────────

    function enterScrub() {
      if (viewRef.current.mode === 'live') {
        viewRef.current.mode = 'scrub';
        scrubCbRef.current(true);
      }
    }

    function onMouseDown(e: React.MouseEvent) {
      if (e.button !== 0) return;
      const v = viewRef.current;
      dragRef.current = { active: true, x0: e.clientX, startMs0: v.startMs, endMs0: v.endMs };
      enterScrub();
    }

    function onMouseMove(e: React.MouseEvent) {
      const d = dragRef.current;
      if (!d.active) return;
      const rect = canvasRef.current!.getBoundingClientRect();
      const plotW = rect.width - PAD.left - PAD.right;
      const spanMs = d.endMs0 - d.startMs0;
      const dMs = ((e.clientX - d.x0) / plotW) * spanMs;
      viewRef.current.startMs = d.startMs0 - dMs;
      viewRef.current.endMs   = d.endMs0   - dMs;
    }

    function onMouseUp() { dragRef.current.active = false; }

    function onDoubleClick() {
      viewRef.current.mode = 'live';
      scrubCbRef.current(false);
    }

    function onWheel(e: React.WheelEvent) {
      const v = viewRef.current;
      const rect = canvasRef.current!.getBoundingClientRect();
      const plotW = rect.width - PAD.left - PAD.right;

      if (e.ctrlKey) {
        // Ctrl+scroll or pinch-to-zoom
        e.preventDefault();
        const frac = (e.clientX - rect.left - PAD.left) / plotW;
        const pivot = v.startMs + frac * (v.endMs - v.startMs);
        const factor = e.deltaY > 0 ? 1.25 : 0.8;
        v.startMs = pivot - (pivot - v.startMs) * factor;
        v.endMs   = pivot + (v.endMs - pivot) * factor;
        enterScrub();
      } else if (Math.abs(e.deltaX) > Math.abs(e.deltaY)) {
        // Horizontal trackpad swipe → pan in time
        e.preventDefault();
        const spanMs = v.endMs - v.startMs;
        const dMs = (e.deltaX / plotW) * spanMs;
        v.startMs -= dMs;
        v.endMs   -= dMs;
        enterScrub();
      }
    }

    return (
      <canvas
        ref={canvasRef}
        className="block w-full h-full flex-1"
        style={{ cursor: dragRef.current.active ? 'grabbing' : 'crosshair' }}
        onMouseDown={onMouseDown}
        onMouseMove={onMouseMove}
        onMouseUp={onMouseUp}
        onMouseLeave={onMouseUp}
        onDoubleClick={onDoubleClick}
        onWheel={onWheel}
      />
    );
  },
);

// ── helpers ───────────────────────────────────────────────────────────────────

function niceTicks(min: number, max: number, n: number): number[] {
  const range = max - min;
  if (range === 0) return [min];
  const rawStep = range / n;
  const mag = Math.pow(10, Math.floor(Math.log10(rawStep)));
  const step = [1, 2, 2.5, 5, 10].map((x) => x * mag).find((x) => x >= rawStep) ?? 10 * mag;
  const start = Math.ceil(min / step) * step;
  const ticks: number[] = [];
  for (let v = start; v <= max + step * 0.001; v += step) {
    ticks.push(parseFloat(v.toPrecision(9)));
  }
  return ticks;
}

function niceTimeTicks(startMs: number, endMs: number, n: number): number[] {
  const spanS = (endMs - startMs) / 1000;
  const steps = [0.1, 0.2, 0.5, 1, 2, 5, 10, 15, 30, 60, 120, 300, 600];
  const step = steps.find((s) => spanS / s <= n) ?? 600;
  const startS = Math.ceil(startMs / 1000 / step) * step;
  const ticks: number[] = [];
  for (let t = startS; t <= endMs / 1000 + step * 0.001; t += step) {
    ticks.push(t * 1000);
  }
  return ticks;
}

function fmtVal(v: number): string {
  const a = Math.abs(v);
  if (a === 0) return '0';
  if (a >= 10000) return `${(v / 1000).toFixed(1)}k`;
  if (a >= 1000)  return `${(v / 1000).toFixed(2)}k`;
  if (a >= 100)   return v.toFixed(0);
  if (a >= 10)    return v.toFixed(1);
  if (a >= 1)     return v.toFixed(2);
  return v.toPrecision(2);
}

function fmtRelTime(t_ms: number, endMs: number): string {
  const d = (t_ms - endMs) / 1000;
  if (Math.abs(d) < 0.5) return '0';
  if (Math.abs(d) < 60) return `${d.toFixed(0)}s`;
  return `${(d / 60).toFixed(0)}m`;
}
