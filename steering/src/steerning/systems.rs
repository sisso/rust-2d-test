use super::components::*;
use crate::math::*;

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
use utils::lerp_2;

pub struct BordersTeleportSystem;
impl<'a> System<'a> for BordersTeleportSystem {
    type SystemData = (ReadExpect<'a, Cfg>, WriteStorage<'a, Vehicle>);

    fn run(&mut self, (cfg, mut mobs): Self::SystemData) {
        use specs::Join;

        let (width, height) = (cfg.screen_width, cfg.screen_height);

        for (vehicle) in (&mut mobs).join() {
            if vehicle.pos.x > width as f32 {
                vehicle.pos.x -= width as f32;
            }
            if vehicle.pos.x < 0.0 {
                vehicle.pos.x += width as f32;
            }
            if vehicle.pos.y > height as f32 {
                vehicle.pos.y -= height as f32;
            }
            if vehicle.pos.y < 0.0 {
                vehicle.pos.y += height as f32;
            }
        }
    }
}

pub struct SteeringSeparationSystem;
impl<'a> System<'a> for SteeringSeparationSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Vehicle>,
        ReadStorage<'a, SteeringSeparation>,
    );

    fn run(&mut self, (entities, mut vehicles, separations): Self::SystemData) {
        use specs::Join;

        let mut changes = vec![];

        for (entity_a, vehicle_a, separation_a) in (&*entities, &vehicles, &separations).join() {
            for (entity_b, vehicle_b, separation_b) in (&*entities, &vehicles, &separations).join()
            {
                if entity_a == entity_b {
                    continue;
                }

                if !separation_a.enabled || !separation_b.enabled {
                    continue;
                }

                let min_distance = separation_a.distance + separation_b.distance;
                let vector = (vehicle_b.pos - vehicle_a.pos);
                let distance = vector.magnitude();

                if distance < min_distance {
                    let vel = lerp_2(
                        vehicle_a.max_speed * separation_a.weight,
                        0.0,
                        0.0,
                        min_distance,
                        distance,
                    );
                    let vector = vector.normalize() * -1.0;

                    changes.push((entity_a, vector * vel));
                }
            }
        }

        let vehicles = &mut vehicles;
        for (entity, desired_vel) in changes {
            let vehicle = vehicles.get_mut(entity).unwrap();
            vehicle.desired_vel += desired_vel;
        }
    }
}

pub struct SteeringVelocitySystem;
impl<'a> System<'a> for SteeringVelocitySystem {
    type SystemData = (ReadStorage<'a, SteeringVelocity>, WriteStorage<'a, Vehicle>);

    fn run(&mut self, (velocity, mut vehicles): Self::SystemData) {
        use specs::Join;

        for (velocity, vehicle) in (&velocity, &mut vehicles).join() {
            if !velocity.enabled {
                continue;
            }

            let vehicle: &mut Vehicle = vehicle;
            // TODO: change vel?
            vehicle.desired_vel += velocity.vel * velocity.weight;
            vehicle.desired_dir = velocity.vel.normalize();
        }
    }
}

pub struct SteeringWallsSystem;
impl<'a> System<'a> for SteeringWallsSystem {
    type SystemData = (WriteStorage<'a, Vehicle>, ReadStorage<'a, Wall>);

    fn run(&mut self, (mut vehicles, walls): Self::SystemData) {
        use specs::Join;

        for (vehicle) in (&mut vehicles).join() {
            for (wall) in (&walls).join() {
                if let Some(vector) =
                    compute_vector_from_point_to_segment(wall.pos, wall.vec, vehicle.pos)
                {
                    let distance = vector.magnitude();
                    if distance < wall.min_distance {
                        // let dir = vector.normalize() * -1.0;
                        let force_intensity = lerp_2(0.0, 1.0, wall.min_distance, 0.0, distance);
                        let desired_vel = wall.force * force_intensity;

                        // println!(
                        //     "wall collision pos {:?} vec {:?} dir {:?} distance {:?}",
                        //     vehicle.pos, vector, dir, distance
                        // );

                        // TODO: change vel?
                        vehicle.desired_vel += desired_vel;
                    }
                }
            }
        }
    }
}

pub struct MoveSystem;
impl<'a> System<'a> for MoveSystem {
    type SystemData = (ReadExpect<'a, GameTime>, WriteStorage<'a, Vehicle>);

    fn run(&mut self, (game_time, mut vehicles): Self::SystemData) {
        use specs::Join;

        let delta_time = game_time.delta_time;

        for (vehicle) in (&mut vehicles).join() {
            let vehicle: &mut Vehicle = vehicle;

            // apply steerning
            {
                // compute new velocity
                let desired_velocity = vehicle.desired_vel;
                vehicle.desired_vel = Vector2::zero();

                let current_vel = vehicle.vel_dir * vehicle.speed;
                let mut delta_velocity = desired_velocity - current_vel;
                if delta_velocity.magnitude() > vehicle.max_acc {
                    delta_velocity = delta_velocity.normalize() * vehicle.max_acc;
                }

                let new_vel = current_vel + delta_velocity * delta_time;

                // normalize velocity
                let mut new_speed = new_vel.magnitude();
                vehicle.vel_dir = new_vel / new_speed;

                if new_speed > vehicle.max_speed {
                    new_speed = vehicle.max_speed;
                }
                vehicle.speed = new_speed;
            }

            // move
            {
                vehicle.pos += vehicle.vel_dir * vehicle.speed * delta_time;
            }

            // update direction
            {
                vehicle.dir = rotate_towards(
                    vehicle.dir,
                    vehicle.desired_dir,
                    Rad(vehicle.rotation_speed * delta_time),
                );
            }
        }
    }
}

pub struct SteerArrivalSystem;
impl<'a> System<'a> for SteerArrivalSystem {
    type SystemData = (
        WriteStorage<'a, Vehicle>,
        WriteStorage<'a, SteeringArrival>,
        WriteExpect<'a, DebugStuff>,
    );

    fn run(&mut self, (mut mobs, mut steering_arrival, mut debug_stuff): Self::SystemData) {
        use specs::Join;
        let min_distance = 0.1;

        for (vehicle, arrival) in (&mut mobs, &mut steering_arrival).join() {
            let arrival: &mut SteeringArrival = arrival;
            if !arrival.enabled {
                continue;
            }

            let vehicle: &mut Vehicle = vehicle;
            let delta = arrival.target_pos - vehicle.pos;
            let distance: f32 = delta.magnitude();
            if !distance.is_normal() || distance < min_distance {
                // arrival
                arrival.arrived = true;
                continue;
            }
            arrival.arrived = false;

            let dir = delta / distance;

            let slowdown_distance = vehicle.max_speed * arrival.distance;

            let speed = if distance > slowdown_distance {
                vehicle.desired_dir = dir;
                vehicle.max_speed
            } else {
                lerp_2(0.0, vehicle.max_speed, 0.0, slowdown_distance, distance)
            };

            let desired_vel = dir * speed * arrival.weight;
            debug_stuff.push_line(
                vehicle.pos,
                vehicle.pos + desired_vel,
                Color::new(1.0, 1.0, 0.0, 1.0),
            );
            vehicle.desired_vel += desired_vel;
        }
    }
}

pub struct SteeringFormationSystem;
impl<'a> System<'a> for SteeringFormationSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Vehicle>,
        WriteStorage<'a, SteeringArrival>,
        ReadStorage<'a, SteeringFormationMember>,
    );

    fn run(&mut self, (entities, vehicles, mut arrivals, formations): Self::SystemData) {
        use specs::Join;

        let mut leader: Option<(P2, V2, P2)> = None;
        let mut followers = BitSet::new();
        let mut total_followers = 0;

        // for (e, v, f, a) in (&entities, &vehicles, &formations, &arrivals).join() {
        //     if f.index == 0 {
        //         leader = Some((v.pos, v.get_velocity(), a.target_pos));
        //     } else {
        //         followers.add(e.id());
        //         total_followers += 1;
        //     }
        // }
        //
        // let (leader_pos) = leader.unwrap();
        // let formation =
        // for (_e, a, f) in (followers, &mut arrivals, &formations).join() {
        //     a.target_pos = formation.get_pos(f.index);
        // }
    }
}

pub struct UpdateModelPosSystem;
impl<'a> System<'a> for UpdateModelPosSystem {
    type SystemData = (ReadStorage<'a, Vehicle>, WriteStorage<'a, Model>);

    fn run(&mut self, (vehicles, mut models): Self::SystemData) {
        use specs::Join;
        for (vehicle, model) in (&vehicles, &mut models).join() {
            model.pos = vehicle.pos;
            model.dir = vehicle.dir;
        }
    }
}
