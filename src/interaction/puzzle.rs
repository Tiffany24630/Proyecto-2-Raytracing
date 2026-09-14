use crate::{
    geometry::Object,
    math::Vec3,
    raytracing::Camera,
    world::{PuzzleLayout, PuzzlePieceId, PuzzlePiecePose},
};

use super::selection::object_at;

pub const POSITION_TOLERANCE: f32 = 0.12;
pub const ROTATION_TOLERANCE: f32 = 5.0_f32.to_radians();
pub const CAMERA_TOLERANCE: f32 = 8.0_f32.to_radians();

#[derive(Clone, Copy, Debug)]
pub struct Puzzle {
    working: [PuzzlePiecePose; 3],
    committed: [PuzzlePiecePose; 3],
    selected: Option<PuzzlePieceId>,
}

impl Default for Puzzle {
    fn default() -> Self {
        let poses = PuzzleLayout::initial().poses;
        Self {
            working: poses,
            committed: poses,
            selected: None,
        }
    }
}

impl Puzzle {
    pub fn from_layout(layout: PuzzleLayout) -> Self {
        Self {
            working: layout.poses,
            committed: layout.poses,
            selected: layout.selected,
        }
    }

    pub fn interact_at(
        &mut self,
        camera: &Camera,
        objects: &[Box<dyn Object>],
        u: f32,
        v: f32,
    ) -> bool {
        if let Some(piece) = self.selected {
            self.committed[piece.index()] = self.working[piece.index()];
            self.selected = None;
            return true;
        }

        self.selected = object_at(camera, objects, u, v).and_then(PuzzlePieceId::from_object_name);
        self.selected.is_some()
    }

    pub fn move_selected(&mut self, delta: Vec3) -> bool {
        let Some(piece) = self.selected else {
            return false;
        };
        let pose = &mut self.working[piece.index()];
        pose.position += delta;
        pose.position.x = pose.position.x.clamp(-2.2, 2.2);
        pose.position.y = pose.position.y.clamp(0.45, 2.10);
        pose.position.z = pose.position.z.clamp(-0.30, 1.10);
        true
    }

    pub fn rotate_selected(&mut self, yaw_delta: f32) -> bool {
        let Some(piece) = self.selected else {
            return false;
        };
        let pose = &mut self.working[piece.index()];
        pose.yaw = (pose.yaw + yaw_delta).rem_euclid(std::f32::consts::TAU);
        true
    }

    pub fn cancel(&mut self) -> bool {
        let Some(piece) = self.selected else {
            return false;
        };
        self.working[piece.index()] = self.committed[piece.index()];
        self.selected = None;
        true
    }

    pub const fn selected(self) -> Option<PuzzlePieceId> {
        self.selected
    }

    pub const fn layout(self) -> PuzzleLayout {
        PuzzleLayout {
            poses: self.working,
            selected: self.selected,
        }
    }

    pub fn pieces_aligned(self) -> bool {
        let target = PuzzleLayout::solved();
        PuzzlePieceId::ALL.into_iter().all(|piece| {
            let current = self.working[piece.index()];
            let expected = target.poses[piece.index()];
            (current.position - expected.position).length() <= POSITION_TOLERANCE
                && angular_distance(current.yaw, expected.yaw) <= ROTATION_TOLERANCE
        })
    }

    pub fn is_solved(self, camera_aligned: bool) -> bool {
        self.pieces_aligned() && camera_aligned
    }
}

fn angular_distance(left: f32, right: f32) -> f32 {
    let difference = (left - right).rem_euclid(std::f32::consts::TAU);
    difference.min(std::f32::consts::TAU - difference)
}

#[cfg(test)]
mod tests {
    use super::{POSITION_TOLERANCE, Puzzle, ROTATION_TOLERANCE};
    use crate::world::PuzzleLayout;

    #[test]
    fn puzzle_requires_position_rotation_and_camera_conditions() {
        let mut puzzle = Puzzle::from_layout(PuzzleLayout::solved());
        assert!(puzzle.pieces_aligned());
        assert!(!puzzle.is_solved(false));
        assert!(puzzle.is_solved(true));

        puzzle.working[0].position.x += POSITION_TOLERANCE * 2.0;
        assert!(!puzzle.pieces_aligned());

        puzzle = Puzzle::from_layout(PuzzleLayout::solved());
        puzzle.working[1].yaw += ROTATION_TOLERANCE * 2.0;
        assert!(!puzzle.pieces_aligned());
    }

    #[test]
    fn initial_pieces_are_not_aligned() {
        assert!(!Puzzle::default().pieces_aligned());
    }
}
