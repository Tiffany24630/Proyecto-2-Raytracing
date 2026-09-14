use crate::{
    geometry::{Cube, Object},
    materials::{Material, crystal, ink, metal},
    math::Vec3,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PuzzlePieceId {
    A,
    B,
    C,
}

impl PuzzlePieceId {
    pub const ALL: [Self; 3] = [Self::A, Self::B, Self::C];

    pub const fn index(self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::C => 2,
        }
    }

    pub const fn object_name(self) -> &'static str {
        match self {
            Self::A => "puzzle piece A",
            Self::B => "puzzle piece B",
            Self::C => "puzzle piece C",
        }
    }

    pub fn from_object_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|piece| piece.object_name() == name)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PuzzlePiecePose {
    pub position: Vec3,
    pub yaw: f32,
}

impl PuzzlePiecePose {
    pub const fn new(position: Vec3, yaw: f32) -> Self {
        Self { position, yaw }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PuzzleLayout {
    pub poses: [PuzzlePiecePose; 3],
    pub selected: Option<PuzzlePieceId>,
}

impl PuzzleLayout {
    pub const fn preserved() -> Self {
        Self {
            poses: [
                PuzzlePiecePose::new(Vec3::new(-0.55, 1.25, -3.10), 0.0),
                PuzzlePiecePose::new(Vec3::new(0.0, 1.25, -3.10), 0.0),
                PuzzlePiecePose::new(Vec3::new(0.55, 1.25, -3.10), 0.0),
            ],
            selected: None,
        }
    }

    pub const fn initial() -> Self {
        Self {
            poses: [
                PuzzlePiecePose::new(Vec3::new(-1.35, 0.68, 0.76), -std::f32::consts::FRAC_PI_6),
                PuzzlePiecePose::new(Vec3::new(0.0, 1.78, 0.62), -std::f32::consts::FRAC_PI_4),
                PuzzlePiecePose::new(Vec3::new(1.35, 0.82, 0.82), -std::f32::consts::FRAC_PI_3),
            ],
            selected: None,
        }
    }

    pub const fn solved() -> Self {
        Self {
            poses: [
                PuzzlePiecePose::new(Vec3::new(-0.55, 1.25, 0.50), 0.0),
                PuzzlePiecePose::new(Vec3::new(0.0, 1.25, 0.50), 0.0),
                PuzzlePiecePose::new(Vec3::new(0.55, 1.25, 0.50), 0.0),
            ],
            selected: None,
        }
    }
}

impl Default for PuzzleLayout {
    fn default() -> Self {
        Self::initial()
    }
}

pub(super) fn add_puzzle_pieces(objects: &mut Vec<Box<dyn Object>>, layout: PuzzleLayout) {
    let materials = [crystal(), ink(), metal()];
    for piece in PuzzlePieceId::ALL {
        let pose = layout.poses[piece.index()];
        let mut material = materials[piece.index()];
        if layout.selected == Some(piece) {
            material.emission = Vec3::new(0.10, 0.30, 0.34);
        }
        add_piece(objects, piece, pose, material);
    }
}

fn add_piece(
    objects: &mut Vec<Box<dyn Object>>,
    piece: PuzzlePieceId,
    pose: PuzzlePiecePose,
    material: Material,
) {
    objects.push(Box::new(Cube::from_center_rotated(
        piece.object_name(),
        pose.position,
        Vec3::new(0.50, 1.25, 0.18),
        pose.yaw,
        material,
    )));
}
