use cgmath::Point2;
use cgmath::{prelude::*, vec2, vec3, Deg, Euler, Quaternion, Rad, Vector2, VectorSpace};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::Color;
use ggez::{graphics, timer, Context, ContextBuilder, GameResult};
use myelin_geometry::*;
use rand::prelude::StdRng;
use rand::{thread_rng, Rng, SeedableRng};
use specs::prelude::*;
use specs::{World, WorldExt};
use specs_derive::Component;
use std::ops::Deref;

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

#[derive(Clone, Debug, Component)]
struct Mob {
    pos: Point2<f32>,
    speed: f32,
    target_pos: Point2<f32>,
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
        // create world
        let mut world = World::new();
        world.register::<Model>();
        world.register::<Cfg>();
        world.register::<Mob>();
        world.register::<MovingArea>();

        world.insert(Cfg::new());
        world.insert(GameTime { delta_time: 0.01 });

        let border = 20.0;
        world.insert(MovingArea {
            polygons: vec![Polygon::try_new(vec![
                (border, border).into(),
                (WIDTH as f64 - border, border).into(),
                (WIDTH as f64 - border, HEIGHT as f64 - border).into(),
                (border, HEIGHT as f64 - border).into(),
            ])
            .unwrap()],
        });

        world
            .create_entity()
            .with(Mob {
                pos: Point2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0),
                speed: 50.0,
                target_pos: Point2::new(50.0, 100.0),
            })
            .with(Model {
                size: 3.0,
                pos: Point2::new(0.0, 0.0),
                color: Color::new(1.0, 0.0, 0.0, 1.0),
            })
            .build();

        let game = App { world };
        Ok(game)
    }
}

struct MobMoviementSystem;
impl<'a> System<'a> for MobMoviementSystem {
    type SystemData = (
        ReadExpect<'a, GameTime>,
        ReadExpect<'a, Cfg>,
        WriteStorage<'a, Mob>,
    );

    fn run(&mut self, (game_time, cfg, mut mobs): Self::SystemData) {
        use specs::Join;

        for (mob) in (&mut mobs).join() {
            let mob: &mut Mob = mob;
            let delta = mob.target_pos - mob.pos;
            let distance = delta.magnitude();
            if distance < 0.1 {
                println!("complete");
            } else {
                let dir = delta.normalize();
                let speed = mob.speed.min(distance * cfg.speed_reduction);
                let vel = dir * speed;
                let change = vel * game_time.delta_time;
                mob.pos = mob.pos + change;
                // println!("{:?} set vel {:?}", entity, movable);
            }
        }
    }
}

struct UpdateModelPos;
impl<'a> System<'a> for UpdateModelPos {
    type SystemData = (ReadStorage<'a, Mob>, WriteStorage<'a, Model>);

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
                .with(MobMoviementSystem, "mob_movement_system", &[])
                .with(UpdateModelPos, "update_model_pos", &["mob_movement_system"])
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
        let text = graphics::Text::new(format!("{:?}", cfg.deref()));
        graphics::draw(ctx, &text, (cgmath::Point2::new(0.0, 0.0), graphics::WHITE))?;

        graphics::present(ctx)
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
