pub mod combat;
pub mod elevator;
pub mod enemy;
pub mod level;
pub mod player;
pub mod state;
pub mod ui;
pub mod world;

use bevy::{
    audio::{GlobalVolume, Volume},
    camera::ScalingMode,
    image::ImagePlugin,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
    ui_render::prelude::UiAntiAlias,
    window::WindowResolution,
};

use combat::*;
use elevator::*;
use enemy::*;
use level::*;
use player::*;
use state::*;
use world::*;

pub fn run() {
    let capture = parse_args();
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "ELEVATOR ACTION: RED DOOR PROTOCOL".into(),
                    resolution: WindowResolution::new(960, 720),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
    )
    .insert_resource(ClearColor(Color::srgb(0.012, 0.018, 0.03)))
    .insert_resource(capture)
    .insert_resource(GlobalVolume::new(Volume::Linear(0.85)))
    .init_resource::<RunState>()
    .init_resource::<Elevators>()
    .init_resource::<Fx>()
    .init_resource::<Paused>()
    .init_resource::<Rng>()
    .init_resource::<PendingLevel>()
    .init_resource::<SpawnTimer>()
    .init_resource::<Demo>()
    .init_state::<AppState>()
    .add_message::<PlaySfx>()
    .add_systems(PreStartup, (load_art, load_sounds))
    .add_systems(Startup, (setup_camera, start_music, apply_capture_config))
    .add_systems(OnEnter(AppState::InGame), begin_run)
    .add_systems(OnEnter(AppState::GameOver), spawn_game_over)
    .add_systems(OnEnter(AppState::Victory), spawn_victory)
    .add_systems(
        Update,
        (
            demo_input,
            global_input,
            play_sfx_system,
            tick_phase
                .run_if(in_state(AppState::InGame))
                .run_if(not_paused),
            load_pending_level.run_if(in_state(AppState::InGame)),
            (
                update_elevators,
                update_player,
                update_enemies,
                spawn_wave,
                update_doors,
                update_bullets,
                collide_bullets,
                check_exit,
                update_particles,
            )
                .chain()
                .run_if(in_state(AppState::InGame))
                .run_if(game_running),
            sync_elevators.run_if(in_state(AppState::InGame)),
            update_exit_sign.run_if(in_state(AppState::InGame)),
            ui::update_hud.run_if(in_state(AppState::InGame)),
            ui::update_title.run_if(in_state(AppState::Title)),
            title_setup.run_if(in_state(AppState::Title)),
            snap_pixels.run_if(in_state(AppState::InGame)),
            decay_fx,
            camera_shake,
            capture_system,
        )
            .chain(),
    )
    .run();
}

fn not_paused(paused: Res<Paused>) -> bool {
    !paused.0
}

fn game_running(run: Res<RunState>, paused: Res<Paused>) -> bool {
    run.phase == Phase::Normal && !paused.0
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: WORLD_H,
            },
            ..OrthographicProjection::default_2d()
        }),
        Msaa::Off,
        UiAntiAlias::Off,
        Transform::from_xyz(0.0, 0.0, 0.0),
        Name::new("Main Camera"),
    ));
}

fn load_art(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let player_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 24),
        7,
        1,
        None,
        None,
    ));
    let enemy_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 24),
        5,
        1,
        None,
        None,
    ));
    commands.insert_resource(Art {
        font: asset_server.load("fonts/Arcade17.ttf"),
        player: asset_server.load("sprites/player.png"),
        player_layout,
        enemy_red: asset_server.load("sprites/enemy_red.png"),
        enemy_blue: asset_server.load("sprites/enemy_blue.png"),
        enemy_layout,
        door_red_closed: asset_server.load("sprites/door_red_closed.png"),
        door_red_open: asset_server.load("sprites/door_red_open.png"),
        door_gray_closed: asset_server.load("sprites/door_gray_closed.png"),
        door_gray_open: asset_server.load("sprites/door_gray_open.png"),
        elevator: asset_server.load("sprites/elevator.png"),
        elevator_move: asset_server.load("sprites/elevator_move.png"),
        car: asset_server.load("sprites/car.png"),
        bullet_player: asset_server.load("sprites/bullet_player.png"),
        bullet_enemy: asset_server.load("sprites/bullet_enemy.png"),
        doc: asset_server.load("sprites/doc.png"),
        life: asset_server.load("sprites/life.png"),
        exit_sign: asset_server.load("sprites/exit_sign.png"),
        logo: asset_server.load("sprites/logo.png"),
        backgrounds: [
            asset_server.load("backgrounds/bg_midnight.png"),
            asset_server.load("backgrounds/bg_amber.png"),
            asset_server.load("backgrounds/bg_violet.png"),
        ],
    });
}

fn load_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SoundAssets {
        shoot: asset_server.load("audio/shoot.wav"),
        enemy_shoot: asset_server.load("audio/enemy_shoot.wav"),
        hit: asset_server.load("audio/hit.wav"),
        pickup: asset_server.load("audio/pickup.wav"),
        death: asset_server.load("audio/death.wav"),
        ding: asset_server.load("audio/ding.wav"),
        door: asset_server.load("audio/door.wav"),
        call: asset_server.load("audio/call.wav"),
        level_clear: asset_server.load("audio/level_clear.wav"),
        game_over: asset_server.load("audio/game_over.wav"),
        victory: asset_server.load("audio/victory.wav"),
        music: asset_server.load("audio/music.wav"),
    });
}

fn start_music(sounds: Res<SoundAssets>, mut commands: Commands) {
    commands.spawn((
        AudioPlayer::new(sounds.music.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.28)),
        Music,
    ));
}

fn apply_capture_config(
    capture: Res<CaptureConfig>,
    mut run: ResMut<RunState>,
    mut next: ResMut<NextState<AppState>>,
    mut demo: ResMut<Demo>,
) {
    demo.enabled = capture.demo;
    run.start_level = capture.level.min(LEVELS.len() - 1);
    run.level = run.start_level;
    if capture.start || capture.demo {
        next.set(AppState::InGame);
    }
}

fn demo_input(
    time: Res<Time>,
    state: Res<State<AppState>>,
    mut demo: ResMut<Demo>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    players: Query<(&Player, &Transform)>,
    elevators: Res<Elevators>,
) {
    if !demo.enabled || *state.get() != AppState::InGame {
        return;
    }
    let prev = demo.time;
    let next = prev + time.delta_secs().min(0.05);
    demo.time = next;
    keys.reset_all();
    let hit = |at: f32| prev < at && next >= at;
    if prev as i32 != next as i32
        && let Ok((player, tf)) = players.single()
    {
        info!(
            "demo t={:.1} floor={} x={:.1} y={:.1} riding={:?} cars={:?}",
            next,
            player.floor,
            tf.translation.x,
            tf.translation.y,
            player.riding,
            elevators
                .0
                .iter()
                .map(|c| (c.bottom, c.moving))
                .collect::<Vec<_>>()
        );
    }
    if (0.0..0.35).contains(&prev) {
        keys.press(KeyCode::KeyD);
    }
    if hit(0.50) {
        keys.press(KeyCode::KeyW);
    }
    if hit(1.20) {
        keys.press(KeyCode::KeyS);
    }
    if (1.95..3.20).contains(&prev) {
        keys.press(KeyCode::KeyD);
    }
    if hit(3.00) {
        keys.press(KeyCode::KeyJ);
    }
    if (3.40..4.85).contains(&prev) {
        keys.press(KeyCode::KeyA);
    }
    if hit(4.90) {
        keys.press(KeyCode::KeyW);
    }
    if hit(5.40) {
        keys.press(KeyCode::KeyS);
    }
    if (6.20..7.40).contains(&prev) {
        keys.press(KeyCode::KeyD);
    }
    if hit(6.60) || hit(7.00) || hit(8.00) {
        keys.press(KeyCode::KeyJ);
    }
}

fn title_setup(
    art: Option<Res<Art>>,
    existing: Query<(), With<TitleEntity>>,
    mut commands: Commands,
) {
    if let Some(art) = art
        && existing.is_empty()
    {
        ui::spawn_title(&mut commands, &art);
    }
}

fn begin_run(
    mut run: ResMut<RunState>,
    mut paused: ResMut<Paused>,
    mut pending: ResMut<PendingLevel>,
) {
    run.reset();
    paused.0 = false;
    pending.0 = Some(run.level);
}

fn spawn_game_over(mut commands: Commands, art: Res<Art>, run: Res<RunState>) {
    ui::spawn_end_screen(&mut commands, &art, &run, false);
}

fn spawn_victory(mut commands: Commands, art: Res<Art>, run: Res<RunState>) {
    ui::spawn_end_screen(&mut commands, &art, &run, true);
}

fn global_input(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut run: ResMut<RunState>,
    mut paused: ResMut<Paused>,
    mut pending: ResMut<PendingLevel>,
    mut next: ResMut<NextState<AppState>>,
    mut volume: ResMut<GlobalVolume>,
) {
    if keys.just_pressed(KeyCode::KeyM) {
        run.muted = !run.muted;
        volume.volume = if run.muted {
            Volume::Linear(0.0)
        } else {
            Volume::Linear(0.85)
        };
    }

    match state.get() {
        AppState::Title => {
            if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
                run.reset();
                paused.0 = false;
                pending.0 = Some(run.level);
                next.set(AppState::InGame);
            }
        }
        AppState::InGame => {
            if keys.just_pressed(KeyCode::KeyP) || keys.just_pressed(KeyCode::Escape) {
                paused.0 = !paused.0;
            }
        }
        AppState::GameOver | AppState::Victory => {
            if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
                run.reset();
                paused.0 = false;
                pending.0 = Some(run.level);
                next.set(AppState::InGame);
            } else if keys.just_pressed(KeyCode::Escape) {
                next.set(AppState::Title);
            }
        }
    }
}

pub fn tick_phase(
    time: Res<Time>,
    mut run: ResMut<RunState>,
    elevators: ResMut<Elevators>,
    mut next: ResMut<NextState<AppState>>,
    mut pending: ResMut<PendingLevel>,
    mut sfx: MessageWriter<PlaySfx>,
    players: Query<(&mut Player, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs().min(0.05);
    run.banner_timer = (run.banner_timer - dt).max(0.0);
    if run.banner_timer <= 0.0 {
        run.banner.clear();
    }

    match run.phase {
        Phase::Dying => {
            run.phase_timer -= dt;
            if run.phase_timer <= 0.0 {
                if run.lives > 0 {
                    respawn_player(players, elevators);
                    run.phase = Phase::Normal;
                    run.phase_timer = 0.0;
                    run.show_banner("REINFORCEMENTS INBOUND", 1.6);
                } else {
                    run.high_score = run.high_score.max(run.score);
                    run.phase_timer = 0.0;
                    sfx.write(PlaySfx(SfxKind::GameOver));
                    next.set(AppState::GameOver);
                }
            }
        }
        Phase::Cleared => {
            run.phase_timer -= dt;
            if run.phase_timer <= 0.0 {
                if run.level + 1 < LEVELS.len() {
                    run.level += 1;
                    run.phase = Phase::Normal;
                    pending.0 = Some(run.level);
                } else {
                    run.high_score = run.high_score.max(run.score);
                    sfx.write(PlaySfx(SfxKind::Victory));
                    next.set(AppState::Victory);
                }
            }
        }
        Phase::Normal => {}
    }
}

fn check_exit(
    mut run: ResMut<RunState>,
    mut fx: ResMut<Fx>,
    mut sfx: MessageWriter<PlaySfx>,
    mut commands: Commands,
    players: Query<(&Player, &Transform)>,
) {
    if run.phase != Phase::Normal || !run.escape_ready {
        return;
    }
    let Ok((player, tf)) = players.single() else {
        return;
    };
    if player.floor == 7 && tf.translation.x > EXIT_X {
        run.phase = Phase::Cleared;
        run.phase_timer = 2.6;
        let bonus = 1000 + run.lives * 250;
        run.add_score(bonus);
        run.show_banner(format!("EXTRACTION COMPLETE  +{}", bonus), 2.6);
        fx.flash = 0.45;
        fx.shake = 3.0;
        sfx.write(PlaySfx(SfxKind::LevelClear));
        combat::spawn_burst(
            &mut commands,
            Vec2::new(110.0, -100.0),
            Color::srgb(0.4, 1.0, 0.55),
            20,
            60.0,
        );
    }
}

fn decay_fx(time: Res<Time>, mut fx: ResMut<Fx>) {
    let dt = time.delta_secs().min(0.05);
    fx.shake = (fx.shake - dt * 13.0).max(0.0);
    fx.flash = (fx.flash - dt * 2.3).max(0.0);
}

fn camera_shake(
    fx: Res<Fx>,
    mut rng: ResMut<Rng>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };
    if fx.shake > 0.05 {
        transform.translation.x = rng.range(-fx.shake, fx.shake) * 0.45;
        transform.translation.y = rng.range(-fx.shake, fx.shake) * 0.45;
    } else {
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
    }
}

fn capture_system(
    capture: Res<CaptureConfig>,
    mut frame: Local<u32>,
    mut fired: Local<bool>,
    mut commands: Commands,
) {
    if capture.screenshot.is_none() || *fired {
        return;
    }
    *frame += 1;
    if *frame < capture.frame {
        return;
    }
    *fired = true;
    let Some(path) = capture.screenshot.clone() else {
        return;
    };
    info!("capturing {} at frame {}", path, *frame);
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path))
        .observe(
            move |_: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                exit.write(AppExit::Success);
            },
        );
}

fn parse_args() -> CaptureConfig {
    let mut cfg = CaptureConfig::default();
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--screenshot" => {
                i += 1;
                if i < args.len() {
                    cfg.screenshot = Some(args[i].clone());
                }
            }
            "--frame" => {
                i += 1;
                if i < args.len() {
                    cfg.frame = args[i].parse().unwrap_or(240);
                }
            }
            "--start" => cfg.start = true,
            "--demo" => {
                cfg.demo = true;
                cfg.start = true;
            }
            "--level" => {
                i += 1;
                if i < args.len() {
                    let n: usize = args[i].parse().unwrap_or(1);
                    cfg.level = n.saturating_sub(1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    cfg
}
