use cgmath::Point2;
use cgmath::{prelude::*, vec2, vec3, Deg, Euler, Quaternion, Rad, Vector2, VectorSpace};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::{graphics, timer, Context, ContextBuilder, GameResult};
use rand::prelude::StdRng;
use rand::{thread_rng, Rng, SeedableRng};
use specs::prelude::*;
use specs::{World, WorldExt};
use specs_derive::Component;
use std::ops::Deref;

#[derive(Clone, Debug, Component)]
struct Cfg {
    gravity: f32,
    update_next: bool,
}

#[derive(Clone, Debug, Component)]
struct Model {
    size: f32,
    pos: Point2<f32>,
    color: graphics::Color,
}

#[derive(Clone, Debug, Component)]
struct OrbtialBody {
    mass: f32,
    pos: Vector2<f32>,
    vel: Vector2<f32>,
    force: Vector2<f32>,
    forces: Vec<Vector2<f32>>,
}

struct App {
    world: World,
}

impl App {
    pub fn new(_ctx: &mut Context) -> GameResult<App> {
        // create world
        let mut world = World::new();
        world.register::<Model>();
        world.register::<Cfg>();
        world.register::<OrbtialBody>();

        world.insert(Cfg {
            gravity: 10.0,
            update_next: true,
        });

        // add elements
        {
            world
                .create_entity()
                .with(Model {
                    size: 6.0,
                    pos: Point2::new(0.0, 0.0),
                    color: graphics::Color::WHITE,
                })
                .with(OrbtialBody {
                    mass: 100000.0,
                    pos: vec2(400.0, 300.0),
                    vel: vec2(0.0, 0.0),
                    force: vec2(0.0, 0.0),
                    forces: vec![],
                })
                .build();

            // let seed: u64 = 0;
            // let rng: StdRng = SeedableRng::from_seed_u64(seed);
            let mut rng = thread_rng();

            for _ in 0..100 {
                world
                    .create_entity()
                    .with(Model {
                        size: 2.0,
                        pos: Point2::new(0.0, 0.0),
                        color: graphics::Color::new(1.0, 0.0, 0.0, 1.0),
                    })
                    .with(OrbtialBody {
                        mass: rng.gen_range(10.0, 100.0),
                        pos: vec2(rng.gen_range(10.0, 590.0), rng.gen_range(10.0, 590.0)),
                        vel: vec2(rng.gen_range(-50.0, 50.0), rng.gen_range(-50.0, 5.0)),
                        force: vec2(0.0, 0.0),
                        forces: vec![],
                    })
                    .build();
            }

            // world
            //     .create_entity()
            //     .with(Model {
            //         size: 2.0,
            //         pos: Point2::new(0.0, 0.0),
            //         color: graphics::Color::new(1.0, 0.0, 0.0, 1.0),
            //     })
            //     .with(OrbtialBody {
            //         mass: 20.0,
            //         pos: vec2(200.0, 300.0),
            //         vel: vec2(0.0, -40.0),
            //         force: vec2(0.0, 0.0),
            //         forces: vec![],
            //     })
            //     .build();
            //
            // world
            //     .create_entity()
            //     .with(Model {
            //         size: 2.0,
            //         pos: Point2::new(0.0, 0.0),
            //         color: graphics::Color::new(1.0, 0.0, 0.0, 1.0),
            //     })
            //     .with(OrbtialBody {
            //         mass: 100.0,
            //         pos: vec2(500.0, 300.0),
            //         vel: vec2(0.0, 90.0),
            //         force: vec2(0.0, 0.0),
            //         forces: vec![],
            //     })
            //     .build();
        }

        let game = App { world };

        Ok(game)
    }
}

fn update_orbits(delta_time: f32, world: &mut World) {
    {
        let gravity = world.read_resource::<Cfg>().gravity;

        let mut orbits = world.write_storage::<OrbtialBody>();
        let orbits_slice = orbits.as_mut_slice();

        for i in 0..orbits_slice.len() {
            let mut a = orbits_slice[i].clone();
            a.forces.clear();

            for j in 0..orbits_slice.len() {
                if i == j {
                    continue;
                }

                let b = &orbits_slice[j];
                let delta = (b.pos - a.pos);
                let distance = delta.magnitude();
                let dir = delta.normalize();
                let force_n = (gravity * a.mass * b.mass) / (distance * distance);
                let force = dir * force_n;
                // println!(
                //     "{:?} {:?} {:?} {:?} {:?} {:?} {:?}",
                //     a, b, delta, distance, dir, force_n, force
                // );
                a.forces.push(force);
            }

            a.force = vec2(0.0, 0.0);
            for force in &a.forces {
                a.force = a.force + force;
            }

            let acc = a.force / a.mass;
            a.vel += acc * delta_time;
            a.pos += a.vel * delta_time;

            // println!("{:?}", a);
            orbits_slice[i] = a;
        }
    }
}

fn update_models_from_orbits(world: &mut World) {
    let models = &mut world.write_storage::<Model>();
    let orbits = &world.read_storage::<OrbtialBody>();

    for (model, orbit) in (models, orbits).join() {
        model.pos = cgmath::Point2::new(orbit.pos.x, orbit.pos.y);
    }
}

impl EventHandler<ggez::GameError> for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let delta = timer::delta(ctx).as_secs_f32();
        if self.world.read_resource::<Cfg>().update_next {
            // self.world.write_resource::<Cfg>().update_next = false;
            update_orbits(delta, &mut self.world);
        }
        update_models_from_orbits(&mut self.world);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::Color::BLACK);

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

        let cfg = &self.world.read_resource::<Cfg>();
        let text = graphics::Text::new(format!("{:?}", cfg.deref()));
        graphics::draw(
            ctx,
            &text,
            (cgmath::Point2::new(0.0, 0.0), graphics::Color::WHITE),
        )?;

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
    event::run(ctx, event_loop, app);
}

#[cfg(test)]
mod test {
    use super::*;
}
