import './style.css'
import { initBoard, getScene, renderScene } from './canvas';

const app = document.querySelector<HTMLDivElement>('#app')!;
app.innerHTML = '';

const board = await initBoard();
app.appendChild(board);

const scene = getScene()!;

// [x, y, z, radius, r, g, b, specular]
const SPHERES: [number, number, number, number, number, number, number, number][] = [
  [0, 0, 6, 0.5, 220, 40, 40, 300],
  [-0.5, 0.18, 5, 0.33, 40, 200, 60, 200],
  [0.55, 0.16, 5.2, 0.35, 40, 80, 220, 200],
];

for (const [x, y, z, radius, r, g, b, s] of SPHERES) {
  scene.addSphere(x, y, z, radius, r, g, b, s);
}

renderScene();
