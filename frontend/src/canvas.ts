import { initWasm } from './wasm-loader';
import { DrawLineWrapper } from '../wrapper/pkg/arch_wasm_wrapper';

const MARGIN = 24;
const BACKGROUND = '#ffffff';
const INK = '#000000';

let ctx: CanvasRenderingContext2D | null = null;
let canvasEl: HTMLCanvasElement | null = null;
let boardW = 0;
let boardH = 0;
let wasmReady: Promise<void> | null = null;

async function ensureWasm(): Promise<void> {
  if (!wasmReady) {
    wasmReady = initWasm();
  }
  await wasmReady;
}

/** Current board size in pixels. Backing store === CSS size (1:1),
 *  so canvas-pixel coords = client coords minus bounding-rect offset. */
export function boardSize(): { w: number; h: number } {
  return { w: boardW, h: boardH };
}

function computeSize(): { w: number; h: number } {
  return {
    w: Math.max(1, window.innerWidth - MARGIN * 2),
    h: Math.max(1, window.innerHeight - MARGIN * 2),
  };
}

function paintBackground(): void {
  if (!ctx) return;
  ctx.fillStyle = BACKGROUND;
  ctx.fillRect(0, 0, boardW, boardH);
  ctx.fillStyle = INK;
}

function applySize(): void {
  if (!canvasEl) return;
  const size = computeSize();
  boardW = size.w;
  boardH = size.h;
  canvasEl.width = boardW;
  canvasEl.height = boardH;
  canvasEl.style.width = `${boardW}px`;
  canvasEl.style.height = `${boardH}px`;
  paintBackground();
}

/** Create the board sized to the viewport, paint its background, return it.
 *  The board refits on window resize (resizing clears the surface).
 *  Pointer listeners attach to the returned canvas (owner: pointer logic). */
export async function initBoard(): Promise<HTMLCanvasElement> {
  await ensureWasm();

  const canvas = document.createElement('canvas');
  canvas.id = 'board';
  canvasEl = canvas;

  const context = canvas.getContext('2d');
  if (!context) {
    throw new Error('Canvas 2D context unavailable');
  }
  ctx = context;
  applySize();
  window.addEventListener('resize', applySize);
  return canvas;
}

/** Draw one rubber-band frame: anchor (x0,y0) to cursor (x1,y1).
 *  Stateless per call — safe to invoke on every pointermove. */
export function previewLine(x0: number, y0: number, x1: number, y1: number): void {
  if (!ctx) return;

  const line = new DrawLineWrapper(
    Math.round(x0),
    Math.round(y0),
    Math.round(x1),
    Math.round(y1),
  );
  const coords = line.get_line_pxl_coordinates();

  ctx.clearRect(0, 0, boardW, boardH);
  paintBackground();
  for (let i = 0; i + 1 < coords.length; i += 2) {
    ctx.fillRect(coords[i], coords[i + 1], 1, 1);
  }

  line.free();
}

/** Anti-aliased variant: same frame contract, but each pixel carries an
 *  alpha byte (triples [x, y, alpha255]) blended over the background.
 *  Swap your pointer handlers to this to see the smooth line. */
export function previewLineAA(x0: number, y0: number, x1: number, y1: number): void {
  if (!ctx) return;

  const line = new DrawLineWrapper(
    Math.round(x0),
    Math.round(y0),
    Math.round(x1),
    Math.round(y1),
  );
  const coords = line.get_line_aa_pxl_coordinates();

  ctx.clearRect(0, 0, boardW, boardH);
  paintBackground();
  for (let i = 0; i + 2 < coords.length; i += 3) {
    ctx.globalAlpha = coords[i + 2] / 255;
    ctx.fillRect(coords[i], coords[i + 1], 1, 1);
  }
  ctx.globalAlpha = 1;

  line.free();
}
