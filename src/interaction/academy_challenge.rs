#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AcademyPhase {
    #[default]
    Arranging,
    Complete,
    Defeated,
}

#[derive(Clone, Copy, Debug)]
pub struct AcademyChallenge {
    phase: AcademyPhase,
    order: [u8; 3],
    selected: Option<u8>,
}

impl Default for AcademyChallenge {
    fn default() -> Self {
        Self {
            phase: AcademyPhase::Arranging,
            order: [2, 0, 1],
            selected: None,
        }
    }
}

impl AcademyChallenge {
    pub const fn phase(self) -> AcademyPhase {
        self.phase
    }

    pub const fn order(self) -> [u8; 3] {
        self.order
    }

    pub const fn selected(self) -> Option<u8> {
        self.selected
    }

    pub const fn selected_label(self) -> &'static str {
        match self.selected {
            Some(0) => "A",
            Some(1) => "B",
            Some(2) => "C",
            _ => "--",
        }
    }

    pub fn select(&mut self, fragment: u8) -> bool {
        if self.phase != AcademyPhase::Arranging || fragment >= 3 {
            return false;
        }
        self.selected = Some(fragment);
        true
    }

    pub fn move_selected_right(&mut self) -> bool {
        let Some(fragment) = self.selected else {
            return false;
        };
        if self.phase != AcademyPhase::Arranging {
            return false;
        }
        let slot = self
            .order
            .iter()
            .position(|candidate| *candidate == fragment)
            .expect("selected academy fragment must exist");
        let destination = (slot + 1) % self.order.len();
        self.order.swap(slot, destination);
        true
    }

    pub fn confirm(&mut self) -> bool {
        if self.phase != AcademyPhase::Arranging {
            return false;
        }
        self.selected = None;
        self.phase = if self.order == [0, 1, 2] {
            AcademyPhase::Complete
        } else {
            AcademyPhase::Defeated
        };
        true
    }

    pub const fn status(self) -> &'static str {
        match self.phase {
            AcademyPhase::Arranging if self.selected.is_some() => {
                "R MUEVE EL FRAGMENTO A LA DERECHA"
            }
            AcademyPhase::Arranging => "SELECCIONA UN FRAGMENTO DE LA PINTURA",
            AcademyPhase::Complete => "PINTURA RESTAURADA - E REGRESAR",
            AcademyPhase::Defeated => "COMPOSICION INCORRECTA - E REINTENTAR",
        }
    }

    pub fn restart(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::{AcademyChallenge, AcademyPhase};

    #[test]
    fn wrong_confirmation_causes_immediate_defeat() {
        let mut challenge = AcademyChallenge::default();
        assert!(challenge.confirm());
        assert_eq!(challenge.phase(), AcademyPhase::Defeated);
    }

    #[test]
    fn initial_painting_has_a_short_reachable_solution() {
        let mut challenge = AcademyChallenge::default();
        assert!(challenge.select(0));
        assert!(challenge.move_selected_right());
        assert!(challenge.move_selected_right());
        assert_eq!(challenge.order(), [0, 1, 2]);
        assert!(challenge.confirm());
        assert_eq!(challenge.phase(), AcademyPhase::Complete);
    }

    #[test]
    fn retry_restores_the_scrambled_layout() {
        let mut challenge = AcademyChallenge::default();
        challenge.confirm();
        challenge.restart();
        assert_eq!(challenge.phase(), AcademyPhase::Arranging);
        assert_eq!(challenge.order(), [2, 0, 1]);
    }
}
