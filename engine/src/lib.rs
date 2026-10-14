pub struct Counter {
    value: u32,
}

impl Counter {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn get(&self) -> u32 {
        return self.value;
    }

    pub fn reset(&mut self) {
        self.value = 0;
    }

    pub fn increment(&mut self) -> u32 {
        self.value += 1;
        return self.value;
    }
}


