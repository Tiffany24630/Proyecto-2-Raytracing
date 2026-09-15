use super::{GameEvent, SceneState};

#[derive(Clone, Copy, Debug, Default)]
pub struct GameState {
    scene: SceneState,
}

impl GameState {
    pub const fn from_scene(scene: SceneState) -> Self {
        Self { scene }
    }

    pub const fn scene(self) -> SceneState {
        self.scene
    }

    pub const fn available_events(self) -> &'static [GameEvent] {
        match self.scene {
            SceneState::Exterior => &[GameEvent::UsePortal],
            SceneState::Entering => &[GameEvent::TransitionComplete],
            SceneState::Temple => &[GameEvent::UsePortal, GameEvent::TogglePuzzle],
            SceneState::Puzzle => &[GameEvent::TogglePuzzle, GameEvent::PuzzleSolved],
            SceneState::MemoryRestored => &[GameEvent::BeginMelanta],
            SceneState::Melanta => &[GameEvent::Finish],
            SceneState::Final => &[],
        }
    }

    pub fn handle(&mut self, event: GameEvent) -> bool {
        let next = match (self.scene, event) {
            (SceneState::Exterior, GameEvent::UsePortal) => SceneState::Entering,
            (SceneState::Entering, GameEvent::TransitionComplete) => SceneState::Temple,
            (SceneState::Temple, GameEvent::UsePortal) => SceneState::Exterior,
            (SceneState::Temple, GameEvent::TogglePuzzle) => SceneState::Puzzle,
            (SceneState::Puzzle, GameEvent::TogglePuzzle) => SceneState::Temple,
            (SceneState::Puzzle, GameEvent::PuzzleSolved) => SceneState::MemoryRestored,
            (SceneState::MemoryRestored, GameEvent::BeginMelanta) => SceneState::Melanta,
            (SceneState::Melanta, GameEvent::Finish) => SceneState::Final,
            _ => return false,
        };
        self.scene = next;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::GameState;
    use crate::game::{GameEvent, SceneState};

    #[test]
    fn progression_follows_the_required_state_sequence() {
        let mut game = GameState::default();
        assert_eq!(game.scene(), SceneState::Exterior);
        for (event, expected) in [
            (GameEvent::UsePortal, SceneState::Entering),
            (GameEvent::TransitionComplete, SceneState::Temple),
            (GameEvent::TogglePuzzle, SceneState::Puzzle),
            (GameEvent::PuzzleSolved, SceneState::MemoryRestored),
            (GameEvent::BeginMelanta, SceneState::Melanta),
            (GameEvent::Finish, SceneState::Final),
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
}
