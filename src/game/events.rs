use super::ExhibitionId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameEvent {
    UsePortal,
    EnterExhibition(ExhibitionId),
    LeaveExhibition,
    TransitionComplete,
    ActivateEyeOfGod,
    TogglePuzzle,
    PuzzleSolved,
    BeginMelanta,
    Finish,
}
