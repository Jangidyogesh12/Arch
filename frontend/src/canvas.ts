import { initWasm } from './wasm-loader';
import { PaintScene } from '../wrapper/pkg/arch_wasm_wrapper';

const MARGIN = 24;
const DISTANCE = 5;

let ctx: CanvasRenderingContext2D | null = null;
let canvasEl: HTMLCanvasElement | null = null;
let scene: PaintScene | null = null;
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

function applySize(): void {
  if (!canvasEl) return;
  const size = computeSize();
  boardW = size.w;
  boardH = size.h;
  canvasEl.width = boardW;
  canvasEl.height = boardH;
  canvasEl.style.width = `${boardW}px`;
  canvasEl.style.height = `${boardH}px`;
  scene = new PaintScene(boardW, boardH, DISTANCE);
}

/** Call render() in Rust and blit the RGBA pixel buffer to the canvas. */
export function renderScene(): void {
  if (!ctx || !scene) return;
  const pixels = scene.render();
  const imageData = new ImageData(
    new Uint8ClampedArray(pixels),
    boardW,
    boardH,
  );
  ctx.putImageData(imageData, 0, 0);
}

/** Create the board sized to the viewport, return it.
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

export function getScene(): PaintScene | null {
  return scene;
}
