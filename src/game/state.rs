use crate::world::MemoryCoreState;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SceneState {
    #[default]
    Exterior,
    Entering,
    Temple,
    Puzzle,
    MemoryRestored,
    Melanta,
    Final,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlMode {
    Portal,
    Camera,
    EyeOfGod,
    Locked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkyState {
    Normal,
    Melanta,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnimationState {
    None,
    PortalEntry,
    MemoryReveal,
    Corruption,
}

impl SceneState {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Exterior => "EXTERIOR",
            Self::Entering => "ENTERING",
            Self::Temple => "TEMPLE",
            Self::Puzzle => "PUZZLE",
            Self::MemoryRestored => "MEMORY RESTORED",
            Self::Melanta => "MELANTA",
            Self::Final => "FINAL",
        }
    }

    pub const fn file_name(self) -> &'static str {
        match self {
            Self::Exterior => "exterior",
            Self::Entering => "entering",
            Self::Temple => "temple",
            Self::Puzzle => "puzzle",
            Self::MemoryRestored => "memory_restored",
            Self::Melanta => "melanta",
            Self::Final => "final",
        }
    }

    pub const fn controls(self) -> ControlMode {
        match self {
            Self::Exterior => ControlMode::Portal,
            Self::Entering | Self::Melanta => ControlMode::Locked,
            Self::Temple | Self::MemoryRestored | Self::Final => ControlMode::Camera,
            Self::Puzzle => ControlMode::EyeOfGod,
        }
    }

    pub const fn sky(self) -> SkyState {
        match self {
            Self::Melanta => SkyState::Melanta,
            _ => SkyState::Normal,
        }
    }

    pub const fn animation(self) -> AnimationState {
        match self {
            Self::Entering => AnimationState::PortalEntry,
            Self::MemoryRestored => AnimationState::MemoryReveal,
            Self::Melanta => AnimationState::Corruption,
            _ => AnimationState::None,
        }
    }

    pub const fn memory_core(self) -> MemoryCoreState {
        match self {
            Self::Puzzle | Self::Melanta => MemoryCoreState::Fragmented,
            Self::MemoryRestored | Self::Final => MemoryCoreState::Restored,
            _ => MemoryCoreState::Stable,
        }
    }

    pub fn object_visible(self, name: &str) -> bool {
        let exterior_detail =
            name == "wind fragment" || (name.starts_with("exterior ") && name != "exterior floor");
        let puzzle_piece = name.starts_with("puzzle piece");
        let eye_aid = name.starts_with("spatial ") || name.starts_with("dex ");
        let interior_detail = puzzle_piece
            || eye_aid
            || name.starts_with("memory ")
            || name.starts_with("restored memory")
            || name.starts_with("preserved echo")
            || name.starts_with("pedestal")
            || name.starts_with("luyang")
            || name.starts_with("mahavaipulya")
            || name.starts_with("desert ");

        match self {
            Self::Exterior => !interior_detail,
            Self::Entering => true,
            Self::Temple => !exterior_detail && !puzzle_piece && !eye_aid,
            Self::Puzzle => !exterior_detail,
            Self::MemoryRestored | Self::Melanta | Self::Final => !exterior_detail && !eye_aid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AnimationState, ControlMode, SceneState, SkyState};

    #[test]
    fn states_define_controls_sky_animation_and_visibility() {
        assert_eq!(SceneState::Exterior.controls(), ControlMode::Portal);
        assert!(!SceneState::Exterior.object_visible("memory core"));
        assert!(!SceneState::Temple.object_visible("puzzle piece A"));
        assert!(SceneState::Puzzle.object_visible("puzzle piece A"));
        assert!(!SceneState::Puzzle.object_visible("exterior rock"));
        assert_eq!(SceneState::Melanta.sky(), SkyState::Melanta);
        assert_eq!(
            SceneState::Entering.animation(),
            AnimationState::PortalEntry
        );
    }
}
