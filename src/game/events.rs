#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameEvent {
    UsePortal,
    TransitionComplete,
    TogglePuzzle,
    PuzzleSolved,
    BeginMelanta,
    Finish,
}
