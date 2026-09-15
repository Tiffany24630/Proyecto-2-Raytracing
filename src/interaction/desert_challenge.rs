const SEAL_TARGET_STEP: u8 = 1;
const SEAL_STEP_COUNT: u8 = 4;
const CALM_SECONDS: f32 = 3.5;
const WARNING_SECONDS: f32 = 1.4;
const WATCHING_SECONDS: f32 = 1.2;
const REQUIRED_WATCHES: u8 = 3;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DesertPhase {
    #[default]
    AccessSeal,
    Calm,
    Warning,
    Watching,
    Complete,
    Defeated,
}

#[derive(Clone, Copy, Debug)]
pub struct DesertChallenge {
    phase: DesertPhase,
    phase_elapsed: f32,
    lives: u8,
    survived_watches: u8,
    seal_step: u8,
    damaged_this_watch: bool,
}

impl Default for DesertChallenge {
    fn default() -> Self {
        Self {
            phase: DesertPhase::AccessSeal,
            phase_elapsed: 0.0,
            lives: 3,
            survived_watches: 0,
            seal_step: 0,
            damaged_this_watch: false,
        }
    }
}

impl DesertChallenge {
    pub const fn phase(self) -> DesertPhase {
        self.phase
    }

    pub const fn lives(self) -> u8 {
        self.lives
    }

    pub const fn survived_watches(self) -> u8 {
        self.survived_watches
    }

    pub const fn seal_aligned(self) -> bool {
        self.seal_step == SEAL_TARGET_STEP
    }

    pub fn seal_yaw(self) -> f32 {
        self.seal_step as f32 * std::f32::consts::FRAC_PI_2
    }

    pub fn rotate_seal(&mut self) -> bool {
        if self.phase != DesertPhase::AccessSeal {
            return false;
        }
        self.seal_step = (self.seal_step + 1) % SEAL_STEP_COUNT;
        true
    }

    pub fn confirm_seal(&mut self) -> bool {
        if self.phase != DesertPhase::AccessSeal || !self.seal_aligned() {
            return false;
        }
        self.enter_phase(DesertPhase::Calm);
        true
    }

    pub const fn melanta_visible(self) -> bool {
        matches!(self.phase, DesertPhase::Watching | DesertPhase::Defeated)
    }

    pub const fn status(self) -> &'static str {
        match self.phase {
            DesertPhase::AccessSeal if self.seal_aligned() => "SELLO ALINEADO - E ACTIVAR",
            DesertPhase::AccessSeal => "R GIRA EL SELLO HACIA LA PUERTA",
            DesertPhase::Calm => "PUEDES MOVERTE",
            DesertPhase::Warning => "MELANTA SE ACERCA - DETENTE",
            DesertPhase::Watching => "NO TE MUEVAS",
            DesertPhase::Complete => "MEMORIA SUPERADA - E REGRESAR",
            DesertPhase::Defeated => "MEMORIA PERDIDA - E REINTENTAR",
        }
    }

    pub fn update(&mut self, delta_seconds: f32, camera_moved: bool) -> bool {
        if matches!(
            self.phase,
            DesertPhase::AccessSeal | DesertPhase::Complete | DesertPhase::Defeated
        ) {
            return false;
        }

        let mut visual_changed = false;
        if self.phase == DesertPhase::Watching && camera_moved && !self.damaged_this_watch {
            self.lives = self.lives.saturating_sub(1);
            self.damaged_this_watch = true;
            visual_changed = true;
            if self.lives == 0 {
                self.enter_phase(DesertPhase::Defeated);
                return true;
            }
        }

        self.phase_elapsed += delta_seconds.max(0.0);
        let duration = match self.phase {
            DesertPhase::Calm => CALM_SECONDS,
            DesertPhase::Warning => WARNING_SECONDS,
            DesertPhase::Watching => WATCHING_SECONDS,
            _ => return visual_changed,
        };
        if self.phase_elapsed < duration {
            return visual_changed;
        }

        match self.phase {
            DesertPhase::Calm => self.enter_phase(DesertPhase::Warning),
            DesertPhase::Warning => {
                self.damaged_this_watch = false;
                self.enter_phase(DesertPhase::Watching);
            }
            DesertPhase::Watching => {
                self.survived_watches += 1;
                if self.survived_watches >= REQUIRED_WATCHES {
                    self.enter_phase(DesertPhase::Complete);
                } else {
                    self.enter_phase(DesertPhase::Calm);
                }
            }
            _ => {}
        }
        true
    }

    pub fn restart(&mut self) {
        *self = Self::default();
    }

    fn enter_phase(&mut self, phase: DesertPhase) {
        self.phase = phase;
        self.phase_elapsed = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::{CALM_SECONDS, DesertChallenge, DesertPhase, WARNING_SECONDS, WATCHING_SECONDS};

    fn active_challenge() -> DesertChallenge {
        let mut challenge = DesertChallenge::default();
        assert!(challenge.rotate_seal());
        assert!(challenge.confirm_seal());
        challenge
    }

    fn reach_watching(challenge: &mut DesertChallenge) {
        assert!(challenge.update(CALM_SECONDS, false));
        assert_eq!(challenge.phase(), DesertPhase::Warning);
        assert!(challenge.update(WARNING_SECONDS, false));
        assert_eq!(challenge.phase(), DesertPhase::Watching);
    }

    #[test]
    fn seal_requires_one_rotation_and_confirmation() {
        let mut challenge = DesertChallenge::default();
        assert!(!challenge.confirm_seal());
        assert!(challenge.rotate_seal());
        assert!(challenge.seal_aligned());
        assert!(challenge.confirm_seal());
        assert_eq!(challenge.phase(), DesertPhase::Calm);
    }

    #[test]
    fn movement_only_costs_one_life_per_appearance() {
        let mut challenge = active_challenge();
        reach_watching(&mut challenge);
        assert!(challenge.update(0.0, true));
        assert_eq!(challenge.lives(), 2);
        assert!(!challenge.update(0.0, true));
        assert_eq!(challenge.lives(), 2);
    }

    #[test]
    fn remaining_still_survives_three_appearances() {
        let mut challenge = active_challenge();
        for expected in 1..=3 {
            reach_watching(&mut challenge);
            assert!(challenge.update(WATCHING_SECONDS, false));
            assert_eq!(challenge.survived_watches(), expected);
        }
        assert_eq!(challenge.phase(), DesertPhase::Complete);
        assert_eq!(challenge.lives(), 3);
    }

    #[test]
    fn moving_during_all_three_appearances_causes_defeat() {
        let mut challenge = active_challenge();
        for _ in 0..3 {
            reach_watching(&mut challenge);
            assert!(challenge.update(0.0, true));
            if challenge.phase() != DesertPhase::Defeated {
                assert!(challenge.update(WATCHING_SECONDS, false));
            }
        }
        assert_eq!(challenge.phase(), DesertPhase::Defeated);
        assert_eq!(challenge.lives(), 0);
    }
}
