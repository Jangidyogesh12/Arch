import './style.css'
import { initWasm } from './wasm-loader';
import { CounterWrapper } from '../wrapper/pkg/arch_wasm_wrapper'

document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
<section id="center">
  <div>
    <h1>Counter</h1>
    <p>Click the button to increase the count</p>
  </div>
  <button id="counter" type="button" class="counter"></button>
</section>
`


await initWasm();

const counter = new CounterWrapper();

const btn = document.querySelector<HTMLButtonElement>("#counter")!;

btn.onclick = () => {
  btn.textContent = String(counter.increment());
}
