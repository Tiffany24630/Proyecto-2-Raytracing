use crate::{
    math::Vec3,
    raytracing::Light,
    world::{MemoryCorePose, MemoryCoreState, PuzzleLayout, PuzzlePieceId, PuzzlePiecePose},
};

use super::interpolation::{lerp, lerp_angle, lerp_vec3};

const TIMELINE_DURATION_SECONDS: f32 = 4.0;

#[derive(Clone, Copy, Debug)]
pub struct TimelineSample {
    pub memory_state: MemoryCoreState,
    pub core_pose: MemoryCorePose,
    pub puzzle: PuzzleLayout,
    pub light: Light,
}

#[derive(Clone, Copy)]
struct TimelineKeyframe {
    puzzle: PuzzleLayout,
    core: MemoryCorePose,
    light: Light,
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryTimeline {
    value: f32,
    playing: bool,
}

impl Default for MemoryTimeline {
    fn default() -> Self {
        Self {
            value: 0.0,
            playing: false,
        }
    }
}

impl MemoryTimeline {
    pub fn at(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            playing: false,
        }
    }

    pub fn restart(&mut self) {
        self.value = 0.0;
        self.playing = true;
    }

    pub fn toggle_playback(&mut self) {
        if self.value >= 1.0 {
            self.restart();
        } else {
            self.playing = !self.playing;
        }
    }

    pub fn update(&mut self, delta_seconds: f32) -> bool {
        if !self.playing || delta_seconds <= 0.0 {
            return false;
        }
        let previous = self.value;
        self.value = (self.value + delta_seconds / TIMELINE_DURATION_SECONDS).min(1.0);
        if self.value >= 1.0 {
            self.playing = false;
        }
        self.value != previous
    }

    pub const fn value(self) -> f32 {
        self.value
    }

    pub const fn is_playing(self) -> bool {
        self.playing
    }

    pub fn sample(self) -> TimelineSample {
        if self.value <= 0.5 {
            sample_segment(
                TimelineKeyframe {
                    puzzle: PuzzleLayout::preserved(),
                    core: MemoryCorePose {
                        offset: Vec3::new(0.0, 0.0, -0.40),
                        yaw: 0.0,
                    },
                    light: MemoryCoreState::Stable.light(),
                },
                TimelineKeyframe {
                    puzzle: PuzzleLayout::initial(),
                    core: MemoryCorePose {
                        offset: Vec3::new(0.0, 0.20, 0.0),
                        yaw: std::f32::consts::FRAC_PI_4,
                    },
                    light: MemoryCoreState::Fragmented.light(),
                },
                self.value * 2.0,
                if self.value < 0.35 {
                    MemoryCoreState::Stable
                } else {
                    MemoryCoreState::Fragmented
                },
            )
        } else {
            sample_segment(
                TimelineKeyframe {
                    puzzle: PuzzleLayout::initial(),
                    core: MemoryCorePose {
                        offset: Vec3::new(0.0, 0.20, 0.0),
                        yaw: std::f32::consts::FRAC_PI_4,
                    },
                    light: MemoryCoreState::Fragmented.light(),
                },
                TimelineKeyframe {
                    puzzle: PuzzleLayout::solved(),
                    core: MemoryCorePose::default(),
                    light: MemoryCoreState::Restored.light(),
                },
                (self.value - 0.5) * 2.0,
                if self.value < 0.90 {
                    MemoryCoreState::Fragmented
                } else {
                    MemoryCoreState::Restored
                },
            )
        }
    }
}

fn sample_segment(
    start: TimelineKeyframe,
    end: TimelineKeyframe,
    t: f32,
    memory_state: MemoryCoreState,
) -> TimelineSample {
    let mut poses = start.puzzle.poses;
    for piece in PuzzlePieceId::ALL {
        let index = piece.index();
        poses[index] = interpolate_pose(start.puzzle.poses[index], end.puzzle.poses[index], t);
    }
    TimelineSample {
        memory_state,
        core_pose: MemoryCorePose {
            offset: lerp_vec3(start.core.offset, end.core.offset, t),
            yaw: lerp_angle(start.core.yaw, end.core.yaw, t),
        },
        puzzle: PuzzleLayout {
            poses,
            selected: None,
        },
        light: Light::new(
            lerp_vec3(start.light.position, end.light.position, t),
            lerp_vec3(start.light.color, end.light.color, t),
            lerp(start.light.intensity, end.light.intensity, t),
        ),
    }
}

fn interpolate_pose(start: PuzzlePiecePose, end: PuzzlePiecePose, t: f32) -> PuzzlePiecePose {
    PuzzlePiecePose {
        position: lerp_vec3(start.position, end.position, t),
        yaw: lerp_angle(start.yaw, end.yaw, t),
    }
}

#[cfg(test)]
mod tests {
    use super::MemoryTimeline;
    use crate::world::{MemoryCoreState, PuzzleLayout};

    #[test]
    fn timeline_samples_preserved_fragmented_and_restored_keyframes() {
        let preserved = MemoryTimeline::at(0.0).sample();
        let fragmented = MemoryTimeline::at(0.5).sample();
        let restored = MemoryTimeline::at(1.0).sample();

        assert_eq!(preserved.memory_state, MemoryCoreState::Stable);
        assert_eq!(preserved.puzzle, PuzzleLayout::preserved());
        assert_eq!(fragmented.memory_state, MemoryCoreState::Fragmented);
        assert_eq!(fragmented.puzzle, PuzzleLayout::initial());
        assert_eq!(restored.memory_state, MemoryCoreState::Restored);
        assert_eq!(restored.puzzle, PuzzleLayout::solved());
    }

    #[test]
    fn playback_clamps_at_one_and_stops() {
        let mut timeline = MemoryTimeline::default();
        timeline.restart();
        assert!(timeline.update(10.0));
        assert_eq!(timeline.value(), 1.0);
        assert!(!timeline.is_playing());
    }
}
