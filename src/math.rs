pub(crate) struct Point {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

impl Point {
    pub(crate) fn new() -> Point {
        Point { x: 0.0, y: 0.0 }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Amount {
    pub(crate) base: f64,
    pub(crate) delta: f64,
}

impl Amount {
    pub(crate) fn new(value: f64) -> Amount {
        Amount {
            delta: 0.0,
            base: value,
        }
    }
    pub(crate) fn update_base(&mut self) {
        self.base += self.delta;
        self.delta = 0.0;
    }
    pub(crate) fn value(&self) -> f64 {
        self.base + self.delta
    }
}
