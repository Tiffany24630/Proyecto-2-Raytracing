use crate::{geometry::Object, math::Vec3, raytracing::Camera, world::MemoryCorePose};

use super::selection::core_at;

const TRANSLATION_LIMIT: f32 = 1.2;

#[derive(Clone, Copy, Debug, Default)]
pub struct EyeOfGod {
    active: bool,
    selection: Option<&'static str>,
    committed_pose: MemoryCorePose,
    working_pose: MemoryCorePose,
}

impl EyeOfGod {
    pub fn toggle(&mut self) {
        if self.active {
            self.working_pose = self.committed_pose;
            self.selection = None;
        }
        self.active = !self.active;
    }

    pub fn interact_at(
        &mut self,
        camera: &Camera,
        objects: &[Box<dyn Object>],
        u: f32,
        v: f32,
    ) -> bool {
        if !self.active {
            return false;
        }

        if self.selection.is_some() {
            self.committed_pose = self.working_pose;
            self.selection = None;
            return true;
        }

        self.selection = core_at(camera, objects, u, v);
        self.selection.is_some()
    }

    pub fn move_selected(&mut self, delta: Vec3) -> bool {
        if self.selection.is_none() {
            return false;
        }
        self.working_pose.offset =
            (self.working_pose.offset + delta).clamp(-TRANSLATION_LIMIT, TRANSLATION_LIMIT);
        true
    }

    pub fn rotate_selected(&mut self, yaw_delta: f32) -> bool {
        if self.selection.is_none() {
            return false;
        }
        self.working_pose.yaw =
            (self.working_pose.yaw + yaw_delta).rem_euclid(std::f32::consts::TAU);
        true
    }

    pub fn cancel(&mut self) -> bool {
        if self.selection.is_some() {
            self.working_pose = self.committed_pose;
            self.selection = None;
            true
        } else if self.active {
            self.active = false;
            true
        } else {
            false
        }
    }

    pub const fn is_active(self) -> bool {
        self.active
    }

    pub const fn selected_name(self) -> Option<&'static str> {
        self.selection
    }

    pub const fn pose(self) -> MemoryCorePose {
        self.working_pose
    }
}

#[cfg(test)]
mod tests {
    use super::EyeOfGod;
    use crate::{
        geometry::{Cube, Object},
        materials::crystal,
        math::Vec3,
        raytracing::Camera,
        world::MemoryCorePose,
    };

    fn selectable_scene() -> Vec<Box<dyn Object>> {
        vec![Box::new(Cube::from_center(
            "memory core",
            Vec3::default(),
            Vec3::new(1.0, 1.0, 1.0),
            crystal(),
        ))]
    }

    #[test]
    fn selection_move_rotation_cancel_and_confirm_are_reversible() {
        let camera = Camera::orbital(Vec3::default(), 5.0, 0.0, 0.0, 45.0, 16.0 / 9.0);
        let scene = selectable_scene();
        let mut eye = EyeOfGod::default();

        eye.toggle();
        assert!(eye.interact_at(&camera, &scene, 0.5, 0.5));
        assert_eq!(eye.selected_name(), Some("memory core"));
        assert!(eye.move_selected(Vec3::new(0.4, 0.0, -0.2)));
        assert!(eye.rotate_selected(0.5));
        assert_ne!(eye.pose(), MemoryCorePose::default());

        assert!(eye.cancel());
        assert_eq!(eye.pose(), MemoryCorePose::default());

        assert!(eye.interact_at(&camera, &scene, 0.5, 0.5));
        assert!(eye.move_selected(Vec3::new(0.3, 0.0, 0.0)));
        assert!(eye.interact_at(&camera, &scene, 0.5, 0.5));
        assert_eq!(eye.selected_name(), None);
        assert_eq!(eye.pose().offset, Vec3::new(0.3, 0.0, 0.0));
    }
}
