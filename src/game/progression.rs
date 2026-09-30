use super::{ExhibitionId, ExhibitionProgress, GameEvent, SceneState, TransitionDestination};

#[derive(Clone, Copy, Debug, Default)]
pub struct GameState {
    scene: SceneState,
    exhibitions: ExhibitionProgress,
    transition_destination: TransitionDestination,
}

impl GameState {
    pub const fn from_scene(scene: SceneState) -> Self {
        Self {
            scene,
            exhibitions: ExhibitionProgress::new(),
            transition_destination: TransitionDestination::Temple,
        }
    }

    pub const fn scene(self) -> SceneState {
        self.scene
    }

    pub const fn exhibitions(self) -> ExhibitionProgress {
        self.exhibitions
    }

    pub const fn transition_destination(self) -> TransitionDestination {
        self.transition_destination
    }

    pub fn complete_exhibition(&mut self, exhibition: ExhibitionId) -> bool {
        self.exhibitions.complete(exhibition)
    }

    pub const fn available_events(self) -> &'static [GameEvent] {
        match self.scene {
            SceneState::Exterior => &[GameEvent::UsePortal],
            SceneState::Entering => &[GameEvent::TransitionComplete],
            SceneState::Temple => &[
                GameEvent::UsePortal,
                GameEvent::ActivateEyeOfGod,
                GameEvent::EnterExhibition(ExhibitionId::DesertPavilion),
                GameEvent::EnterExhibition(ExhibitionId::MahavaipulyaChamber),
                GameEvent::EnterExhibition(ExhibitionId::LuyangAcademy),
            ],
            SceneState::Puzzle => &[GameEvent::TogglePuzzle, GameEvent::PuzzleSolved],
            SceneState::MemoryRestored => &[GameEvent::BeginMelanta],
            SceneState::Melanta => &[GameEvent::Finish],
            SceneState::DesertPavilion => &[GameEvent::LeaveExhibition],
            SceneState::MahavaipulyaChamber => &[GameEvent::LeaveExhibition],
            SceneState::LuyangAcademy => &[GameEvent::LeaveExhibition],
            SceneState::Final => &[],
        }
    }

    pub fn handle(&mut self, event: GameEvent) -> bool {
        let next = match (self.scene, event) {
            (SceneState::Exterior, GameEvent::UsePortal) => {
                self.transition_destination = TransitionDestination::Temple;
                SceneState::Entering
            }
            (SceneState::Entering, GameEvent::TransitionComplete) => {
                match self.transition_destination {
                    TransitionDestination::Temple if self.exhibitions.all_completed() => {
                        SceneState::Final
                    }
                    TransitionDestination::Temple => SceneState::Temple,
                    TransitionDestination::Exhibition(ExhibitionId::DesertPavilion) => {
                        SceneState::DesertPavilion
                    }
                    TransitionDestination::Exhibition(ExhibitionId::MahavaipulyaChamber) => {
                        SceneState::MahavaipulyaChamber
                    }
                    TransitionDestination::Exhibition(ExhibitionId::LuyangAcademy) => {
                        SceneState::LuyangAcademy
                    }
                }
            }
            (SceneState::Temple, GameEvent::UsePortal) => SceneState::Exterior,
            (SceneState::Temple, GameEvent::ActivateEyeOfGod) => SceneState::Temple,
            (SceneState::Temple, GameEvent::EnterExhibition(exhibition))
                if self.exhibitions.can_enter(exhibition) =>
            {
                self.transition_destination = TransitionDestination::Exhibition(exhibition);
                SceneState::Entering
            }
            (SceneState::Puzzle, GameEvent::TogglePuzzle) => SceneState::Temple,
            (SceneState::Puzzle, GameEvent::PuzzleSolved) => SceneState::MemoryRestored,
            (SceneState::MemoryRestored, GameEvent::BeginMelanta) => SceneState::Melanta,
            (SceneState::Melanta, GameEvent::Finish) if self.exhibitions.hub_unlocked() => {
                SceneState::Temple
            }
            (SceneState::Melanta, GameEvent::Finish) => SceneState::Final,
            (SceneState::DesertPavilion, GameEvent::LeaveExhibition) => {
                self.transition_destination = TransitionDestination::Temple;
                SceneState::Entering
            }
            (SceneState::MahavaipulyaChamber, GameEvent::LeaveExhibition) => {
                self.transition_destination = TransitionDestination::Temple;
                SceneState::Entering
            }
            (SceneState::LuyangAcademy, GameEvent::LeaveExhibition) => {
                self.transition_destination = TransitionDestination::Temple;
                SceneState::Entering
            }
            _ => return false,
        };
        self.scene = next;
        if matches!(event, GameEvent::ActivateEyeOfGod | GameEvent::PuzzleSolved) {
            self.exhibitions.unlock_hub();
        }
        true
    }
}
