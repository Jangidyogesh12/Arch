import './style.css'
import { initBoard, getScene, renderScene } from './canvas';

const app = document.querySelector<HTMLDivElement>('#app')!;
app.innerHTML = '';

const board = await initBoard();
app.appendChild(board);

const scene = getScene()!;

// [x, y, z, radius, r, g, b, specular, reflective]
const SPHERES: [number, number, number, number, number, number, number, number, number][] = [
  // Olive ground sphere: its top edge forms the horizon.
  [0, -2.5, 12, 3.0, 170, 140, 25, 0, 0.0],
  // Red front sphere, green/blue behind it reflect the red sphere.
  [0, -0.7, 5.5, 0.85, 220, 40, 40, 300, 0.08],
  [-1.15, 0.1, 6.5, 0.65, 40, 200, 60, 200, 0.6],
  [1.15, 0.1, 6.5, 0.65, 40, 80, 220, 200, 0.6],
];

for (const [x, y, z, radius, r, g, b, s, refl] of SPHERES) {
  scene.addSphere(x, y, z, radius, r, g, b, s, refl);
}

renderScene();
