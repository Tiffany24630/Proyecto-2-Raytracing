use std::{fs::File, io::BufReader, path::{Path, PathBuf}};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use crate::game::{ExhibitionId, SceneState};

pub const BACKGROUND_MUSIC_ENV: &str = "TEMPLE_OF_SPACE_MUSIC";
pub const DEFAULT_BACKGROUND_MUSIC: &str = "assets/audio/background.ogg";
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
            Self::DoorOpen => "door_open.ogg",
            Self::EnterDesert => "enter_desert.ogg",
            Self::EnterLibrary => "enter_library.ogg",
            Self::EnterAcademy => "enter_academy.ogg",
            Self::MelantaAppear => "melanta_appear.ogg",
            Self::PageCollect => "page_collect.ogg",
            Self::PaintingMove => "painting_move.ogg",
            Self::DesertLifeLost => "desert_life_lost.ogg",
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
            SceneState::DesertPavilion => "music_desert.ogg",
            SceneState::MahavaipulyaChamber => "music_library.ogg",
            SceneState::LuyangAcademy => "music_academy.ogg",
            SceneState::Final => "music_nihilita.ogg",
            _ => "music_world.ogg",
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
