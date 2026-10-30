use bevy::audio::Volume;
use bevy::prelude::*;

use crate::world::{CarState, LEVELS};

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    Title,
    InGame,
    GameOver,
    Victory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Normal,
    Cleared,
    Dying,
}

#[derive(Resource)]
pub struct RunState {
    pub level: usize,
    pub start_level: usize,
    pub score: u32,
    pub lives: u32,
    pub docs_collected: u32,
    pub docs_total: u32,
    pub escape_ready: bool,
    pub phase: Phase,
    pub phase_timer: f32,
    pub high_score: u32,
    pub muted: bool,
    pub banner: String,
    pub banner_timer: f32,
}

impl Default for RunState {
    fn default() -> Self {
        Self {
            level: 0,
            start_level: 0,
            score: 0,
            lives: 3,
            docs_collected: 0,
            docs_total: 0,
            escape_ready: false,
            phase: Phase::Normal,
            phase_timer: 0.0,
            high_score: 0,
            muted: false,
            banner: String::new(),
            banner_timer: 0.0,
        }
    }
}

impl RunState {
    pub fn reset(&mut self) {
        let start = self.start_level.min(LEVELS.len() - 1);
        let high = self.high_score;
        let muted = self.muted;
        *self = Self::default();
        self.level = start;
        self.start_level = start;
        self.high_score = high;
        self.muted = muted;
    }

    pub fn level_def(&self) -> &'static crate::world::LevelDef {
        &LEVELS[self.level.min(LEVELS.len() - 1)]
    }

    pub fn show_banner(&mut self, text: impl Into<String>, seconds: f32) {
        self.banner = text.into();
        self.banner_timer = seconds;
    }

    pub fn add_score(&mut self, points: u32) {
        self.score += points;
        self.high_score = self.high_score.max(self.score);
    }
}

#[derive(Resource)]
pub struct Elevators(pub [CarState; 2]);

impl Default for Elevators {
    fn default() -> Self {
        Self([CarState::new(1), CarState::new(4)])
    }
}

#[derive(Resource)]
pub struct Fx {
    pub shake: f32,
    pub flash: f32,
}

impl Default for Fx {
    fn default() -> Self {
        Self {
            shake: 0.0,
            flash: 0.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct Paused(pub bool);

#[derive(Resource)]
pub struct Rng(pub u64);

impl Default for Rng {
    fn default() -> Self {
        Self(0x1234_5678_9abc_def0)
    }
}

impl Rng {
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }

    pub fn unit(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.unit()
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.unit() < p
    }
}

#[derive(Resource)]
pub struct Art {
    pub font: Handle<Font>,
    pub player: Handle<Image>,
    pub player_layout: Handle<TextureAtlasLayout>,
    pub enemy_red: Handle<Image>,
    pub enemy_blue: Handle<Image>,
    pub enemy_layout: Handle<TextureAtlasLayout>,
    pub door_red_closed: Handle<Image>,
    pub door_red_open: Handle<Image>,
    pub door_gray_closed: Handle<Image>,
    pub door_gray_open: Handle<Image>,
    pub elevator: Handle<Image>,
    pub elevator_move: Handle<Image>,
    pub car: Handle<Image>,
    pub bullet_player: Handle<Image>,
    pub bullet_enemy: Handle<Image>,
    pub doc: Handle<Image>,
    pub life: Handle<Image>,
    pub exit_sign: Handle<Image>,
    pub logo: Handle<Image>,
    pub backgrounds: [Handle<Image>; 3],
}

impl Art {
    pub fn background(&self, index: usize) -> Handle<Image> {
        self.backgrounds[index.min(self.backgrounds.len() - 1)].clone()
    }

    pub fn enemy_sheet(&self, variant: u8) -> Handle<Image> {
        if variant.is_multiple_of(2) {
            self.enemy_red.clone()
        } else {
            self.enemy_blue.clone()
        }
    }

    pub fn door_sheet(&self, secret: bool, open: bool) -> Handle<Image> {
        match (secret, open) {
            (true, false) => self.door_red_closed.clone(),
            (true, true) => self.door_red_open.clone(),
            (false, false) => self.door_gray_closed.clone(),
            (false, true) => self.door_gray_open.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SfxKind {
    Shoot,
    EnemyShoot,
    Hit,
    Pickup,
    Death,
    Ding,
    Door,
    Call,
    LevelClear,
    GameOver,
    Victory,
}

#[derive(Message)]
pub struct PlaySfx(pub SfxKind);

#[derive(Resource)]
pub struct SoundAssets {
    pub shoot: Handle<AudioSource>,
    pub enemy_shoot: Handle<AudioSource>,
    pub hit: Handle<AudioSource>,
    pub pickup: Handle<AudioSource>,
    pub death: Handle<AudioSource>,
    pub ding: Handle<AudioSource>,
    pub door: Handle<AudioSource>,
    pub call: Handle<AudioSource>,
    pub level_clear: Handle<AudioSource>,
    pub game_over: Handle<AudioSource>,
    pub victory: Handle<AudioSource>,
    pub music: Handle<AudioSource>,
}

impl SoundAssets {
    pub fn get(&self, kind: SfxKind) -> Handle<AudioSource> {
        match kind {
            SfxKind::Shoot => self.shoot.clone(),
            SfxKind::EnemyShoot => self.enemy_shoot.clone(),
            SfxKind::Hit => self.hit.clone(),
            SfxKind::Pickup => self.pickup.clone(),
            SfxKind::Death => self.death.clone(),
            SfxKind::Ding => self.ding.clone(),
            SfxKind::Door => self.door.clone(),
            SfxKind::Call => self.call.clone(),
            SfxKind::LevelClear => self.level_clear.clone(),
            SfxKind::GameOver => self.game_over.clone(),
            SfxKind::Victory => self.victory.clone(),
        }
    }
}

#[derive(Resource, Default)]
pub struct PendingLevel(pub Option<usize>);

#[derive(Resource)]
pub struct CaptureConfig {
    pub screenshot: Option<String>,
    pub frame: u32,
    pub start: bool,
    pub level: usize,
    pub demo: bool,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            screenshot: None,
            frame: 240,
            start: false,
            level: 0,
            demo: false,
        }
    }
}

#[derive(Component)]
pub struct LevelEntity;

#[derive(Component)]
pub struct PixelPerfect;

#[derive(Component)]
pub struct HUDEntity;

#[derive(Component)]
pub struct Music;

#[derive(Component)]
pub struct TitleEntity;

#[derive(Resource, Default)]
pub struct Demo {
    pub enabled: bool,
    pub time: f32,
}

/// Marker for sprites that should be rounded to whole world pixels.
pub fn play_sfx_system(
    mut messages: MessageReader<PlaySfx>,
    sounds: Res<SoundAssets>,
    mut commands: Commands,
) {
    for msg in messages.read() {
        let base = match msg.0 {
            SfxKind::Shoot => 0.42,
            SfxKind::EnemyShoot => 0.30,
            SfxKind::Hit => 0.55,
            SfxKind::Pickup => 0.50,
            SfxKind::Death => 0.55,
            SfxKind::Ding => 0.40,
            SfxKind::Door => 0.35,
            SfxKind::Call => 0.35,
            SfxKind::LevelClear => 0.55,
            SfxKind::GameOver => 0.55,
            SfxKind::Victory => 0.55,
        };
        commands.spawn((
            AudioPlayer::new(sounds.get(msg.0)),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(base)),
        ));
    }
}
