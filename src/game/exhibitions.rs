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
