#[derive(Clone, Copy, Debug, Default)]
pub struct EyeOfGod {
    active: bool,
}

impl EyeOfGod {
    pub fn toggle(&mut self) {
        self.active = !self.active;
    }

    pub const fn is_active(self) -> bool {
        self.active
    }
}
