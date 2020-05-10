use cgmath::{prelude::*, vec2, vec3, Deg, Euler, Quaternion, Rad, Vector2, VectorSpace};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::{graphics, timer, Context, ContextBuilder, GameResult};
use itertools::Itertools;
use obj::raw::object::Point;
use obj::{load_obj, Obj, Position};
use std::fs::File;
use std::io::BufReader;
use utils::lerp_2;

const WIDTH: f32 = 1600.0;
const HEIGHT: f32 = 1200.0;

struct App {
    obj: Obj<Position>,
    pos: cgmath::Point2<f32>,
    scale: f32,
    disply_points: bool,
}

impl App {
    pub fn new(ctx: &mut Context) -> GameResult<App> {
        let input = BufReader::new(File::open("steering/resources/navmesh.obj").unwrap());
        let obj: Obj<Position> = load_obj(input).unwrap();

        let game = App {
            obj,
            pos: cgmath::Point2::new(65.0, 32.0),
            scale: 14.0,
            disply_points: false,
        };
        Ok(game)
    }
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        fn convert(coords: [f32; 3]) -> cgmath::Point2<f32> {
            (coords[0], coords[2]).into()
        }

        // TOOD this is just horrible, simplify
        for i in (0..self.obj.indices.len()).step_by(3) {
            let i0 = self.obj.indices[i + 0] as usize;
            let i1 = self.obj.indices[i + 1] as usize;
            let i2 = self.obj.indices[i + 2] as usize;
            let v0 = convert(self.obj.vertices[i0].position);
            let v1 = convert(self.obj.vertices[i1].position);
            let v2 = convert(self.obj.vertices[i2].position);

            let mut vertices = [v0, v1, v2, v0];
            for i in 0..vertices.len() {
                vertices[i] += self.pos.to_vec();
                vertices[i] *= self.scale;
            }

            if self.disply_points {
                let text =
                    graphics::Text::new(format!("{} ({},{})", i0, vertices[0].x, vertices[0].y));
                graphics::draw(ctx, &text, (vertices[0], graphics::WHITE))?;

                let text =
                    graphics::Text::new(format!("{} ({},{})", i1, vertices[1].x, vertices[1].y));
                graphics::draw(ctx, &text, (vertices[1], graphics::WHITE))?;

                let text =
                    graphics::Text::new(format!("{} ({},{})", i2, vertices[2].x, vertices[2].y));
                graphics::draw(ctx, &text, (vertices[2], graphics::WHITE))?;

                println!(
                    "{:?} {:?} {:?} => {:?} {:?} {:?}",
                    i0, i1, i2, vertices[0], vertices[1], vertices[2]
                );
            }

            let mesh = graphics::Mesh::new_line(ctx, &vertices, 1.0, graphics::WHITE)?;
            graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
        }

        let text = graphics::Text::new(format!("pos: {:?} scale {:?}", self.pos, self.scale));
        graphics::draw(ctx, &text, (cgmath::Point2::new(0.0, 0.0), graphics::WHITE))?;

        graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        let delta = timer::delta(ctx);
        let speed = 1000.0;
        let scale_speed = 5.0;

        match keycode {
            KeyCode::Up => {
                self.pos.y -= speed * delta.as_secs_f32();
            }
            KeyCode::Down => {
                self.pos.y += speed * delta.as_secs_f32();
            }
            KeyCode::Left => {
                self.pos.x += speed * delta.as_secs_f32();
            }
            KeyCode::Right => {
                self.pos.x -= speed * delta.as_secs_f32();
            }
            KeyCode::W => {
                self.scale += scale_speed * delta.as_secs_f32();
            }
            KeyCode::S => {
                self.scale -= scale_speed * delta.as_secs_f32();
            }
            _ => {}
        }
    }
}

fn to_2d(v: &[f32; 3]) -> (f32, f32) {
    (v[0], v[2])
}

/// NOT WORKING
fn normalize(obj: &mut Obj) {
    let mut min_max: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
    for v in &obj.vertices {
        let (x, y) = to_2d(&v.position);
        min_max[0] = min_max[0].min(x);
        min_max[1] = min_max[1].min(y);
        min_max[2] = min_max[2].max(x);
        min_max[3] = min_max[3].max(y);
    }

    for v in &mut obj.vertices {
        let (x, y) = to_2d(&v.position);
        let nx = lerp_2(min_max[0], min_max[2], 0.0, WIDTH, x);
        let ny = lerp_2(min_max[1], min_max[3], 0.0, HEIGHT, y);
        v.position[0] = nx;
        v.position[2] = ny;
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
