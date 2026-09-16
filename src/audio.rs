use std::path::{Path, PathBuf};

pub const BACKGROUND_MUSIC_ENV: &str = "TEMPLE_OF_SPACE_MUSIC";
pub const DEFAULT_BACKGROUND_MUSIC: &str = "assets/audio/background.ogg";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AudioConfig {
    background_music: Option<PathBuf>,
}

impl AudioConfig {
    pub fn discover() -> Self {
        let configured = std::env::var_os(BACKGROUND_MUSIC_ENV)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from);
        let background_music = configured.or_else(|| {
            Path::new(DEFAULT_BACKGROUND_MUSIC)
                .is_file()
                .then(|| PathBuf::from(DEFAULT_BACKGROUND_MUSIC))
        });
        Self { background_music }
    }

    pub fn background_music(&self) -> Option<&Path> {
        self.background_music.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::AudioConfig;
    use std::path::PathBuf;

    #[test]
    fn optional_track_can_remain_unconfigured() {
        assert_eq!(AudioConfig::default().background_music(), None);
    }

    #[test]
    fn future_backend_receives_a_typed_track_path() {
        let config = AudioConfig {
            background_music: Some(PathBuf::from("music/theme.ogg")),
        };
        assert_eq!(
            config.background_music(),
            Some(std::path::Path::new("music/theme.ogg"))
        );
    }
}
