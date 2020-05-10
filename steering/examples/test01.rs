extern crate steerning;
use steerning::*;

use cgmath::{
    prelude::*, vec2, vec3, Deg, Euler, InnerSpace, Point2, Quaternion, Rad, Vector2, VectorSpace,
};
use ggez::conf::WindowMode;
use ggez::event::{self, Button, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::Color;
use ggez::{graphics, timer, Context, ContextBuilder, GameError, GameResult};
use myelin_geometry::{Point as GPoint, Polygon};
use rand::prelude::StdRng;
use rand::{thread_rng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs::{World, WorldExt};
use specs_derive::Component;
use std::ops::Deref;
use utils::lerp_2;

// TODO: add parallelism

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;

#[derive(Clone, Debug, Component, Serialize, Deserialize)]
struct Cfg {
    seed: u64,
    vehicles: usize,
    followers: usize,
    max_acc: f32,
    max_speed: f32,
    rotation_speed: f32,
    separation_radius: f32,
}

impl Cfg {
    pub fn new() -> Self {
        Cfg {
            seed: 0,
            vehicles: 0,
            followers: 0,
            max_acc: 0.0,
            max_speed: 0.0,
            rotation_speed: 0.0,
            separation_radius: 0.0,
        }
    }
}

#[derive(Clone, Debug, Component)]
struct DebugStuff {
    lines: Vec<(P2, P2, Color)>,
    circles: Vec<(P2, f32, Color)>,
}

impl DebugStuff {
    pub fn new() -> Self {
        DebugStuff {
            lines: Default::default(),
            circles: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Component)]
struct GameTime {
    delta_time: f32,
}

#[derive(Clone, Debug, Component)]
struct MovingArea {
    polygons: Vec<Polygon>,
}

fn to_point(p2: P2) -> GPoint {
    (p2.x as f64, p2.y as f64).into()
}

impl MovingArea {
    pub fn is_valid(&self, point: P2) -> bool {
        self.polygons
            .iter()
            .any(|polygon| polygon.contains_point(to_point(point)))
    }
}

/// Replace vel by unit vel + speed
#[derive(Clone, Debug, Component)]
struct Vehicle {
    pos: P2,
    /// normalized
    dir: V2,
    desired_dir: V2,
    vel: V2,
    max_acc: f32,
    // TODO: replace by single vector that is zero in start
    steering_vel: Vec<V2>,
    rotation_speed: f32,
    max_speed: f32,
    desired_dir_toward_vel: bool,
}

impl Vehicle {
    pub fn rotate_towards_vec(&mut self, vec: V2, delta_time: f32) {
        self.dir = rotate_towards(self.dir, vec, Rad(self.rotation_speed * delta_time));
    }
}

#[derive(Clone, Debug, Component)]
struct SteeringSeparation {
    enabled: bool,
    distance: f32,
    weight: f32,
}

#[derive(Clone, Debug, Component)]
struct SteeringVelocity {
    enabled: bool,
    vel: V2,
    weight: f32,
}

#[derive(Clone, Debug, Component)]
struct SteeringArrival {
    enabled: bool,
    target_pos: P2,
    distance: f32,
    weight: f32,
}

#[derive(Clone, Debug, Component)]
struct SteeringFormationLeader {
    formation: Option<Formation>,
}

#[derive(Clone, Debug, Component)]
struct SteeringFormationMember {
    index: usize,
}

#[derive(Clone, Debug)]
struct Formation {
    pos: P2,
    next: V2,
}

// #[derive(Clone, Debug)]
// enum FormationType {
//     Circle,
//     Bar,
//     Line,
//     Column
// }

impl Formation {
    // TODO: formations need to be persistent between interaction to remove flackness
    pub fn new(leader_pos: P2, leader_vel: V2, target_pos: P2, total_members: usize) -> Self {
        let mut right = Vector2::new(0.0, 15.0);

        // let delta_to_target = target_pos - leader_pos;
        // if delta_to_target.magnitude() > 1.0 {
        //     let dir = delta_to_target.normalize();
        //     right = rotate_vector(dir, right);
        // }

        let leader_speed = leader_vel.magnitude();
        if leader_speed > 30.0 {
            let dir = leader_vel / leader_speed;
            right = rotate_vector(dir, right);
        }

        Formation {
            pos: leader_pos + leader_vel,
            next: right,
        }
    }

    pub fn get_pos(&self, index: usize) -> P2 {
        let mult = if index % 2 == 0 { 1.0 } else { -1.0 };
        self.pos + self.next * ((index / 2) as f32) * mult
    }
}

#[derive(Clone, Debug, Component)]
struct Model {
    size: f32,
    pos: P2,
    /// normalize direction
    dir: V2,
    color: graphics::Color,
}

#[derive(Clone, Debug, Component)]
struct Wall {
    pos: P2,
    vec: V2,
    force: V2,
    min_distance: f32,
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

struct App {
    update_next: bool,
    world: World,
}

impl App {
    pub fn new(ctx: &mut Context) -> GameResult<App> {
        let cfg = App::load_cfg()?;
        let world = App::create_world(cfg)?;
        let game = App {
            update_next: true,
            world,
        };
        Ok(game)
    }

    pub fn reload(&mut self) -> GameResult<()> {
        let cfg = App::load_cfg()?;
        let world = App::create_world(cfg)?;
        self.world = world;
        Ok(())
    }

    pub fn load_cfg() -> GameResult<Cfg> {
        let reader =
            std::io::BufReader::new(std::fs::File::open("steering/resources/config.json").unwrap());

        let cfg: serde_json::Result<Cfg> = serde_json::from_reader(reader);

        match cfg {
            Ok(cfg) => Ok(cfg),
            Err(e) => Err(GameError::ConfigError("invalid config file".to_string())),
        }
    }

    pub fn create_world(cfg: Cfg) -> GameResult<World> {
        let mut rng: StdRng = SeedableRng::seed_from_u64(0);

        // create world
        let mut world = World::new();
        world.register::<Model>();
        world.register::<Cfg>();
        world.register::<Vehicle>();
        world.register::<MovingArea>();
        world.register::<SteeringArrival>();
        world.register::<SteeringSeparation>();
        world.register::<SteeringVelocity>();
        world.register::<Wall>();
        world.register::<SteeringFormationMember>();
        world.register::<SteeringFormationLeader>();

        world.insert(GameTime { delta_time: 0.01 });
        world.insert(DebugStuff::new());

        {
            let border = 80.0;
            let points = vec![
                (border, border).into(),
                (WIDTH as f64 - border, border).into(),
                (WIDTH as f64 - border, HEIGHT as f64 - border).into(),
                (border, HEIGHT as f64 - border).into(),
            ];

            world.insert(MovingArea {
                polygons: vec![Polygon::try_new(points.clone()).unwrap()],
            });

            let wall_width = 15.0;
            let wall_force = 200.0;

            for (p0, p1, distance, force) in vec![
                (
                    points[0],
                    points[1],
                    wall_width,
                    Vector2::new(0.0, 1.0) * wall_force,
                ),
                (
                    points[1],
                    points[2],
                    wall_width,
                    Vector2::new(-1.0, 0.0) * wall_force,
                ),
                (
                    points[2],
                    points[3],
                    wall_width,
                    Vector2::new(0.0, -1.0) * wall_force,
                ),
                (
                    points[3],
                    points[0],
                    wall_width,
                    Vector2::new(1.0, 0.0) * wall_force,
                ),
            ] {
                let p0 = Point2::new(p0.x as f32, p0.y as f32);
                let p1 = Point2::new(p1.x as f32, p1.y as f32);

                world
                    .create_entity()
                    .with(Wall::new_from_points(p0, p1, distance, force))
                    .build();
            }
        }

        let max_acc = cfg.max_acc;
        let max_speed = cfg.max_speed;
        let separation_mut = cfg.separation_radius;

        let mut formation_index = 0;

        for i in 0..cfg.vehicles {
            let pos = Point2::new(
                rng.gen_range(0.0, WIDTH as f32),
                rng.gen_range(0.0, HEIGHT as f32),
            );

            let vel = Vector2::new(
                rng.gen_range(-max_speed, max_speed),
                rng.gen_range(-max_speed, max_speed),
            );

            let radius = if rng.gen::<f32>() <= 0.1f32 { 6.0 } else { 3.0 };
            let follow = i < cfg.followers;

            let color = if follow {
                if formation_index == 0 {
                    Color::new(0.0, 1.0, 1.0, 1.0)
                } else {
                    Color::new(1.0, 1.0, 0.0, 1.0)
                }
            } else {
                Color::new(1.0, 0.0, 0.0, 1.0)
            };

            let dir =
                rotate_vector_by_angle(Vector2::unit_y(), Deg(rng.gen_range(0.0, 360.0)).into());

            let mut builder = world
                .create_entity()
                .with(Vehicle {
                    pos,
                    dir: dir,
                    desired_dir: dir,
                    vel: Vector2::zero(),
                    max_acc: max_acc,
                    rotation_speed: deg2rad(cfg.rotation_speed),
                    steering_vel: vec![],
                    max_speed,
                    desired_dir_toward_vel: false,
                })
                .with(Model {
                    size: radius,
                    pos: Point2::new(0.0, 0.0),
                    dir,
                    color,
                })
                .with(SteeringArrival {
                    enabled: follow,
                    target_pos: Point2::new(300.0, 300.0),
                    distance: 50.0,
                    weight: 1.0,
                })
                .with(SteeringSeparation {
                    enabled: true,
                    distance: radius * separation_mut,
                    weight: 2.0,
                })
                .with(SteeringVelocity {
                    enabled: !follow,
                    vel: vel,
                    weight: 1.0,
                });

            if follow {
                if formation_index == 0 {
                    builder = builder.with(SteeringFormationLeader { formation: None });
                }

                builder = builder.with(SteeringFormationMember {
                    index: formation_index,
                });

                formation_index += 1;
            }

            builder.build();
        }

        world.insert(cfg);

        Ok(world)
    }
}

struct BordersTeleportSystem;
impl<'a> System<'a> for BordersTeleportSystem {
    type SystemData = (WriteStorage<'a, Vehicle>);

    fn run(&mut self, (mut mobs): Self::SystemData) {
        use specs::Join;

        for (vehicle) in (&mut mobs).join() {
            if vehicle.pos.x > WIDTH as f32 {
                vehicle.pos.x -= WIDTH as f32;
            }
            if vehicle.pos.x < 0.0 {
                vehicle.pos.x += WIDTH as f32;
            }
            if vehicle.pos.y > HEIGHT as f32 {
                vehicle.pos.y -= HEIGHT as f32;
            }
            if vehicle.pos.y < 0.0 {
                vehicle.pos.y += HEIGHT as f32;
            }
        }
    }
}

struct SteeringSeparationSystem;
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
                    changes.push((
                        entity_a,
                        vector,
                        separation_a.weight,
                        min_distance,
                        distance,
                    ));
                    // changes.push((entity_b, vector, min_distance, distance));
                }
            }
        }

        let vehicles = &mut vehicles;
        for (entity, vector, weight, min_distance, distance) in changes {
            let vehicle = vehicles.get_mut(entity).unwrap();
            let vel = lerp_2(vehicle.max_speed * weight, 0.0, 0.0, min_distance, distance);
            let vector = vector.normalize() * -1.0;
            vehicle.steering_vel.push(vector * vel);
        }
    }
}

struct SteeringVelocitySystem;
impl<'a> System<'a> for SteeringVelocitySystem {
    type SystemData = (ReadStorage<'a, SteeringVelocity>, WriteStorage<'a, Vehicle>);

    fn run(&mut self, (velocity, mut vehicles): Self::SystemData) {
        use specs::Join;

        for (velocity, vehicle) in (&velocity, &mut vehicles).join() {
            if !velocity.enabled {
                continue;
            }

            let vehicle: &mut Vehicle = vehicle;
            vehicle.steering_vel.push(velocity.vel * velocity.weight);
            vehicle.desired_dir = velocity.vel.normalize();
        }
    }
}

struct SteeringWallsSystem;
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

                        vehicle.steering_vel.push(desired_vel);
                    }
                }
            }
        }
    }
}

struct MoveSystem;
impl<'a> System<'a> for MoveSystem {
    type SystemData = (ReadExpect<'a, GameTime>, WriteStorage<'a, Vehicle>);

    fn run(&mut self, (game_time, mut vehicles): Self::SystemData) {
        use specs::Join;

        let delta_time = game_time.delta_time;

        for (vehicle) in (&mut vehicles).join() {
            let vehicle: &mut Vehicle = vehicle;

            // compute new velocity
            let velocities = std::mem::replace(&mut vehicle.steering_vel, vec![]);
            let desired_velocity = velocities.into_iter().fold(Vector2::zero(), |a, b| a + b);

            let mut delta_velocity = desired_velocity - vehicle.vel;
            if delta_velocity.magnitude() > vehicle.max_acc {
                delta_velocity = delta_velocity.normalize() * vehicle.max_acc;
            }

            vehicle.vel += delta_velocity * delta_time;

            // normalize velocity
            let current_speed = vehicle.vel.magnitude();
            if current_speed > vehicle.max_speed {
                vehicle.vel = vehicle.vel.normalize() * vehicle.max_speed;
            }

            // move
            vehicle.pos += vehicle.vel * delta_time;

            // update direction
            if vehicle.desired_dir_toward_vel && current_speed > 1.0 {
                vehicle.desired_dir = vehicle.vel.normalize();
            }

            vehicle.dir = rotate_towards(
                vehicle.dir,
                vehicle.desired_dir,
                Rad(vehicle.rotation_speed * delta_time),
            );
        }
    }
}

struct SteerArrivalSystem;
impl<'a> System<'a> for SteerArrivalSystem {
    type SystemData = (
        ReadExpect<'a, GameTime>,
        WriteStorage<'a, Vehicle>,
        ReadStorage<'a, SteeringArrival>,
    );

    fn run(&mut self, (game_time, mut mobs, steering_arrival): Self::SystemData) {
        use specs::Join;

        for (vehicle, arrival) in (&mut mobs, &steering_arrival).join() {
            let arrival: &SteeringArrival = arrival;
            if !arrival.enabled {
                continue;
            }

            let vehicle: &mut Vehicle = vehicle;
            let delta = arrival.target_pos - vehicle.pos;
            let distance: f32 = delta.magnitude();
            if !distance.is_normal() || distance < 0.1 {
                // arrival
                continue;
            }

            let speed = if distance > arrival.distance {
                vehicle.max_speed
            } else {
                lerp_2(0.0, vehicle.max_speed, 0.0, arrival.distance, distance)
            };

            let step_speed = speed * game_time.delta_time;
            if step_speed > distance {
                // complete
                // vehicle.pos = arrival.target_pos;
            } else {
                let dir = delta.normalize();
                vehicle.steering_vel.push(dir * speed * arrival.weight);
                vehicle.desired_dir = dir;
            }
        }
    }
}

struct SteeringFormationSystem;
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

        for (e, v, f, a) in (&entities, &vehicles, &formations, &arrivals).join() {
            if f.index == 0 {
                leader = Some((v.pos, v.vel, a.target_pos));
            } else {
                followers.add(e.id());
                total_followers += 1;
            }
        }

        let (leader_pos, leader_vel, leader_target_pos) = leader.unwrap();
        let formation = Formation::new(leader_pos, leader_vel, leader_target_pos, total_followers);
        for (e, a, f) in (followers, &mut arrivals, &formations).join() {
            a.target_pos = formation.get_pos(f.index);
        }
    }
}

struct UpdateModelPosSystem;
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

fn draw_circle(
    ctx: &mut Context,
    pos: P2,
    size: f32,
    color: Color,
    width: f32,
    fill: bool,
) -> GameResult<()> {
    let mode = if fill {
        graphics::DrawMode::fill()
    } else {
        graphics::DrawMode::stroke(width)
    };

    let circle = graphics::Mesh::new_circle(ctx, mode, pos, size, 0.1, color)?;

    graphics::draw(ctx, &circle, graphics::DrawParam::default())
}

fn draw_line(ctx: &mut Context, p0: P2, p1: P2, color: Color, width: f32) -> GameResult<()> {
    let mesh = graphics::Mesh::new_line(ctx, &[p0, p1], width, color)?;
    graphics::draw(ctx, &mesh, graphics::DrawParam::default())
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let delta = timer::delta(ctx).as_secs_f32();
        self.world.insert(GameTime { delta_time: delta });

        if self.update_next {
            let mut dispatcher = DispatcherBuilder::new()
                .with(SteerArrivalSystem, "steering_arrival", &[])
                .with(SteeringSeparationSystem, "steering_separation", &[])
                .with(SteeringVelocitySystem, "steering_velocity", &[])
                .with(SteeringWallsSystem, "steering_walls", &[])
                // .with(SteeringFormationSystem, "steering_formation", &[])
                .with(
                    MoveSystem,
                    "move",
                    &[
                        "steering_arrival",
                        "steering_separation",
                        "steering_velocity",
                        "steering_walls",
                        // "steering_formation",
                    ],
                )
                .with(BordersTeleportSystem, "border_teleport", &["move"])
                .with(UpdateModelPosSystem, "update_model", &["border_teleport"])
                .build();

            dispatcher.run_now(&mut self.world);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        let cfg = &self.world.read_resource::<Cfg>();
        let color_wall = Color::new(0.0, 1.0, 0.5, 0.5);

        {
            for wall in (&self.world.read_storage::<Wall>()).join() {
                let mut mb = graphics::MeshBuilder::new();
                mb.line(
                    &[wall.pos, Point2::from_vec(wall.pos.to_vec() + wall.vec)],
                    wall.min_distance * 2.0,
                    color_wall,
                );
                let mesh = mb.build(ctx)?;
                graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
            }
        }

        {
            let moving_area = &self.world.read_resource::<MovingArea>();

            for polygon in &moving_area.polygons {
                let points: Vec<P2> = polygon
                    .vertices()
                    .iter()
                    .map(|point| Point2::new(point.x as f32, point.y as f32))
                    .collect();

                let walking_area_color = Color::new(1.0, 1.0, 1.0, 1.0);
                let mesh = graphics::Mesh::new_polygon(
                    ctx,
                    graphics::DrawMode::stroke(1.0),
                    points.as_slice(),
                    walking_area_color,
                )?;

                graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
            }
        }

        {
            let models = &self.world.read_storage::<Model>();

            for (model) in (models).join() {
                draw_circle(ctx, model.pos, model.size, model.color, 1.0, false)?;
                draw_line(
                    ctx,
                    model.pos,
                    Point2::from_vec(model.pos.to_vec() + model.dir * model.size),
                    model.color,
                    1.0,
                )?;
            }
        }

        let cfg = &self.world.read_resource::<Cfg>();
        let text = graphics::Text::new(format!(
            "ftps: {}, {:?}",
            ggez::timer::fps(ctx) as i32,
            cfg.deref()
        ));
        graphics::draw(ctx, &text, (cgmath::Point2::new(0.0, 0.0), graphics::WHITE))?;

        graphics::present(ctx)
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        let mut arrival = self.world.write_component::<SteeringArrival>();
        let formation = self.world.read_component::<SteeringFormationMember>();
        for (formation, arrival) in (&formation, &mut arrival).join() {
            // if formation.index != 0 {
            //     continue;
            // }

            arrival.target_pos = (x, y).into();
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        match keycode {
            KeyCode::Space => {
                self.update_next = !self.update_next;
            }
            KeyCode::Return => {
                if let Err(e) = self.reload() {
                    println!("fail to relaod config file: {:?}", e);
                }
            }
            _ => {}
        }
    }
}

fn main() -> GameResult<()> {
    // Make a Context.
    let mut window_mode: WindowMode = Default::default();
    window_mode.width = WIDTH;
    window_mode.height = HEIGHT;

    let (mut ctx, mut event_loop) = ContextBuilder::new("my_game", "Cool Game Author")
        .window_mode(window_mode)
        .build()
        .expect("aieee, could not create ggez context!");

    let mut app = App::new(&mut ctx)?;

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut app) {
        Ok(_) => {
            println!("Exited cleanly.");
            Ok(())
        }
        Err(e) => {
            println!("Error occured: {}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn testgeometry() {
        use myelin_geometry::{Point as MPoint, Polygon};

        let polygon = Polygon::try_new(vec![
            (0.0, 0.0).into(),
            (5.0, 0.0).into(),
            (5.0, 5.0).into(),
            (0.0, 5.0).into(),
        ])
        .unwrap();

        println!("{:?}", polygon);
        println!("{:?}", polygon.contains_point((10.0, 10.0).into()));
        println!("{:?}", polygon.contains_point((2.0, 3.0).into()));
    }
}
