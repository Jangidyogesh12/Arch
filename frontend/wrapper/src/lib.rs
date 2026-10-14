use arch_engine::Counter;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]

pub struct CounterWraper {
    inner: Counter,
}

#[wasm_bindgen]

impl CounterWraper {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Counter::new(),
        }
    }

    pub fn increment(&mut self) {
        self.inner.increment();
    }

    pub fn value(&self) -> u32 {
        self.inner.get()
    }

    pub fn reset(&mut self) {
        self.inner.reset();
    }
}
