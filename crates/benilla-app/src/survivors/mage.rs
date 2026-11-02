//! Mage-only absorption, separate from equipment and talent shields.
#[derive(Default)]
pub(super) struct Barrier {
    pub capacity: f32,
    pub left: f32,
    pub feedback: Vec<(f32, bool)>,
}

impl Barrier {
    pub fn present(&self) -> bool {
        self.capacity > 0.0 && self.left > 0.0
    }

    pub fn cast(&mut self, capacity: f32, duration: f32) -> bool {
        if self.present() {
            return false;
        }
        self.capacity = capacity;
        self.left = duration;
        true
    }

    pub fn clear(&mut self) {
        self.capacity = 0.0;
        self.left = 0.0;
    }

    pub fn tick(&mut self, dt: f32) {
        self.left = (self.left - dt).max(0.0);
        if !self.present() {
            self.clear();
        }
    }

    pub fn absorb(&mut self, damage: f32) -> f32 {
        if !self.present() {
            return 0.0;
        }
        let absorbed = damage.min(self.capacity);
        self.capacity -= absorbed;
        if absorbed > 0.0 {
            self.feedback.push((absorbed, damage > absorbed));
        }
        if self.capacity <= 0.0 {
            self.clear();
        }
        absorbed
    }
}
