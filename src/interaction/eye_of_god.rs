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

#[cfg(test)]
mod tests {
    use super::EyeOfGod;

    #[test]
    fn mode_can_be_enabled_and_disabled() {
        let mut eye = EyeOfGod::default();
        assert!(!eye.is_active());
        eye.toggle();
        assert!(eye.is_active());
        eye.toggle();
        assert!(!eye.is_active());
    }
}
