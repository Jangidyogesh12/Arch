import './style.css'
import { initBoard, getScene, renderScene } from './canvas';

const app = document.querySelector<HTMLDivElement>('#app')!;
app.innerHTML = '';

const board = await initBoard();
app.appendChild(board);

const scene = getScene()!;

// [x, y, z, radius, r, g, b]
const SPHERES: [number, number, number, number, number, number, number][] = [
  [-0.2, 0, 10, 0.15, 255, 0, 0],
  [0.2, 0, 5, 0.15, 0, 255, 0],
  [0, 0.2, 5, 0.12, 0, 0, 255],
];

for (const [x, y, z, radius, r, g, b] of SPHERES) {
  scene.addSphere(x, y, z, radius, r, g, b);
}

renderScene();
