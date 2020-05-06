use cgmath::Point2;
use cgmath::{prelude::*, vec2, vec3, Deg, Euler, Quaternion, Rad, Vector2, VectorSpace};
use ggez::conf::WindowMode;
use ggez::event::{self, Button, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::Color;
use ggez::{graphics, timer, Context, ContextBuilder, GameResult};
use myelin_geometry::*;
use rand::prelude::StdRng;
use rand::{thread_rng, Rng, SeedableRng};
use specs::prelude::*;
use specs::{World, WorldExt};
use specs_derive::Component;
use std::ops::Deref;
use utils::lerp_2;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

#[derive(Clone, Debug, Component)]
struct Cfg {
    update_next: bool,
    speed_reduction: f32,
}

impl Cfg {
    pub fn new() -> Self {
        Cfg {
            update_next: true,
            speed_reduction: 0.9,
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

fn to_point(p2: Point2<f32>) -> Point {
    (p2.x as f64, p2.y as f64).into()
}

impl MovingArea {
    pub fn is_valid(&self, point: Point2<f32>) -> bool {
        self.polygons
            .iter()
            .any(|polygon| polygon.contains_point(to_point(point)))
    }
}

#[derive(Clone, Debug, Component)]
struct Vehicle {
    pos: Point2<f32>,
    vel: Vector2<f32>,
    max_speed: f32,
}

#[derive(Clone, Debug, Component)]
struct SteeringSeparation {
    enabled: bool,
    distance: f32,
}

#[derive(Clone, Debug, Component)]
struct SteeringArrival {
    enabled: bool,
    target_pos: Point2<f32>,
    distance: f32,
}

#[derive(Clone, Debug, Component)]
struct Model {
    size: f32,
    pos: Point2<f32>,
    color: graphics::Color,
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

        world.insert(Cfg::new());
        world.insert(GameTime { delta_time: 0.01 });

        let border = 160.0;
        world.insert(MovingArea {
            polygons: vec![Polygon::try_new(vec![
                (border, border).into(),
                (WIDTH as f64 - border, border).into(),
                (WIDTH as f64 - border, HEIGHT as f64 - border).into(),
                (border, HEIGHT as f64 - border).into(),
            ])
            .unwrap()],
        });

        let max_speed = 100.0;

        for _ in 0..100 {
            let pos = Point2::new(
                rng.gen_range(0.0, WIDTH as f32),
                rng.gen_range(0.0, HEIGHT as f32),
            );

            let vel = Vector2::new(
                rng.gen_range(-max_speed, max_speed),
                rng.gen_range(-max_speed, max_speed),
            );

            let radius = rng.gen_range(1.0, 5.0);

            world
                .create_entity()
                .with(Vehicle {
                    pos,
                    vel,
                    max_speed,
                })
                .with(Model {
                    size: radius,
                    pos: Point2::new(0.0, 0.0),
                    color: Color::new(1.0, 0.0, 0.0, 1.0),
                })
                .with(SteeringArrival {
                    enabled: rng.gen(),
                    target_pos: Point2::new(300.0, 300.0),
                    distance: 50.0,
                })
                .with(SteeringSeparation {
                    enabled: rng.gen(),
                    distance: radius,
                })
                .build();
        }

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

                let min_distance = separation_a.distance + separation_b.distance;
                let vector = (vehicle_b.pos - vehicle_a.pos);
                let distance = vector.magnitude();

                if distance < min_distance {
                    changes.push((entity_a, vector * 1.0, distance));
                    changes.push((entity_b, vector, distance));
                }
            }
        }

        let vehicles = &mut vehicles;
        for (entity, vector, distance) in changes {
            let vehicle = vehicles.get_mut(entity).unwrap();
            vehicle.vel = vector;
        }
    }
}

struct MoveSystem;
impl<'a> System<'a> for MoveSystem {
    type SystemData = (ReadExpect<'a, GameTime>, WriteStorage<'a, Vehicle>);

    fn run(&mut self, (game_time, mut vehicles): Self::SystemData) {
        use specs::Join;

        for (vehicle) in (&mut vehicles).join() {
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
                vehicle.pos = arrival.target_pos;
                vehicle.vel = Vector2::zero();
            } else {
                let dir = delta.normalize();
                vehicle.vel = dir * speed;
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

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let delta = timer::delta(ctx).as_secs_f32();
        self.world.insert(GameTime { delta_time: delta });

        if self.world.read_resource::<Cfg>().update_next {
            let mut dispatcher = DispatcherBuilder::new()
                .with(SteerArrivalSystem, "steering_arrival", &[])
                .with(SteeringSeparationSystem, "steering_separation", &[])
                .with(
                    MoveSystem,
                    "move",
                    &["steering_arrival", "steering_separation"],
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

        {
            let models = &self.world.read_storage::<Model>();

            for (model) in (models).join() {
                // println!("{:?} drawing {:?} at {:?}", e, model, mov);
                let circle = graphics::Mesh::new_circle(
                    ctx,
                    graphics::DrawMode::fill(),
                    model.pos,
                    model.size,
                    0.1,
                    model.color,
                )?;
                graphics::draw(ctx, &circle, graphics::DrawParam::default())?;
            }
        }

        {
            let moving_area = &self.world.read_resource::<MovingArea>();

            for polygon in &moving_area.polygons {
                let points: Vec<Point2<f32>> = polygon
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
    window_mode.resizable = true;

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
