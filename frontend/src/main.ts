import './style.css'
import { initBoard, previewLine } from './canvas';

const app = document.querySelector<HTMLDivElement>('#app')!;
app.innerHTML = '';

const board = await initBoard();
app.appendChild(board);

// ---- Pointer logic (owner: user) ----
// Attach pointerdown / pointermove / pointerup on `board` here and call
// previewLine(ax, ay, cx, cy) from './canvas' with canvas-pixel coordinates.


let anchor: { x: number, y: number } | null= null;


function to_canvas(e: PointerEvent) {
  const rect = board.getBoundingClientRect();
  return {x : e.clientX - rect.left, y : e.clientY - rect.top}
}


board.addEventListener('pointerdown', (e) => {
  if (e.button !== 0) return;
  anchor = to_canvas(e);
  board.setPointerCapture(e.pointerId)
})


board.addEventListener('pointermove', (e) => {
  if (!anchor) return;
  const cur = to_canvas(e);
  previewLine(anchor.x, anchor.y, cur.x, cur.y)
})

board.addEventListener("pointerup", (e) => {
  if (!anchor) return;
  const cur = to_canvas(e);
  previewLine(anchor.x, anchor.y, cur.x, cur.y);
  anchor = null;
})


board.addEventListener('pointercancel', () => {
  anchor = null
})
