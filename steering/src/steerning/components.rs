use commons::math::*;

use cgmath::{prelude::*, Deg, Point2, Rad, Vector2};
use ggez::graphics::Color;
use ggez::{GameError, GameResult};
use myelin_geometry::Polygon;
use rand::prelude::StdRng;
use rand::{thread_rng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs::{World, WorldExt};
use specs_derive::Component;

#[derive(Clone, Debug, Component, Serialize, Deserialize)]
pub struct Cfg {
    pub screen_width: f32,
    pub screen_height: f32,
    pub seed: u64,
    pub vehicles: usize,
    pub followers: usize,
    pub max_acc: f32,
    pub max_speed: f32,
    pub rotation_speed: f32,
    pub separation_radius: f32,
    pub start_position: [f32; 4],
    pub arrival_distance: f32,
}

impl Cfg {
    // pub fn new() -> Self {
    //     Cfg {
    //         screen_width: 0.0,
    //         screen_height: 0.0,
    //         seed: 0,
    //         vehicles: 0,
    //         followers: 0,
    //         max_acc: 0.0,
    //         max_speed: 0.0,
    //         rotation_speed: 0.0,
    //         separation_radius: 0.0,
    //         start_position: [0.0, 0.0, 0.0, 0.0],
    //         arrival_distance: 0.0
    //     }
    // }
}

#[derive(Clone, Debug, Component)]
pub struct DebugStuff {
    pub lines: Vec<(P2, P2, Color)>,
    pub circles: Vec<(P2, f32, Color)>,
}

impl DebugStuff {
    pub fn new() -> Self {
        DebugStuff {
            lines: Default::default(),
            circles: Default::default(),
        }
    }

    pub fn push_line(&mut self, a: P2, b: P2, color: Color) {
        self.lines.push((a, b, color));
    }

    pub fn take_lines(&mut self) -> Vec<(P2, P2, Color)> {
        std::mem::replace(&mut self.lines, vec![])
    }
}

#[derive(Clone, Debug, Component)]
pub struct GameTime {
    pub delta_time: f32,
}

#[derive(Clone, Debug, Component)]
pub struct MovingArea {
    pub polygons: Vec<Polygon>,
}

// fn to_point(p2: P2) -> GPoint {
//     (p2.x as f64, p2.y as f64).into()
// }

impl MovingArea {
    pub fn is_valid(&self, point: P2) -> bool {
        // self.polygons
        //     .iter()
        //     .any(|polygon| polygon.contains_point(to_point(point)))
        unimplemented!()
    }
}

#[derive(Clone, Debug, Component)]
pub struct Vehicle {
    pub pos: P2,
    /// normalized
    pub dir: V2,
    /// normalized
    pub desired_dir: V2,
    /// normalized
    pub vel_dir: V2,
    pub speed: f32,
    pub max_acc: f32,
    pub desired_vel: V2,
    pub rotation_speed: f32,
    pub max_speed: f32,
}

impl Vehicle {
    pub fn get_velocity(&self) -> V2 {
        self.vel_dir * self.speed
    }

    pub fn rotate_towards_vec(&mut self, vec: V2, delta_time: f32) {
        self.dir = rotate_towards(self.dir, vec, Rad(self.rotation_speed * delta_time));
    }
}

#[derive(Clone, Debug, Component)]
pub struct SteeringSeparation {
    pub enabled: bool,
    pub distance: f32,
    pub weight: f32,
}

#[derive(Clone, Debug, Component)]
pub struct SteeringVelocity {
    pub enabled: bool,
    pub vel: V2,
    pub weight: f32,
}

#[derive(Clone, Debug, Component)]
pub struct SteeringArrival {
    pub enabled: bool,
    pub target_pos: P2,
    pub distance: f32,
    pub weight: f32,
    pub arrived: bool,
}

// #[derive(Clone, Debug, Component)]
// struct SteeringKeepPosition {
//     enable: bool,
//     target_pos: P2,
// }

#[derive(Clone, Debug, Component)]
pub struct SteeringFormationLeader {
    pub formation: FormationType,
}

#[derive(Clone, Debug, Component)]
pub struct SteeringFormationMember {
    pub index: usize,
}

#[derive(Clone, Debug, Copy)]
pub enum FormationType {
    Circle,
    Bar,
    Line,
    Column,
}

impl FormationType {
    // // TODO: formations need to be persistent between interaction to remove flackness
    // pub fn new(leader_pos: P2, leader_vel: V2, target_pos: P2, total_members: usize) -> Self {
    //     let mut right = Vector2::new(0.0, 15.0);
    //
    //     // let delta_to_target = target_pos - leader_pos;
    //     // if delta_to_target.magnitude() > 1.0 {
    //     //     let dir = delta_to_target.normalize();
    //     //     right = rotate_vector(dir, right);
    //     // }
    //
    //     let leader_speed = leader_vel.magnitude();
    //     if leader_speed > 30.0 {
    //         let dir = leader_vel / leader_speed;
    //         right = rotate_vector(dir, right);
    //     }
    //
    //     Formation {
    //         pos: leader_pos + leader_vel,
    //         next: right,
    //     }
    // }

    pub fn get_pos(&self, look_dir: V2, leader_pos: P2, total: usize, index: usize) -> P2 {
        if index == 0 {
            return leader_pos;
        }

        let mult = if (index) % 2 == 0 {
            index as f32 / 2.0
        } else {
            -1.0 * index as f32 / 2.0
        };
        let vec = Vector2::new(0.0, 20.0) * mult;
        let rotated = rotate_vector(look_dir, vec);
        leader_pos + rotated
    }
}

#[derive(Clone, Debug, Component)]
pub struct Model {
    pub size: f32,
    pub pos: P2,
    /// normalize direction
    pub dir: V2,
    pub color: Color,
}

#[derive(Clone, Debug, Component)]
pub struct Wall {
    pub pos: P2,
    pub vec: V2,
    pub force: V2,
    pub min_distance: f32,
}

impl Wall {
    pub fn new_from_points(p0: P2, p1: P2, min_distance: f32, force: V2) -> Self {
        let vec = p1.to_vec() - p0.to_vec();
        Wall {
            pos: p0,
            vec: vec,
            force: force,
            min_distance,
        }
    }
}
