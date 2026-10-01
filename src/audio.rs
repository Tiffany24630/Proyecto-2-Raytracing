use std::{collections::HashMap, io::Cursor, path::{Path, PathBuf}, sync::Arc,
    time::{Duration, Instant}};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use crate::game::{ExhibitionId, SceneState};

pub const BACKGROUND_MUSIC_ENV: &str = "TEMPLE_OF_SPACE_MUSIC";
pub const DEFAULT_BACKGROUND_MUSIC: &str = "assets/audio/music_world.mpeg";
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
    TaskComplete,
    Loading,
}

impl SoundEffect {
    const fn file_name(&self) -> &'static str {
        match self {
            Self::DoorOpen => "door_open.mpeg",
            Self::EnterDesert => "enter_miniature.mpeg",
            Self::EnterLibrary => "enter_miniature.mpeg",
            Self::EnterAcademy => "enter_miniature.mpeg",
            Self::MelantaAppear => "melanta_appear.mpeg",
            Self::PageCollect => "page_collect.mpeg",
            Self::PaintingMove => "painting_move.mpeg",
            Self::DesertLifeLost => "desert_life_lost.mpeg",
            Self::TaskComplete => "task_complete.mp3",
            Self::Loading => "enter_miniature.mpeg",
        }
    }
}

pub struct AudioDirector {
    stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    music: Option<Sink>,
    music_path: Option<PathBuf>,
    melanta: Option<Sink>,
    melanta_active: bool,
    melanta_looping: bool,
    requested_music: Option<&'static str>,
    music_retry_at: Instant,
    melanta_retry_at: Instant,
    clips: HashMap<PathBuf, Arc<[u8]>>,
    effects: Vec<Sink>,
}

impl AudioDirector {
    pub fn new() -> Self {
        match OutputStream::try_default() {
            Ok((stream, handle)) => Self { stream: Some(stream), handle: Some(handle), music: None,
                music_path: None, melanta: None, melanta_active: false, melanta_looping: false,
                requested_music: None, music_retry_at: Instant::now(), melanta_retry_at: Instant::now(),
                clips: HashMap::new(), effects: Vec::new() },
            Err(error) => {
                eprintln!("No se pudo abrir la salida de audio: {error}");
                Self { stream: None, handle: None, music: None,
                    music_path: None, melanta: None, melanta_active: false, melanta_looping: false,
                    requested_music: None, music_retry_at: Instant::now(), melanta_retry_at: Instant::now(),
                    clips: HashMap::new(), effects: Vec::new() }
            },
        }
    }

    pub fn play_effect(&mut self, effect: SoundEffect) {
        if self.handle.is_none() { return; }
        self.effects.retain(|sink| !sink.empty());
        if self.effects.len() >= 8 {
            self.effects.remove(0).stop();
        }
        let Some(path) = audio_path(effect.file_name()) else { return; };
        let Some(source) = self.decode(&path) else { return; };
        let Some(handle) = &self.handle else { return; };
        let Ok(sink) = Sink::try_new(handle) else { return; };
        sink.append(source);
        self.effects.push(sink);
    }

    pub fn play_music_for(&mut self, state: SceneState) {
        if state == SceneState::Entering { return; }
        let file_name = match state {
            SceneState::DesertPavilion => "music_desert.mpeg",
            SceneState::MahavaipulyaChamber => "music_library.mpeg",
            SceneState::LuyangAcademy => "music_academy.mpeg",
            SceneState::Final => "music_world.mpeg",
            _ => "music_world.mpeg",
        };
        if self.handle.is_none() { return; }
        let changed = self.requested_music != Some(file_name);
        if changed {
            self.requested_music = Some(file_name);
            if let Some(music) = self.music.take() { music.stop(); }
            self.music_path = None;
            self.stop_melanta();
            self.music_retry_at = Instant::now();
        }
        if self.music.as_ref().is_some_and(|music| !music.empty()) { return; }
        if Instant::now() < self.music_retry_at { return; }
        self.music_retry_at = Instant::now() + Duration::from_secs(3);
        let path = if file_name == "music_world.mpeg" {
            std::env::var_os(BACKGROUND_MUSIC_ENV).filter(|value| !value.is_empty())
                .map(PathBuf::from).filter(|path| path.is_file())
                .or_else(|| audio_path(file_name))
        } else { audio_path(file_name) };
        let Some(path) = path else {
            eprintln!("No se encontro la musica: {file_name}");
            return;
        };
        let Some(source) = self.decode(&path) else { return; };
        let Some(handle) = &self.handle else { return; };
        let sink = match Sink::try_new(handle) {
            Ok(sink) => sink,
            Err(error) => { eprintln!("No se pudo iniciar la musica: {error}"); return; }
        };
        sink.set_volume(if self.melanta_active { 0.10 } else { 0.65 });
        sink.append(source.repeat_infinite());
        if let Some(music) = self.music.take() { music.stop(); }
        self.music = Some(sink);
        self.music_path = Some(path);
    }

    pub fn set_melanta(&mut self, active: bool, looping: bool) {
        let appeared = active && !self.melanta_active;
        let mode_changed = self.melanta_looping != looping;
        self.melanta_active = active;
        if self.melanta_looping && !looping {
            if let Some(voice) = self.melanta.take() { voice.stop(); }
        }
        self.melanta_looping = looping;
        let playing = self.melanta.as_ref().is_some_and(|voice| !voice.empty());
        if let Some(music) = &self.music {
            music.set_volume(if active || playing { 0.10 } else { 0.65 });
        }
        if appeared || mode_changed { self.melanta_retry_at = Instant::now(); }
        if !active || (playing && !mode_changed)
            || Instant::now() < self.melanta_retry_at { return; }
        if !looping && !appeared && self.melanta.is_some() { return; }
        self.melanta_retry_at = Instant::now() + Duration::from_secs(3);
        if let Some(voice) = self.melanta.take() { voice.stop(); }
        let Some(path) = audio_path("melanta_appear.mpeg") else {
            eprintln!("No se encontro melanta_appear en assets/audio");
            return;
        };
        let Some(source) = self.decode(&path) else { return; };
        let Some(handle) = &self.handle else { return; };
        let voice = match Sink::try_new(handle) {
            Ok(voice) => voice,
            Err(error) => { eprintln!("No se pudo iniciar el audio de Melanta: {error}"); return; }
        };
        voice.set_volume(1.0);
        if looping { voice.append(source.repeat_infinite()); } else { voice.append(source); }
        self.melanta = Some(voice);
    }

    pub fn stop_melanta(&mut self) {
        if let Some(voice) = self.melanta.take() { voice.stop(); }
        self.melanta_active = false;
        self.melanta_looping = false;
        if let Some(music) = &self.music { music.set_volume(0.65); }
    }

    fn decode(&mut self, path: &Path) -> Option<Decoder<Cursor<Arc<[u8]>>>> {
        if !self.clips.contains_key(path) {
            let bytes = match std::fs::read(path) {
                Ok(bytes) => bytes,
                Err(error) => { eprintln!("No se pudo leer {}: {error}", path.display()); return None; }
            };
            let offset = audio_payload_offset(&bytes);
            self.clips.insert(path.to_path_buf(), Arc::from(&bytes[offset..]));
        }
        let bytes = Arc::clone(self.clips.get(path)?);
        let mp3 = bytes.len() >= 2 && bytes[0] == 0xff && bytes[1] & 0xe0 == 0xe0;
        let result = if mp3 { Decoder::new_mp3(Cursor::new(bytes)) }
            else { Decoder::new(Cursor::new(bytes)) };
        let mut source = match result {
            Ok(source) => source,
            Err(error) => { eprintln!("No se pudo decodificar {}: {error}", path.display()); return None; }
        };
        let channels = source.channels();
        let limit = source.sample_rate() as usize * 5;
        for _ in 0..limit {
            let mut audible = false;
            for _ in 0..channels {
                let Some(sample) = source.next() else {
                    eprintln!("Audio vacio o sin muestras audibles: {}", path.display());
                    return None;
                };
                audible |= (sample as f32).abs() > 32.0;
            }
            if audible { break; }
        }
        Some(source)
    }

    pub fn enter_exhibition(&mut self, exhibition: ExhibitionId) {
        self.play_effect(match exhibition {
            ExhibitionId::DesertPavilion => SoundEffect::EnterDesert,
            ExhibitionId::MahavaipulyaChamber => SoundEffect::EnterLibrary,
            ExhibitionId::LuyangAcademy => SoundEffect::EnterAcademy,
        });
    }
}

fn audio_path(file_name: &str) -> Option<PathBuf> {
    let mut directories = vec![PathBuf::from(AUDIO_DIRECTORY),
        Path::new(env!("CARGO_MANIFEST_DIR")).join(AUDIO_DIRECTORY)];
    if let Ok(executable) = std::env::current_exe() {
        directories.extend(executable.ancestors().skip(1).take(4)
            .map(|parent| parent.join(AUDIO_DIRECTORY)));
    }
    for directory in directories {
        let original = directory.join(file_name);
        for path in [original.clone(), original.with_extension("mp3"),
            original.with_extension("mpeg"), original.with_extension("ogg")] {
            if path.is_file() { return Some(path); }
        }
    }
    None
}

fn audio_payload_offset(bytes: &[u8]) -> usize {
    let mut offset = 0;
    while let Some(header) = bytes.get(offset..offset + 10) {
        if &header[..3] != b"ID3" || header[6..10].iter().any(|byte| byte & 0x80 != 0) { break; }
        let size = header[6..10].iter().fold(0usize, |size, byte| (size << 7) | *byte as usize);
        let footer = if header[3] == 4 && header[5] & 0x10 != 0 { 10 } else { 0 };
        let next = offset + 10 + size + footer;
        if next > bytes.len() { break; }
        offset = next;
    }
    offset
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
