use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::player::{Player, shaft_near};
use crate::state::*;
use crate::world::*;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum HudField {
    Score,
    Level,
    Files,
    Hint,
    Banner,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum OverlayKind {
    Banner,
    Clear,
    Pause,
}

#[derive(Component)]
pub struct LifeIcon(pub u32);

#[derive(Component)]
pub struct FlashOverlay;

#[derive(Component)]
pub struct TitlePrompt;

#[derive(Component)]
pub struct TitleCar {
    pub phase: f32,
}

fn fonted(art: &Art, content: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(content.into()),
        TextFont {
            font: art.font.clone().into(),
            font_size: FontSize::Px(size),
            font_smoothing: FontSmoothing::None,
            ..default()
        },
        TextColor(color),
        TextShadow {
            offset: Vec2::new(2.0, 2.0),
            color: Color::srgba(0.0, 0.0, 0.0, 0.8),
        },
    )
}

pub fn spawn_hud(commands: &mut Commands, art: &Art) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(54.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect {
                    left: Val::Px(20.0),
                    right: Val::Px(20.0),
                    ..default()
                },
                ..default()
            },
            BackgroundColor(Color::srgba(0.015, 0.025, 0.04, 0.88)),
            HUDEntity,
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|bar| {
            bar.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn(fonted(art, "SCORE", 20.0, Color::srgb(0.35, 0.85, 0.82)));
                row.spawn((fonted(art, "000000", 30.0, Color::WHITE), HudField::Score));
            });

            bar.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn(fonted(art, "STAGE", 20.0, Color::srgb(0.35, 0.85, 0.82)));
                row.spawn((
                    fonted(art, "1/3", 30.0, Color::srgb(0.65, 0.92, 1.0)),
                    HudField::Level,
                ));
            });

            bar.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn(fonted(art, "FILES", 20.0, Color::srgb(0.35, 0.85, 0.82)));
                row.spawn((
                    fonted(art, "0/6", 30.0, Color::srgb(1.0, 0.83, 0.38)),
                    HudField::Files,
                ));
            });

            bar.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(5.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn(fonted(art, "LIVES", 20.0, Color::srgb(0.35, 0.85, 0.82)));
                for i in 0..3 {
                    row.spawn((
                        ImageNode::new(art.life.clone()),
                        Node {
                            width: Val::Px(24.0),
                            height: Val::Px(30.0),
                            ..default()
                        },
                        LifeIcon(i),
                    ));
                }
            });
        });

    // Thin accent rule.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(54.0),
            width: Val::Percent(100.0),
            height: Val::Px(2.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.85, 0.16, 0.2, 0.9)),
        HUDEntity,
        DespawnOnExit(AppState::InGame),
    ));

    // Bottom hint bar.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(30.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.015, 0.025, 0.04, 0.84)),
            HUDEntity,
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|bar| {
            bar.spawn((
                fonted(
                    art,
                    "A/D MOVE   J SHOOT   K JUMP   W RIDE   P PAUSE   M MUTE",
                    20.0,
                    Color::srgb(0.66, 0.71, 0.8),
                ),
                HudField::Hint,
            ));
        });

    // Banner text.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(68.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Visibility::Hidden,
            OverlayKind::Banner,
            HUDEntity,
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|root| {
            root.spawn(fonted(art, "", 30.0, Color::srgb(1.0, 0.85, 0.42)));
        });

    // Clear overlay.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.02, 0.035, 0.66)),
            Visibility::Hidden,
            OverlayKind::Clear,
            HUDEntity,
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|root| {
            root.spawn(fonted(
                art,
                "FLOOR CLEARED",
                60.0,
                Color::srgb(1.0, 0.84, 0.38),
            ));
            root.spawn(fonted(art, "NEXT ASSIGNMENT INCOMING", 30.0, Color::WHITE));
        });

    // Pause overlay.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.02, 0.035, 0.72)),
            Visibility::Hidden,
            OverlayKind::Pause,
            HUDEntity,
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|root| {
            root.spawn(fonted(art, "PAUSED", 60.0, Color::srgb(0.45, 0.9, 0.92)));
            root.spawn(fonted(art, "P TO RESUME", 30.0, Color::WHITE));
        });

    // Damage flash.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 0.9, 0.8, 0.0)),
        FlashOverlay,
        HUDEntity,
        DespawnOnExit(AppState::InGame),
    ));
}

#[allow(clippy::too_many_arguments)]
pub fn update_hud(
    run: Res<RunState>,
    paused: Res<Paused>,
    fx: Res<Fx>,
    player_query: Query<(&Player, &Transform)>,
    elevators: Res<Elevators>,
    mut fields: Query<(&HudField, &mut Text)>,
    mut overlays: Query<(&mut Visibility, Option<&OverlayKind>, Option<&LifeIcon>)>,
    mut flash: Query<&mut BackgroundColor, With<FlashOverlay>>,
) {
    let player = player_query.single().ok();
    let mut hint = String::from("A/D MOVE   J SHOOT   K JUMP   W RIDE   P PAUSE   M MUTE");
    if let Some((player, tf)) = player {
        if run.phase == Phase::Cleared {
            hint = "EXTRACTION COMPLETE".to_string();
        } else if run.escape_ready {
            hint = "ALL FILES SECURED - GET TO THE CAR, BOTTOM RIGHT".to_string();
        } else if let Some(shaft) = player.riding {
            let car = &elevators.0[shaft];
            hint = if car.docked_floor().is_some() {
                "W/S  PICK A FLOOR      A/D  STEP OFF".to_string()
            } else {
                format!(
                    "ELEVATOR MOVING - NEXT STOP {:02}",
                    7usize.saturating_sub(car.current_floor())
                )
            };
        } else if let Some(shaft) = shaft_near(tf.translation.x, SHAFT_HALF) {
            hint = if elevators.0[shaft].docked_floor() == Some(player.floor) {
                "W  BOARD ELEVATOR".to_string()
            } else {
                "W  CALL ELEVATOR".to_string()
            };
        } else if player.floor == 7 && tf.translation.x > EXIT_X - 20.0 {
            hint = "THE GETAWAY CAR NEEDS EVERY RED FILE".to_string();
        }
    }
    if paused.0 {
        hint = "PAUSED - P TO RESUME".to_string();
    }
    let mute = if run.muted { "   [MUTED]" } else { "" };
    hint.push_str(mute);

    for (field, mut text) in &mut fields {
        text.0 = match field {
            HudField::Score => format!("{:06}", run.score),
            HudField::Level => format!("{}/{}", run.level + 1, LEVELS.len()),
            HudField::Files => format!("{}/{}", run.docs_collected, run.docs_total),
            HudField::Hint => hint.clone(),
            HudField::Banner => run.banner.clone(),
        };
    }

    for (mut visibility, kind, life) in &mut overlays {
        if let Some(kind) = kind {
            *visibility = match kind {
                OverlayKind::Banner => {
                    if run.banner_timer > 0.0 && !paused.0 && run.phase != Phase::Cleared {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    }
                }
                OverlayKind::Clear => {
                    if run.phase == Phase::Cleared && !paused.0 {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    }
                }
                OverlayKind::Pause => {
                    if paused.0 {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    }
                }
            };
        } else if let Some(life) = life {
            *visibility = if life.0 < run.lives {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }

    for mut bg in &mut flash {
        let a = (fx.flash * 0.55).clamp(0.0, 0.55);
        bg.0 = Color::srgba(1.0, 0.92, 0.82, a);
    }
}

pub fn spawn_title(commands: &mut Commands, art: &Art) {
    commands.spawn((
        Sprite {
            image: art.background(0),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -20.0),
        TitleEntity,
        DespawnOnExit(AppState::Title),
    ));

    for (i, phase) in [0.0f32, 2.1].iter().enumerate() {
        commands.spawn((
            Sprite {
                image: art.elevator.clone(),
                ..default()
            },
            Anchor::BOTTOM_CENTER,
            Transform::from_xyz(SHAFT_X[i], -60.0, -7.0),
            TitleCar { phase: *phase },
            PixelPerfect,
            TitleEntity,
            DespawnOnExit(AppState::Title),
        ));
    }

    commands.spawn((
        Sprite {
            image: art.logo.clone(),
            ..default()
        },
        Transform::from_xyz(0.0, 64.0, 5.0),
        TitleEntity,
        DespawnOnExit(AppState::Title),
    ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.02, 0.04, 0.38)),
            TitleEntity,
            DespawnOnExit(AppState::Title),
        ))
        .with_children(|_| {});

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(388.0),
                width: Val::Percent(100.0),
                height: Val::Px(296.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(11.0),
                padding: UiRect::vertical(Val::Px(18.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.02, 0.04, 0.62)),
            TitleEntity,
            DespawnOnExit(AppState::Title),
        ))
        .with_children(|root| {
            root.spawn((
                fonted(
                    art,
                    "PRESS ENTER TO START",
                    40.0,
                    Color::srgb(1.0, 0.84, 0.38),
                ),
                TitlePrompt,
            ));
            root.spawn(fonted(
                art,
                "A/D OR ARROWS  MOVE        J / X  SHOOT        K / SPACE  JUMP",
                20.0,
                Color::WHITE,
            ));
            root.spawn(fonted(
                art,
                "W / S  CALL & RIDE ELEVATORS      P  PAUSE      M  MUTE",
                20.0,
                Color::WHITE,
            ));
            root.spawn(fonted(
                art,
                "COLLECT EVERY RED FILE, THEN REACH THE GETAWAY CAR",
                20.0,
                Color::srgb(0.45, 0.9, 0.92),
            ));
            root.spawn(fonted(
                art,
                "THREE ASSIGNMENTS. ONE AGENT. NO WITNESSES.",
                20.0,
                Color::srgb(0.72, 0.48, 0.52),
            ));
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(12.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            TitleEntity,
            DespawnOnExit(AppState::Title),
        ))
        .with_children(|root| {
            root.spawn(fonted(
                art,
                "A BEVY 0.20 ARCADE TRIBUTE",
                20.0,
                Color::srgb(0.48, 0.54, 0.64),
            ));
        });
}

pub fn update_title(
    time: Res<Time>,
    mut cars: Query<(&TitleCar, &mut Transform)>,
    mut prompts: Query<&mut TextColor, With<TitlePrompt>>,
) {
    let t = time.elapsed_secs();
    for (car, mut transform) in &mut cars {
        let wave = ((t * 0.55 + car.phase).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        transform.translation.y = -92.0 + wave * 182.0;
    }
    let alpha = 0.55 + ((t * 3.2).sin() * 0.45);
    for mut color in &mut prompts {
        color.0 = Color::srgba(1.0, 0.84, 0.38, alpha);
    }
}

pub fn spawn_end_screen(commands: &mut Commands, art: &Art, run: &RunState, victory: bool) {
    commands.spawn((
        Sprite {
            image: art.background(run.level),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -20.0),
        DespawnOnExit(if victory {
            AppState::Victory
        } else {
            AppState::GameOver
        }),
    ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.02, 0.04, 0.78)),
            DespawnOnExit(if victory {
                AppState::Victory
            } else {
                AppState::GameOver
            }),
        ))
        .with_children(|root| {
            root.spawn((
                ImageNode::new(art.logo.clone()),
                Node {
                    width: Val::Px(792.0),
                    height: Val::Px(288.0),
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ));
            if victory {
                root.spawn(fonted(
                    art,
                    "MISSION COMPLETE",
                    60.0,
                    Color::srgb(1.0, 0.84, 0.38),
                ));
                root.spawn(fonted(
                    art,
                    "ALL THREE FILES ARE IN THE WIND. THE CAR IS GONE.",
                    20.0,
                    Color::srgb(0.45, 0.9, 0.92),
                ));
            } else {
                root.spawn(fonted(
                    art,
                    "MISSION FAILED",
                    60.0,
                    Color::srgb(0.95, 0.3, 0.32),
                ));
                root.spawn(fonted(
                    art,
                    "THE NIGHT DESK WILL NOT BE PLEASED.",
                    20.0,
                    Color::srgb(0.45, 0.9, 0.92),
                ));
            }
            root.spawn(fonted(
                art,
                format!("FINAL SCORE   {:06}", run.score),
                40.0,
                Color::WHITE,
            ));
            root.spawn(fonted(
                art,
                format!("HIGH SCORE    {:06}", run.high_score),
                30.0,
                Color::srgb(1.0, 0.83, 0.38),
            ));
            root.spawn(fonted(
                art,
                "PRESS ENTER TO RUN IT AGAIN      ESC FOR TITLE",
                20.0,
                Color::srgb(0.7, 0.76, 0.86),
            ));
        });
}
