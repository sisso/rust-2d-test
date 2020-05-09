extern crate steerning;
use steerning::*;

use cgmath::{
    prelude::*, vec2, vec3, Deg, Euler, InnerSpace, Point2, Quaternion, Rad, Vector2, VectorSpace,
};
use ggez::conf::WindowMode;
use ggez::event::{self, Button, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::Color;
use ggez::{graphics, timer, Context, ContextBuilder, GameResult};
use myelin_geometry::{Point as GPoint, Polygon};
use rand::prelude::StdRng;
use rand::{thread_rng, Rng, SeedableRng};
use specs::prelude::*;
use specs::{World, WorldExt};
use specs_derive::Component;
use std::ops::Deref;
use utils::lerp_2;

// TODO: add parallelism

const WIDTH: f32 = 1600.0;
const HEIGHT: f32 = 1200.0;

#[derive(Clone, Debug, Component)]
struct Cfg {
    update_next: bool,
    speed_reduction: f32,
    draw_walls: bool,
    mobs: usize,
}

impl Cfg {
    pub fn new() -> Self {
        Cfg {
            update_next: true,
            speed_reduction: 0.9,
            draw_walls: true,
            mobs: 100,
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

#[derive(Clone, Debug, Component)]
struct Vehicle {
    pos: P2,
    vel: V2,
    max_acc: f32,
    steering_vel: Vec<V2>,
    max_speed: f32,
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
struct Model {
    size: f32,
    pos: P2,
    color: graphics::Color,
}

#[derive(Clone, Debug, Component)]
struct Wall {
    pos: P2,
    vec: V2,
    force: f32,
    min_distance: f32,
}

impl Wall {
    pub fn new_from_points(p0: P2, p1: P2, min_distance: f32, force: f32) -> Self {
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
    world: World,
}

impl App {
    pub fn new(ctx: &mut Context) -> GameResult<App> {
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

        let cfg = Cfg::new();

        world.insert(GameTime { delta_time: 0.01 });

        {
            let border = 160.0;
            let points = vec![
                (border, border).into(),
                (WIDTH as f64 - border, border).into(),
                (WIDTH as f64 - border, HEIGHT as f64 - border).into(),
                (border, HEIGHT as f64 - border).into(),
            ];

            world.insert(MovingArea {
                polygons: vec![Polygon::try_new(points.clone()).unwrap()],
            });

            for (p0, p1, f, d) in vec![
                (points[0], points[1], 10.0, 20.0),
                (points[1], points[2], 10.0, 20.0),
                (points[2], points[3], 10.0, 20.0),
                (points[3], points[0], 10.0, 20.0),
            ] {
                let p0 = Point2::new(p0.x as f32, p0.y as f32);
                let p1 = Point2::new(p1.x as f32, p1.y as f32);

                world
                    .create_entity()
                    .with(Wall::new_from_points(p0, p1, f, d))
                    .build();
            }
        }

        let max_acc = 100.0;
        let max_speed = 50.0;
        let radius_mut = 2.0;

        for _ in 0..cfg.mobs {
            let pos = Point2::new(
                rng.gen_range(0.0, WIDTH as f32),
                rng.gen_range(0.0, HEIGHT as f32),
            );

            let vel = Vector2::new(
                rng.gen_range(-max_speed, max_speed),
                rng.gen_range(-max_speed, max_speed),
            );

            let radius = match rng.gen_range(0, 10) {
                0 => 5.0,
                _ => 3.0,
            };
            let follow: bool = match rng.gen_range(0, 5) {
                0 => true,
                _ => false,
            };

            let color = if follow {
                Color::new(1.0, 1.0, 0.0, 1.0)
            } else {
                Color::new(1.0, 0.0, 0.0, 1.0)
            };

            world
                .create_entity()
                .with(Vehicle {
                    pos,
                    vel: Vector2::zero(),
                    max_acc: max_acc,
                    steering_vel: vec![],
                    max_speed,
                })
                .with(Model {
                    size: radius,
                    pos: Point2::new(0.0, 0.0),
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
                    distance: radius * radius_mut,
                    weight: 2.0,
                })
                .with(SteeringVelocity {
                    enabled: !follow,
                    vel: vel,
                    weight: 1.0,
                })
                .build();
        }

        world.insert(cfg);

        let game = App { world };
        Ok(game)
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
        }
    }
}

struct SteeringWallsSystem;
impl<'a> System<'a> for SteeringWallsSystem {
    type SystemData = (WriteStorage<'a, Vehicle>, ReadStorage<'a, Wall>);

    fn run(&mut self, (mut vehicles, walls): Self::SystemData) {
        use specs::Join;

        for (vehicle) in (&mut vehicles).join() {
            for (wall) in (&walls).join() {}
        }
    }
}

struct MoveSystem;
impl<'a> System<'a> for MoveSystem {
    type SystemData = (ReadExpect<'a, GameTime>, WriteStorage<'a, Vehicle>);

    fn run(&mut self, (game_time, mut vehicles): Self::SystemData) {
        use specs::Join;

        for (vehicle) in (&mut vehicles).join() {
            let vehicle: &mut Vehicle = vehicle;
            let velocities = std::mem::replace(&mut vehicle.steering_vel, vec![]);
            let desired_velocity = velocities.into_iter().fold(Vector2::zero(), |a, b| a + b);
            let mut delta_velocity = desired_velocity - vehicle.vel;
            let step_max_acc = vehicle.max_acc * game_time.delta_time;
            if delta_velocity.magnitude() > step_max_acc {
                delta_velocity = delta_velocity.normalize() * step_max_acc;
            }
            vehicle.vel = vehicle.vel + delta_velocity;
            vehicle.pos = vehicle.pos + vehicle.vel * game_time.delta_time;
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
            }
        }
    }
}

struct UpdateModelPosSystem;
impl<'a> System<'a> for UpdateModelPosSystem {
    type SystemData = (ReadStorage<'a, Vehicle>, WriteStorage<'a, Model>);

    fn run(&mut self, (mobs, mut models): Self::SystemData) {
        use specs::Join;
        for (mob, model) in (&mobs, &mut models).join() {
            model.pos = mob.pos;
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

        if self.world.read_resource::<Cfg>().update_next {
            let mut dispatcher = DispatcherBuilder::new()
                .with(SteerArrivalSystem, "steering_arrival", &[])
                .with(SteeringSeparationSystem, "steering_separation", &[])
                .with(SteeringVelocitySystem, "steering_velocity", &[])
                .with(
                    MoveSystem,
                    "move",
                    &[
                        "steering_arrival",
                        "steering_separation",
                        "steering_velocity",
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
        let color_wall = Color::new(0.0, 1.0, 0.5, 1.0);

        {
            let models = &self.world.read_storage::<Model>();

            for (model) in (models).join() {
                // println!("{:?} drawing {:?} at {:?}", e, model, mov);
                let circle = graphics::Mesh::new_circle(
                    ctx,
                    // graphics::DrawMode::fill(),
                    graphics::DrawMode::stroke(1.0),
                    model.pos,
                    model.size,
                    0.1,
                    model.color,
                )?;
                graphics::draw(ctx, &circle, graphics::DrawParam::default())?;
            }
        }

        if cfg.draw_walls {
            for wall in (&self.world.read_storage::<Wall>()).join() {
                let mut mb = graphics::MeshBuilder::new();
                mb.line(
                    &[wall.pos, Point2::from_vec(wall.pos.to_vec() + wall.vec)],
                    wall.min_distance,
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
        for (arrival) in (&mut self.world.write_component::<SteeringArrival>()).join() {
            arrival.target_pos = (x, y).into();
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        match keycode {
            KeyCode::Space => {
                let cfg = &mut self.world.write_resource::<Cfg>();
                cfg.update_next = !cfg.update_next;
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
