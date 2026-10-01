use std::{fs::File, io::BufReader, path::{Path, PathBuf}};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use crate::game::{ExhibitionId, SceneState};

pub const BACKGROUND_MUSIC_ENV: &str = "TEMPLE_OF_SPACE_MUSIC";
pub const DEFAULT_BACKGROUND_MUSIC: &str = "assets/audio/music_world.mp3";
pub const AUDIO_DIRECTORY: &str = "assets/audio";

pub enum SoundEffect {
    DoorOpen,
    EnterDesert,
    EnterLibrary,
    EnterAcademy,
    MelantaAppear,
    PageCollect,
    PaintingMove,
    DesertLifeLost,
}

impl SoundEffect {
    const fn file_name(&self) -> &'static str {
        match self {
            Self::DoorOpen => "door_open.mp3",
            Self::EnterDesert => "enter_desert.mp3",
            Self::EnterLibrary => "enter_library.mp3",
            Self::EnterAcademy => "enter_academy.mp3",
            Self::MelantaAppear => "melanta_appear.mp3",
            Self::PageCollect => "page_collect.mp3",
            Self::PaintingMove => "painting_move.mp3",
            Self::DesertLifeLost => "desert_life_lost.mp3",
        }
    }
}

pub struct AudioDirector {
    stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    music: Option<Sink>,
}

impl AudioDirector {
    pub fn new() -> Self {
        match OutputStream::try_default() {
            Ok((stream, handle)) => Self { stream: Some(stream), handle: Some(handle), music: None },
            Err(_) => Self { stream: None, handle: None, music: None },
        }
    }

    pub fn play_effect(&self, effect: SoundEffect) {
        let Some(handle) = &self.handle else { return; };
        let Ok(file) = File::open(Path::new(AUDIO_DIRECTORY).join(effect.file_name())) else { return; };
        let Ok(source) = Decoder::new(BufReader::new(file)) else { return; };
        let Ok(sink) = Sink::try_new(handle) else { return; };
        sink.append(source);
        sink.detach();
    }

    pub fn play_music_for(&mut self, state: SceneState) {
        let file_name = match state {
            SceneState::DesertPavilion => "music_desert.mp3",
            SceneState::MahavaipulyaChamber => "music_library.mp3",
            SceneState::LuyangAcademy => "music_academy.mp3",
            SceneState::Final => "music_nihilita.mp3",
            _ => "music_world.mp3",
        };
        let Some(handle) = &self.handle else { return; };
        if let Some(music) = self.music.take() { music.stop(); }
        let Ok(file) = File::open(Path::new(AUDIO_DIRECTORY).join(file_name)) else { return; };
        let Ok(source) = Decoder::new(BufReader::new(file)) else { return; };
        let Ok(sink) = Sink::try_new(handle) else { return; };
        sink.append(source.repeat_infinite());
        self.music = Some(sink);
    }

    pub fn enter_exhibition(&self, exhibition: ExhibitionId) {
        self.play_effect(match exhibition {
            ExhibitionId::DesertPavilion => SoundEffect::EnterDesert,
            ExhibitionId::MahavaipulyaChamber => SoundEffect::EnterLibrary,
            ExhibitionId::LuyangAcademy => SoundEffect::EnterAcademy,
        });
    }
}

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
