#[derive(Clone, Copy, Debug)]
pub struct SceneTransition {
    progress: f32,
    duration_seconds: f32,
    playing: bool,
}

impl SceneTransition {
    pub fn new(duration_seconds: f32) -> Self {
        assert!(duration_seconds > 0.0);
        Self {
            progress: 1.0,
            duration_seconds,
            playing: false,
        }
    }

    pub fn at(duration_seconds: f32, progress: f32) -> Self {
        assert!(duration_seconds > 0.0);
        Self {
            progress: progress.clamp(0.0, 1.0),
            duration_seconds,
            playing: false,
        }
    }

    pub fn restart(&mut self) {
        self.progress = 0.0;
        self.playing = true;
    }

    pub fn update(&mut self, delta_seconds: f32) -> bool {
        if !self.playing || delta_seconds <= 0.0 {
            return false;
        }
        let previous = self.progress;
        self.progress = (self.progress + delta_seconds / self.duration_seconds).clamp(0.0, 1.0);
        if self.progress >= 1.0 {
            self.playing = false;
        }
        self.progress != previous
    }

    pub const fn progress(self) -> f32 {
        self.progress
    }

    pub fn eased_progress(self) -> f32 {
        smoothstep(self.progress)
    }

    pub fn fade_opacity(self) -> f32 {
        (1.0 - (2.0 * self.progress - 1.0).abs()) * 0.82
    }

    pub const fn is_finished(self) -> bool {
        self.progress >= 1.0
    }
}

fn smoothstep(value: f32) -> f32 {
    let t = value.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::SceneTransition;

    #[test]
    fn transition_advances_clamps_and_fades_at_the_midpoint() {
        let mut transition = SceneTransition::new(2.0);
        transition.restart();
        assert!(transition.update(1.0));
        assert_eq!(transition.progress(), 0.5);
        assert!((transition.fade_opacity() - 0.82).abs() < f32::EPSILON);
        assert!(transition.update(2.0));
        assert!(transition.is_finished());
        assert_eq!(transition.fade_opacity(), 0.0);
    }

    #[test]
    fn easing_preserves_endpoints() {
        assert_eq!(SceneTransition::at(1.0, 0.0).eased_progress(), 0.0);
        assert_eq!(SceneTransition::at(1.0, 1.0).eased_progress(), 1.0);
    }
}
