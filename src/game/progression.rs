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

#[cfg(test)]
mod tests {
    use super::GameState;
    use crate::game::{ExhibitionId, GameEvent, SceneState, TransitionDestination};

    #[test]
    fn progression_follows_the_required_state_sequence() {
        let mut game = GameState::default();
        assert_eq!(game.scene(), SceneState::Exterior);
        for (event, expected) in [
            (GameEvent::UsePortal, SceneState::Entering),
            (GameEvent::TransitionComplete, SceneState::Temple),
            (GameEvent::ActivateEyeOfGod, SceneState::Temple),
        ] {
            assert!(game.handle(event));
            assert_eq!(game.scene(), expected);
        }
    }

    #[test]
    fn invalid_event_does_not_change_state() {
        let mut game = GameState::default();
        assert!(!game.handle(GameEvent::PuzzleSolved));
        assert_eq!(game.scene(), SceneState::Exterior);
    }

    #[test]
    fn restoring_the_core_unlocks_exhibitions_without_completing_them() {
        let mut game = GameState::from_scene(SceneState::Puzzle);
        assert!(game.handle(GameEvent::PuzzleSolved));
        assert!(game.exhibitions().hub_unlocked());
        assert_eq!(game.exhibitions().completed_count(), 0);
    }

    #[test]
    fn eye_of_god_unlocks_exhibitions_without_the_fragment_puzzle() {
        let mut game = GameState::from_scene(SceneState::Temple);
        assert!(game.handle(GameEvent::ActivateEyeOfGod));
        assert_eq!(game.scene(), SceneState::Temple);
        assert!(game.exhibitions().hub_unlocked());
        assert!(game
            .exhibitions()
            .can_enter(ExhibitionId::DesertPavilion));
        assert!(game
            .exhibitions()
            .can_enter(ExhibitionId::MahavaipulyaChamber));
    }

    #[test]
    fn game_state_tracks_each_completed_exhibition_once() {
        let mut game = GameState::from_scene(SceneState::Puzzle);
        assert!(game.handle(GameEvent::PuzzleSolved));

        for exhibition in crate::game::ExhibitionId::ALL {
            assert!(game.complete_exhibition(exhibition));
            assert!(!game.complete_exhibition(exhibition));
        }

        assert!(game.exhibitions().all_completed());
    }

    #[test]
    fn desert_entry_is_locked_then_uses_a_typed_round_trip_transition() {
        let mut game = GameState::from_scene(SceneState::Temple);
        assert!(!game.handle(GameEvent::EnterExhibition(ExhibitionId::DesertPavilion)));

        game = GameState::from_scene(SceneState::Puzzle);
        assert!(game.handle(GameEvent::PuzzleSolved));
        assert!(game.handle(GameEvent::BeginMelanta));
        assert!(game.handle(GameEvent::Finish));
        assert!(game.handle(GameEvent::EnterExhibition(ExhibitionId::DesertPavilion)));
        assert_eq!(game.scene(), SceneState::Entering);
        assert_eq!(
            game.transition_destination(),
            TransitionDestination::Exhibition(ExhibitionId::DesertPavilion)
        );
        assert!(game.handle(GameEvent::TransitionComplete));
        assert_eq!(game.scene(), SceneState::DesertPavilion);

        assert!(game.handle(GameEvent::LeaveExhibition));
        assert_eq!(game.scene(), SceneState::Entering);
        assert_eq!(game.transition_destination(), TransitionDestination::Temple);
        assert!(game.handle(GameEvent::TransitionComplete));
        assert_eq!(game.scene(), SceneState::Temple);
    }

    #[test]
    fn library_uses_the_same_preserved_round_trip_contract() {
        let mut game = GameState::from_scene(SceneState::Puzzle);
        assert!(game.handle(GameEvent::PuzzleSolved));
        assert!(game.handle(GameEvent::BeginMelanta));
        assert!(game.handle(GameEvent::Finish));
        assert!(game.handle(GameEvent::EnterExhibition(
            ExhibitionId::MahavaipulyaChamber
        )));
        assert!(game.handle(GameEvent::TransitionComplete));
        assert_eq!(game.scene(), SceneState::MahavaipulyaChamber);
        assert!(game.complete_exhibition(ExhibitionId::MahavaipulyaChamber));
        assert!(game.handle(GameEvent::LeaveExhibition));
        assert!(game.handle(GameEvent::TransitionComplete));
        assert_eq!(game.scene(), SceneState::Temple);
    }

    #[test]
    fn eye_of_god_opens_luyang_without_the_old_perspective_puzzle() {
        let mut game = GameState::from_scene(SceneState::Temple);
        assert!(game.handle(GameEvent::ActivateEyeOfGod));
        assert!(game.handle(GameEvent::EnterExhibition(
            ExhibitionId::LuyangAcademy
        )));
        assert!(game.handle(GameEvent::TransitionComplete));
        assert_eq!(game.scene(), SceneState::LuyangAcademy);
        assert!(game.complete_exhibition(ExhibitionId::LuyangAcademy));
        assert!(game.handle(GameEvent::LeaveExhibition));
        assert!(game.handle(GameEvent::TransitionComplete));
        assert_eq!(game.scene(), SceneState::Temple);
    }

    #[test]
    fn returning_after_all_three_memories_starts_the_epilogue() {
        let mut game = GameState::from_scene(SceneState::Temple);
        assert!(game.handle(GameEvent::ActivateEyeOfGod));
        assert!(game.complete_exhibition(ExhibitionId::DesertPavilion));
        assert!(game.complete_exhibition(ExhibitionId::MahavaipulyaChamber));
        assert!(game.handle(GameEvent::EnterExhibition(
            ExhibitionId::LuyangAcademy
        )));
        assert!(game.handle(GameEvent::TransitionComplete));
        assert!(game.complete_exhibition(ExhibitionId::LuyangAcademy));
        assert!(game.handle(GameEvent::LeaveExhibition));
        assert!(game.handle(GameEvent::TransitionComplete));
        assert_eq!(game.scene(), SceneState::Final);
        assert!(game.exhibitions().all_completed());
    }

    #[test]
    fn every_exhibition_order_reaches_the_same_final_epilogue() {
        for order in [
            [
                ExhibitionId::DesertPavilion,
                ExhibitionId::MahavaipulyaChamber,
                ExhibitionId::LuyangAcademy,
            ],
            [
                ExhibitionId::DesertPavilion,
                ExhibitionId::LuyangAcademy,
                ExhibitionId::MahavaipulyaChamber,
            ],
            [
                ExhibitionId::MahavaipulyaChamber,
                ExhibitionId::DesertPavilion,
                ExhibitionId::LuyangAcademy,
            ],
            [
                ExhibitionId::MahavaipulyaChamber,
                ExhibitionId::LuyangAcademy,
                ExhibitionId::DesertPavilion,
            ],
            [
                ExhibitionId::LuyangAcademy,
                ExhibitionId::DesertPavilion,
                ExhibitionId::MahavaipulyaChamber,
            ],
            [
                ExhibitionId::LuyangAcademy,
                ExhibitionId::MahavaipulyaChamber,
                ExhibitionId::DesertPavilion,
            ],
        ] {
            let mut game = GameState::from_scene(SceneState::Temple);
            assert!(game.handle(GameEvent::ActivateEyeOfGod));
            for (index, exhibition) in order.into_iter().enumerate() {
                assert!(game.handle(GameEvent::EnterExhibition(exhibition)));
                assert!(game.handle(GameEvent::TransitionComplete));
                assert!(game.complete_exhibition(exhibition));
                assert!(game.handle(GameEvent::LeaveExhibition));
                assert!(game.handle(GameEvent::TransitionComplete));
                assert_eq!(
                    game.scene(),
                    if index == 2 {
                        SceneState::Final
                    } else {
                        SceneState::Temple
                    }
                );
            }
        }
    }
}
