const TIME_LIMIT_SECONDS: f32 = 45.0;
const ALL_PAGES: u8 = 0b111;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LibraryPhase {
    #[default]
    Collecting,
    Complete,
    Defeated,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LibraryThreatStage {
    #[default]
    Distant,
    Near,
    Imminent,
    Defeated,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LibraryUpdate {
    pub ui_changed: bool,
    pub visual_changed: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct LibraryChallenge {
    phase: LibraryPhase,
    remaining_seconds: f32,
    collected: u8,
    displayed_second: u8,
    threat: LibraryThreatStage,
}

impl Default for LibraryChallenge {
    fn default() -> Self {
        Self {
            phase: LibraryPhase::Collecting,
            remaining_seconds: TIME_LIMIT_SECONDS,
            collected: 0,
            displayed_second: TIME_LIMIT_SECONDS as u8,
            threat: LibraryThreatStage::Distant,
        }
    }
}

impl LibraryChallenge {
    pub const fn phase(self) -> LibraryPhase {
        self.phase
    }

    pub const fn collected_count(self) -> u32 {
        self.collected.count_ones()
    }

    pub const fn page_collected(self, page: u8) -> bool {
        page < 3 && self.collected & (1 << page) != 0
    }

    pub fn remaining_display(self) -> u8 {
        self.remaining_seconds.ceil().clamp(0.0, TIME_LIMIT_SECONDS) as u8
    }

    pub const fn corruption(self) -> f32 {
        match self.threat {
            LibraryThreatStage::Distant => 0.0,
            LibraryThreatStage::Near => 0.38,
            LibraryThreatStage::Imminent => 0.72,
            LibraryThreatStage::Defeated => 1.0,
        }
    }

    pub const fn status(self) -> &'static str {
        match self.phase {
            LibraryPhase::Complete => "LIBRO RESTAURADO - E REGRESAR",
            LibraryPhase::Defeated => "EL TIEMPO SE AGOTO - E REINTENTAR",
            LibraryPhase::Collecting => match self.threat {
                LibraryThreatStage::Distant => "ENCUENTRA LAS TRES PAGINAS",
                LibraryThreatStage::Near => "MELANTA SE ACERCA",
                LibraryThreatStage::Imminent => "MELANTA ESTA MUY CERCA",
                LibraryThreatStage::Defeated => "EL TIEMPO SE AGOTO",
            },
        }
    }

    pub fn collect(&mut self, page: u8) -> bool {
        if self.phase != LibraryPhase::Collecting || page >= 3 || self.page_collected(page) {
            return false;
        }
        self.collected |= 1 << page;
        if self.collected == ALL_PAGES {
            self.phase = LibraryPhase::Complete;
        }
        true
    }

    pub fn update(&mut self, delta_seconds: f32) -> LibraryUpdate {
        if self.phase != LibraryPhase::Collecting || delta_seconds <= 0.0 {
            return LibraryUpdate::default();
        }

        let previous_second = self.displayed_second;
        let previous_threat = self.threat;
        self.remaining_seconds = (self.remaining_seconds - delta_seconds).max(0.0);
        self.displayed_second = self.remaining_display();
        self.threat = if self.remaining_seconds <= 0.0 {
            self.phase = LibraryPhase::Defeated;
            LibraryThreatStage::Defeated
        } else if self.remaining_seconds <= 15.0 {
            LibraryThreatStage::Imminent
        } else if self.remaining_seconds <= 30.0 {
            LibraryThreatStage::Near
        } else {
            LibraryThreatStage::Distant
        };

        LibraryUpdate {
            ui_changed: self.displayed_second != previous_second
                || self.phase == LibraryPhase::Defeated,
            visual_changed: self.threat != previous_threat,
        }
    }

    pub fn restart(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::{LibraryChallenge, LibraryPhase};

    #[test]
    fn three_unique_pages_restore_the_book() {
        let mut challenge = LibraryChallenge::default();
        assert!(challenge.collect(0));
        assert!(!challenge.collect(0));
        assert!(challenge.collect(1));
        assert!(challenge.collect(2));
        assert_eq!(challenge.collected_count(), 3);
        assert_eq!(challenge.phase(), LibraryPhase::Complete);
    }

    #[test]
    fn threat_approaches_in_stages_and_timeout_defeats_the_player() {
        let mut challenge = LibraryChallenge::default();
        assert!(challenge.update(16.0).visual_changed);
        assert_eq!(challenge.corruption(), 0.38);
        assert!(challenge.update(15.0).visual_changed);
        assert_eq!(challenge.corruption(), 0.72);
        assert!(challenge.update(20.0).visual_changed);
        assert_eq!(challenge.phase(), LibraryPhase::Defeated);
        assert_eq!(challenge.remaining_display(), 0);
    }

    #[test]
    fn timer_reports_ui_ticks_without_visual_scene_changes() {
        let mut challenge = LibraryChallenge::default();
        let update = challenge.update(1.0);
        assert!(update.ui_changed);
        assert!(!update.visual_changed);
    }
}
