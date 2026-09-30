use crate::{
    geometry::Object,
    math::Vec3,
    raytracing::Camera,
    world::{PuzzleLayout, PuzzlePieceId, PuzzlePiecePose},
};

use super::selection::puzzle_piece_at;

pub const POSITION_TOLERANCE: f32 = 0.16;
pub const ROTATION_TOLERANCE: f32 = 8.0_f32.to_radians();
pub const CAMERA_TOLERANCE: f32 = 12.0_f32.to_radians();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PieceStatus {
    Fragmented,
    Near,
    Aligned,
}

impl PieceStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Fragmented => "--",
            Self::Near => "CERCA",
            Self::Aligned => "OK",
        }
    }
}

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

        self.selected =
            puzzle_piece_at(camera, objects, u, v).and_then(PuzzlePieceId::from_object_name);
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
        PuzzlePieceId::ALL
            .into_iter()
            .all(|piece| self.piece_aligned(piece))
    }

    pub fn aligned_count(self) -> usize {
        PuzzlePieceId::ALL
            .into_iter()
            .filter(|piece| self.piece_aligned(*piece))
            .count()
    }

    pub fn piece_statuses(self) -> [PieceStatus; 3] {
        PuzzlePieceId::ALL.map(|piece| {
            if self.piece_aligned(piece) {
                PieceStatus::Aligned
            } else {
                let current = self.working[piece.index()];
                let expected = PuzzleLayout::solved().poses[piece.index()];
                if (current.position - expected.position).length() <= POSITION_TOLERANCE * 2.5 {
                    PieceStatus::Near
                } else {
                    PieceStatus::Fragmented
                }
            }
        })
    }

    pub const fn selected_label(self) -> &'static str {
        match self.selected {
            Some(PuzzlePieceId::A) => "A",
            Some(PuzzlePieceId::B) => "B",
            Some(PuzzlePieceId::C) => "C",
            None => "--",
        }
    }

    pub fn interaction_hint(self, camera_aligned: bool) -> &'static str {
        let Some(piece) = self.selected else {
            return if self.pieces_aligned() {
                if camera_aligned {
                    "PERSPECTIVA CORRECTA"
                } else {
                    "PIEZAS LISTAS: GIRA A LA DERECHA CON D"
                }
            } else {
                "APUNTA A UN FRAGMENTO Y PULSA E"
            };
        };

        let current = self.working[piece.index()];
        let expected = PuzzleLayout::solved().poses[piece.index()];
        let delta = expected.position - current.position;
        if delta.y > POSITION_TOLERANCE {
            "SUBE CON PAGEUP"
        } else if delta.y < -POSITION_TOLERANCE {
            "BAJA CON PAGEDOWN"
        } else if delta.x > POSITION_TOLERANCE {
            "MUEVE A LA DERECHA"
        } else if delta.x < -POSITION_TOLERANCE {
            "MUEVE A LA IZQUIERDA"
        } else if delta.z < -POSITION_TOLERANCE {
            "MUEVE AL FONDO CON FLECHA ARRIBA"
        } else if delta.z > POSITION_TOLERANCE {
            "ACERCA CON FLECHA ABAJO"
        } else if angular_distance(current.yaw, expected.yaw) > ROTATION_TOLERANCE {
            "ROTA CON R"
        } else {
            "DESTINO ALCANZADO: PULSA E"
        }
    }

    pub fn is_solved(self, camera_aligned: bool) -> bool {
        self.pieces_aligned() && camera_aligned
    }

    fn piece_aligned(self, piece: PuzzlePieceId) -> bool {
        let current = self.working[piece.index()];
        let expected = PuzzleLayout::solved().poses[piece.index()];
        (current.position - expected.position).length() <= POSITION_TOLERANCE
            && angular_distance(current.yaw, expected.yaw) <= ROTATION_TOLERANCE
    }
}

fn angular_distance(left: f32, right: f32) -> f32 {
    let difference = (left - right).rem_euclid(std::f32::consts::TAU);
    difference.min(std::f32::consts::TAU - difference)
}
