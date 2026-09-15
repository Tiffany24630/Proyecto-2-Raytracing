#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExhibitionId {
    DesertPavilion,
    MahavaipulyaChamber,
    LuyangAcademy,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TransitionDestination {
    #[default]
    Temple,
    Exhibition(ExhibitionId),
}

impl ExhibitionId {
    pub const ALL: [Self; 3] = [
        Self::DesertPavilion,
        Self::MahavaipulyaChamber,
        Self::LuyangAcademy,
    ];

    const fn bit(self) -> u8 {
        match self {
            Self::DesertPavilion => 1 << 0,
            Self::MahavaipulyaChamber => 1 << 1,
            Self::LuyangAcademy => 1 << 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExhibitionProgress {
    hub_unlocked: bool,
    completed: u8,
}

impl ExhibitionProgress {
    const ALL_COMPLETED: u8 = (1 << ExhibitionId::ALL.len()) - 1;

    pub const fn new() -> Self {
        Self {
            hub_unlocked: false,
            completed: 0,
        }
    }

    pub const fn hub_unlocked(self) -> bool {
        self.hub_unlocked
    }

    pub fn unlock_hub(&mut self) {
        self.hub_unlocked = true;
    }

    pub const fn can_enter(self, exhibition: ExhibitionId) -> bool {
        self.hub_unlocked && !self.is_completed(exhibition)
    }

    pub const fn is_completed(self, exhibition: ExhibitionId) -> bool {
        self.completed & exhibition.bit() != 0
    }

    pub fn complete(&mut self, exhibition: ExhibitionId) -> bool {
        if !self.hub_unlocked || self.is_completed(exhibition) {
            return false;
        }
        self.completed |= exhibition.bit();
        true
    }

    pub const fn completed_count(self) -> u32 {
        (self.completed & Self::ALL_COMPLETED).count_ones()
    }

    pub const fn all_completed(self) -> bool {
        self.completed & Self::ALL_COMPLETED == Self::ALL_COMPLETED
    }
}

#[cfg(test)]
mod tests {
    use super::{ExhibitionId, ExhibitionProgress};

    #[test]
    fn exhibitions_are_locked_until_the_memory_core_is_restored() {
        let mut progress = ExhibitionProgress::default();
        assert!(!progress.can_enter(ExhibitionId::DesertPavilion));

        progress.unlock_hub();
        assert!(progress.can_enter(ExhibitionId::DesertPavilion));
    }

    #[test]
    fn completion_is_unique_and_the_epilogue_requires_all_three() {
        let mut progress = ExhibitionProgress::default();
        progress.unlock_hub();

        for (index, exhibition) in ExhibitionId::ALL.into_iter().enumerate() {
            assert!(progress.complete(exhibition));
            assert!(!progress.complete(exhibition));
            assert_eq!(progress.completed_count(), (index + 1) as u32);
        }

        assert!(progress.all_completed());
        assert!(!progress.can_enter(ExhibitionId::LuyangAcademy));
    }
}
