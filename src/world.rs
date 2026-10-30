//! Shared world layout, level data and small pure helpers.
//!
//! World space is 320x240 with (0,0) in the middle, exactly three screen
//! pixels per world unit at the default 960x720 window.

pub const WORLD_W: f32 = 320.0;
pub const WORLD_H: f32 = 240.0;
pub const FLOOR_COUNT: usize = 8;

/// Floor surface heights, top floor first.  Screen y = 120 - world y.
pub const FLOOR_Y: [f32; FLOOR_COUNT] = [90.0, 64.0, 38.0, 12.0, -14.0, -40.0, -66.0, -92.0];

pub const SHAFT_X: [f32; 2] = [-42.0, 60.0];
pub const SHAFT_COUNT: usize = 2;
pub const SHAFT_HALF: f32 = 10.0;

pub const BUILDING_LEFT: f32 = -136.0;
pub const BUILDING_RIGHT: f32 = 136.0;
pub const SPAWN_X: f32 = -60.0;
pub const EXIT_X: f32 = 112.0;

pub const PLAYER_W: f32 = 10.0;
pub const PLAYER_H: f32 = 22.0;
pub const PLAYER_SPEED: f32 = 58.0;
pub const PLAYER_JUMP: f32 = 112.0;
pub const GRAVITY: f32 = 430.0;

pub const ENEMY_W: f32 = 10.0;
pub const ENEMY_H: f32 = 22.0;

/// Car bottom sits one pixel below the floor surface, so its plate reads as
/// being flush with the slab.
pub fn car_bottom_for_floor(floor: usize) -> f32 {
    FLOOR_Y[floor] - 1.0
}

pub const CAR_SPEED: f32 = 44.0;
pub const CAR_PLATE_OFFSET: f32 = 5.0;

pub fn clamp_x(x: f32, half_width: f32) -> f32 {
    x.clamp(BUILDING_LEFT + half_width, BUILDING_RIGHT - half_width)
}

pub fn floor_label(index: usize) -> String {
    if index >= FLOOR_COUNT - 1 {
        "LOBBY".to_string()
    } else {
        format!("{:02}", FLOOR_COUNT - 1 - index)
    }
}

pub fn nearest_floor(y: f32) -> usize {
    let mut best = 0;
    let mut best_dist = f32::MAX;
    for (i, fy) in FLOOR_Y.iter().enumerate() {
        let d = (y - fy).abs();
        if d < best_dist {
            best_dist = d;
            best = i;
        }
    }
    best
}

pub fn floor_index_for_y(y: f32) -> Option<usize> {
    FLOOR_Y.iter().position(|fy| (y - fy).abs() < 2.0)
}

/// True when two bottom-centre anchored rectangles intersect.
#[allow(clippy::too_many_arguments)]
pub fn feet_overlap(
    ax: f32,
    ay: f32,
    aw: f32,
    ah: f32,
    bx: f32,
    by: f32,
    bw: f32,
    bh: f32,
) -> bool {
    let a_left = ax - aw * 0.5;
    let a_right = ax + aw * 0.5;
    let b_left = bx - bw * 0.5;
    let b_right = bx + bw * 0.5;
    a_left < b_right && a_right > b_left && ay < by + bh && ay + ah > by
}

#[derive(Clone, Copy, Debug)]
pub struct DoorSpec {
    pub floor: usize,
    pub x: f32,
    /// Red document door (worth a file) rather than a gray enemy door.
    pub secret: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct LevelDef {
    pub name: &'static str,
    pub theme: &'static str,
    pub background: &'static str,
    pub doors: &'static [DoorSpec],
    pub enemy_interval: f32,
    pub max_enemies: usize,
    pub enemy_speed: f32,
    pub enemy_accuracy: f32,
}

const fn d(floor: usize, x: f32, secret: bool) -> DoorSpec {
    DoorSpec { floor, x, secret }
}

// Layouts keep doors clear of the two shafts (x -53..-31 and 49..71) and the
// getaway arch on the bottom-right of floor 6 (x > 99).
pub const LEVEL_1: LevelDef = LevelDef {
    name: "THE HOTEL MERIDIAN",
    theme: "MIDNIGHT",
    background: "backgrounds/bg_midnight.png",
    doors: &[
        d(1, -20.0, false),
        d(1, -100.0, true),
        d(1, 100.0, false),
        d(2, 40.0, true),
        d(2, -110.0, false),
        d(3, -110.0, true),
        d(3, 120.0, false),
        d(4, 20.0, true),
        d(4, -90.0, false),
        d(5, 110.0, true),
        d(5, -20.0, false),
        d(6, -70.0, true),
        d(6, 80.0, false),
        d(7, 0.0, false),
    ],
    enemy_interval: 2.6,
    max_enemies: 4,
    enemy_speed: 26.0,
    enemy_accuracy: 0.75,
};

pub const LEVEL_2: LevelDef = LevelDef {
    name: "THE NIGHT EXCHANGE",
    theme: "AMBER",
    background: "backgrounds/bg_amber.png",
    doors: &[
        d(1, -120.0, true),
        d(1, 0.0, true),
        d(1, 40.0, false),
        d(1, -100.0, false),
        d(2, -90.0, true),
        d(2, 110.0, false),
        d(3, 80.0, true),
        d(3, -20.0, false),
        d(4, -110.0, true),
        d(4, 90.0, false),
        d(5, 30.0, true),
        d(5, -80.0, false),
        d(6, -120.0, true),
        d(6, -10.0, false),
        d(7, 20.0, false),
    ],
    enemy_interval: 2.2,
    max_enemies: 5,
    enemy_speed: 30.0,
    enemy_accuracy: 0.82,
};

pub const LEVEL_3: LevelDef = LevelDef {
    name: "THE RED DOOR PROTOCOL",
    theme: "VIOLET",
    background: "backgrounds/bg_violet.png",
    doors: &[
        d(1, -20.0, true),
        d(1, -80.0, true),
        d(1, 40.0, false),
        d(1, 100.0, false),
        d(2, 20.0, true),
        d(2, -70.0, false),
        d(3, -120.0, true),
        d(3, 100.0, false),
        d(4, 80.0, true),
        d(4, -100.0, false),
        d(5, -20.0, true),
        d(5, 80.0, false),
        d(6, 80.0, true),
        d(6, 20.0, false),
        d(7, -90.0, true),
        d(7, 90.0, false),
    ],
    enemy_interval: 1.9,
    max_enemies: 6,
    enemy_speed: 34.0,
    enemy_accuracy: 0.88,
};

pub const LEVELS: [LevelDef; 3] = [LEVEL_1, LEVEL_2, LEVEL_3];

pub fn docs_in(level: &LevelDef) -> usize {
    level.doors.iter().filter(|d| d.secret).count()
}

#[derive(Clone, Debug)]
pub struct CarState {
    pub bottom: f32,
    pub target_floor: usize,
    pub moving: bool,
    pub riders: u32,
}

impl CarState {
    pub fn new(floor: usize) -> Self {
        Self {
            bottom: car_bottom_for_floor(floor),
            target_floor: floor,
            moving: false,
            riders: 0,
        }
    }

    pub fn current_floor(&self) -> usize {
        let mut best = 0;
        let mut best_dist = f32::MAX;
        for (i, fy) in FLOOR_Y.iter().enumerate() {
            let d = (self.bottom - (fy - 1.0)).abs();
            if d < best_dist {
                best_dist = d;
                best = i;
            }
        }
        best
    }

    pub fn docked_floor(&self) -> Option<usize> {
        if self.moving {
            return None;
        }
        let f = self.current_floor();
        if (self.bottom - car_bottom_for_floor(f)).abs() < 0.75 {
            Some(f)
        } else {
            None
        }
    }

    pub fn request(&mut self, floor: usize) {
        if self.moving || self.docked_floor() != Some(floor) {
            self.target_floor = floor.min(FLOOR_COUNT - 1);
            self.moving = true;
        }
    }

    /// Advance one frame. Returns the floor that was reached this frame.
    pub fn update(&mut self, dt: f32) -> Option<usize> {
        if !self.moving {
            return None;
        }
        let target = car_bottom_for_floor(self.target_floor);
        let delta = target - self.bottom;
        let step = CAR_SPEED * dt;
        if delta.abs() <= step {
            self.bottom = target;
            self.moving = false;
            Some(self.target_floor)
        } else {
            self.bottom += delta.signum() * step;
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn car_rides_and_docks() {
        let mut car = CarState::new(0);
        assert_eq!(car.docked_floor(), Some(0));
        car.request(4);
        assert!(car.moving);
        let mut ticks = 0;
        while car.moving && ticks < 1000 {
            car.update(1.0 / 60.0);
            ticks += 1;
        }
        assert!(!car.moving);
        assert_eq!(car.docked_floor(), Some(4));
        assert!((car.bottom - car_bottom_for_floor(4)).abs() < 0.001);
    }

    #[test]
    fn car_does_not_move_when_already_there() {
        let mut car = CarState::new(2);
        car.request(2);
        assert!(!car.moving);
        assert_eq!(car.docked_floor(), Some(2));
    }

    #[test]
    fn overlap_is_strict() {
        assert!(feet_overlap(0.0, 0.0, 10.0, 20.0, 5.0, 0.0, 10.0, 20.0));
        assert!(!feet_overlap(0.0, 0.0, 10.0, 20.0, 10.0, 0.0, 10.0, 20.0));
        assert!(!feet_overlap(0.0, 0.0, 10.0, 20.0, 0.0, 20.0, 10.0, 20.0));
    }

    #[test]
    fn levels_have_expected_documents() {
        assert_eq!(docs_in(&LEVEL_1), 6);
        assert_eq!(docs_in(&LEVEL_2), 7);
        assert_eq!(docs_in(&LEVEL_3), 8);
    }

    #[test]
    fn doors_avoid_shafts() {
        for level in LEVELS {
            for door in level.doors {
                for sx in SHAFT_X {
                    assert!(
                        (door.x - sx).abs() > 17.0,
                        "{}: door at {} floor {} overlaps shaft {}",
                        level.name,
                        door.x,
                        door.floor,
                        sx
                    );
                }
                if door.floor == 6 {
                    assert!(door.x < 95.0, "door blocks the getaway arch");
                }
            }
        }
    }

    #[test]
    fn floor_math() {
        assert_eq!(nearest_floor(90.0), 0);
        assert_eq!(nearest_floor(-92.0), 7);
        assert_eq!(floor_index_for_y(38.0), Some(2));
        assert_eq!(clamp_x(999.0, 5.0), BUILDING_RIGHT - 5.0);
    }
}
