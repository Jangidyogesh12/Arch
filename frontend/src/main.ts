import './style.css'

document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
<section id="center">
  <div>
    <h1>Counter</h1>
    <p>Click the button to increase the count</p>
  </div>
  <button id="counter" type="button" class="counter"></button>
</section>
`
