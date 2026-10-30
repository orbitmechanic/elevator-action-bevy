use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use elevator_action_bevy::combat::{Bullet, Door, collide_bullets, update_doors};
use elevator_action_bevy::level::{SpawnTimer, load_pending_level};
use elevator_action_bevy::player::Player;
use elevator_action_bevy::state::*;
use elevator_action_bevy::world::*;

fn test_art() -> Art {
    Art {
        font: Handle::default(),
        player: Handle::default(),
        player_layout: Handle::default(),
        enemy_red: Handle::default(),
        enemy_blue: Handle::default(),
        enemy_layout: Handle::default(),
        door_red_closed: Handle::default(),
        door_red_open: Handle::default(),
        door_gray_closed: Handle::default(),
        door_gray_open: Handle::default(),
        elevator: Handle::default(),
        elevator_move: Handle::default(),
        car: Handle::default(),
        bullet_player: Handle::default(),
        bullet_enemy: Handle::default(),
        doc: Handle::default(),
        life: Handle::default(),
        exit_sign: Handle::default(),
        logo: Handle::default(),
        backgrounds: [Handle::default(), Handle::default(), Handle::default()],
    }
}

fn test_player(floor: usize, x: f32) -> (Player, Sprite, Transform) {
    (
        Player {
            facing: 1.0,
            floor,
            ducking: false,
            vy: 0.0,
            airborne: false,
            fire_cd: 0.0,
            fire_anim: 0.0,
            invuln: 0.0,
            walk_phase: 0.0,
            riding: None,
            alive: true,
            dead_timer: 0.0,
        },
        Sprite::default(),
        Transform::from_xyz(x, FLOOR_Y[floor], 1.0),
    )
}

fn advance(app: &mut App, secs: f32) {
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f32(secs),
    ));
    app.update();
}

#[test]
fn secret_door_grants_file_and_escape_ready() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<PlaySfx>()
        .init_resource::<RunState>()
        .init_resource::<Fx>()
        .init_resource::<Elevators>()
        .insert_resource(test_art())
        .add_systems(Update, update_doors);
    {
        let mut run = app.world_mut().resource_mut::<RunState>();
        run.docs_total = 1;
    }
    app.world_mut().spawn((
        Door {
            floor: 2,
            x: 40.0,
            secret: true,
            opened: false,
            close_timer: 0.0,
        },
        Sprite::default(),
    ));
    app.world_mut().spawn(test_player(2, 40.0));

    advance(&mut app, 0.05);

    let run = app.world().resource::<RunState>();
    assert_eq!(run.docs_collected, 1);
    assert_eq!(run.score, 500);
    assert!(run.escape_ready);
    let mut doors = app.world_mut().query::<&Door>();
    assert!(doors.iter(app.world()).next().unwrap().opened);
}

#[test]
fn enemy_bullet_costs_a_life() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<PlaySfx>()
        .init_resource::<RunState>()
        .init_resource::<Fx>()
        .init_resource::<Elevators>()
        .add_systems(Update, collide_bullets);
    app.world_mut().spawn(test_player(1, 0.0));
    app.world_mut().spawn((
        Bullet {
            vx: 0.0,
            enemy: true,
            life: 1.0,
        },
        Sprite::default(),
        Transform::from_xyz(0.0, FLOOR_Y[1] + 10.0, 2.0),
    ));

    advance(&mut app, 0.05);

    let run = app.world().resource::<RunState>();
    assert_eq!(run.lives, 2);
    assert_eq!(run.phase, Phase::Dying);
    let mut players = app.world_mut().query::<&Player>();
    assert!(!players.iter(app.world()).next().unwrap().alive);
}

#[test]
fn cleared_level_advances_to_next_assignment() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<AppState>()
        .add_message::<PlaySfx>()
        .init_resource::<RunState>()
        .init_resource::<Elevators>()
        .init_resource::<PendingLevel>()
        .add_systems(Update, elevator_action_bevy::tick_phase);
    {
        let mut run = app.world_mut().resource_mut::<RunState>();
        run.phase = Phase::Cleared;
        run.phase_timer = -1.0;
    }
    advance(&mut app, 0.1);

    let run = app.world().resource::<RunState>();
    assert_eq!(run.level, 1);
    assert_eq!(run.phase, Phase::Normal);
    assert_eq!(app.world().resource::<PendingLevel>().0, Some(1));
}

#[test]
fn level_loader_builds_a_full_stage() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<RunState>()
        .init_resource::<Elevators>()
        .init_resource::<Rng>()
        .init_resource::<SpawnTimer>()
        .insert_resource(test_art())
        .insert_resource(PendingLevel(Some(0)))
        .add_systems(Update, load_pending_level);

    app.update();

    {
        let run = app.world().resource::<RunState>();
        assert_eq!(run.level, 0);
        assert_eq!(run.docs_total, 6);
        assert_eq!(run.docs_collected, 0);
    }
    let mut players = app.world_mut().query::<&Player>();
    assert_eq!(players.iter(app.world()).count(), 1);
    let mut doors = app.world_mut().query::<&Door>();
    assert_eq!(doors.iter(app.world()).count(), LEVEL_1.doors.len());
    let mut elevators = app
        .world_mut()
        .query::<&elevator_action_bevy::elevator::ElevatorCar>();
    assert_eq!(elevators.iter(app.world()).count(), 2);

    app.world_mut().resource_mut::<PendingLevel>().0 = Some(1);
    app.update();
    let run = app.world().resource::<RunState>();
    assert_eq!(run.level, 1);
    assert_eq!(run.docs_total, 7);
}

#[test]
fn final_clear_moves_to_victory() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<AppState>()
        .add_message::<PlaySfx>()
        .init_resource::<RunState>()
        .init_resource::<Elevators>()
        .init_resource::<PendingLevel>()
        .add_systems(Update, elevator_action_bevy::tick_phase);
    {
        let mut run = app.world_mut().resource_mut::<RunState>();
        run.level = 2;
        run.phase = Phase::Cleared;
        run.phase_timer = -1.0;
    }
    advance(&mut app, 0.1);
    let next = app.world().resource::<NextState<AppState>>();
    assert!(matches!(next, NextState::Pending(AppState::Victory)));
}

#[test]
fn death_respawns_on_floor_one() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<AppState>()
        .add_message::<PlaySfx>()
        .init_resource::<RunState>()
        .init_resource::<Elevators>()
        .init_resource::<PendingLevel>()
        .add_systems(Update, elevator_action_bevy::tick_phase);
    {
        let mut run = app.world_mut().resource_mut::<RunState>();
        run.phase = Phase::Dying;
        run.phase_timer = -1.0;
        run.lives = 2;
    }
    let mut player = test_player(3, 0.0);
    player.0.alive = false;
    app.world_mut().spawn(player);

    advance(&mut app, 0.1);

    let run = app.world().resource::<RunState>();
    assert_eq!(run.phase, Phase::Normal);
    let mut players = app.world_mut().query::<(&Player, &Transform)>();
    let (player, transform) = players.iter(app.world()).next().unwrap();
    assert!(player.alive);
    assert_eq!(player.floor, 1);
    assert!((transform.translation.x - SPAWN_X).abs() < 0.01);
}
