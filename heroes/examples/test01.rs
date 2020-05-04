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

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

#[derive(Clone, Debug, Component)]
struct Cfg {
    update_next: bool,
    speed_reduction: f32,
    target: Point2<f32>,
}

impl Cfg {
    pub fn new() -> Self {
        Cfg {
            update_next: true,
            speed_reduction: 0.9,
            target: Point2::new(0.0, 0.0),
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
struct Mob {
    pos: Point2<f32>,
    speed: f32,
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

        // {
        //     let paths: Vec<Vec<f32>> = vec![
        //         vec![
        //             0.0, 287.73959, 3.96875, -17.19792, 26.458333, -3.96875, -1.322916, 19.84375,
        //             -15.875, 10.58334,
        //         ],
        //         vec![
        //             30.427083, 266.57292, 10.583333, 1.32291, 6.614583, 25.13542, -3.96875,
        //             2.64583, 31.75, 2.64583, -2.645834, -9.26041,
        //         ],
        //     ];
        //
        //     let mut min: [f32; 2] = [0.0, 0.0];
        //     let mut max: [f32; 2] = [0.0, 0.0];
        //     let mut polygons: Vec<Polygon> = vec![];
        //
        //     for path in paths {
        //         let mut vertices: Vec<Point> = vec![];
        //
        //         for i in 0..(path.len() / 2) {
        //             let x = path[i * 2];
        //             let y = path[i * 2 + 1];
        //
        //             max[0] = max[0].max(x);
        //             max[1] = max[1].max(y);
        //             min[0] = min[0].min(x);
        //             min[1] = min[1].min(y);
        //
        //             vertices.push((x as f64, y as f64).into());
        //         }
        //
        //         vertices.push(vertices[0].clone());
        //
        //         let polygon = match Polygon::try_new(vertices.clone()) {
        //             Ok(polygon) => polygon,
        //             Err(e) => panic!("fail to generate a polygon {:?} from {:?}", e, vertices),
        //         };
        //         polygons.push(polygon);
        //     }
        //
        //     world.insert(MovingArea { polygons });
        // }

        world
            .create_entity()
            .with(Mob {
                pos: Point2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0),
                speed: 200.0,
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
        ReadExpect<'a, MovingArea>,
    );

    fn run(&mut self, (game_time, cfg, mut mobs, moving_area): Self::SystemData) {
        use specs::Join;

        for (mob) in (&mut mobs).join() {
            let mob: &mut Mob = mob;
            let delta = cfg.target - mob.pos;
            let distance = delta.magnitude();
            if distance < 0.1 {
                println!("complete");
            } else {
                let dir = delta.normalize();
                let speed = mob.speed.min(distance * cfg.speed_reduction);
                let vel = dir * speed;
                let change = vel * game_time.delta_time;
                let new_pos = mob.pos + change;

                if moving_area.is_valid(new_pos) {
                    mob.pos = new_pos;
                }

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

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if MouseButton::Right == button {
            self.world.write_resource::<Cfg>().target = (x, y).into();
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
