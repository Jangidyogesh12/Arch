import init from "../wrapper/pkg/arch_wasm_wrapper"

let initiated = false;

export async function initWasm(){
  if (!initiated) {
    await init();
    initiated = true;
  }
}
