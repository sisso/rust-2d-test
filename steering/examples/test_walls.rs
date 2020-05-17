extern crate steerning;
use steerning::*;

use cgmath::{prelude::*, vec2, EuclideanSpace, Point2, Vector2, VectorSpace};
use commons::math::*;
use geo::Point;
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::Color;
use ggez::{graphics, timer, Context, ContextBuilder, GameResult};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;

#[derive(Debug, Clone)]
struct Wall {
    /// position of the wall
    pos: P2,
    /// vector that represent wall direction and size
    vec: V2,
    /// min distance until get affected by the wall
    min_distance: f32,
}

#[derive(Debug)]
struct App {
    point: P2,
    walls: Vec<Wall>,
}

impl App {
    pub fn new(ctx: &mut Context) -> GameResult<App> {
        let mut rng: StdRng = SeedableRng::seed_from_u64(0);

        let mut walls = vec![];
        {
            for i in 0..10 {
                let wall = Wall {
                    pos: Point2::new(rng.gen_range(0.0, WIDTH), rng.gen_range(0.0, HEIGHT)),
                    vec: vec2(rng.gen_range(0.0, WIDTH), rng.gen_range(0.0, HEIGHT)),
                    min_distance: 50.0,
                };

                walls.push(wall);
            }
        }
        {
            // walls = vec![Wall {
            //     pos: Point2::new(100.0, 300.0),
            //     vec: Vector2::new(500.0, 0.0),
            //     min_distance: 50.0,
            // }];
        }

        Ok(App {
            point: Point2::new(300.0, 200.0),
            walls: walls,
        })
    }
}

fn draw_line(ctx: &mut Context, p0: P2, p1: P2, color: Color, width: f32) -> GameResult<()> {
    let mesh = graphics::Mesh::new_line(ctx, &[p0, p1], width, color)?;

    graphics::draw(ctx, &mesh, graphics::DrawParam::default())
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        let mut debug_text = format!("fps {:.0}", timer::fps(ctx));

        let wall_color = graphics::WHITE;
        let proj_miss_color = Color::new(0.0, 1.0, 0.0, 1.0);
        let proj_hit_color = Color::new(1.0, 0.0, 0.0, 1.0);
        let point_color = graphics::WHITE;

        for wall in &self.walls {
            draw_line(
                ctx,
                wall.pos,
                Point2::from_vec(wall.vec + wall.pos.to_vec()),
                wall_color,
                1.0,
            );
        }

        {
            let mesh = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                self.point,
                5.0,
                1.0,
                point_color,
            )?;

            graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
        }

        {
            for wall in &self.walls {
                if let Some(vec) =
                    compute_vector_from_point_to_segment(wall.pos, wall.vec, self.point)
                {
                    let color = if vec.magnitude() > wall.min_distance {
                        proj_miss_color
                    } else {
                        proj_hit_color
                    };

                    draw_line(ctx, self.point, self.point + vec, color, 1.0);
                }
            }
        }

        {
            let mut text = graphics::Text::new(debug_text);
            graphics::draw(ctx, &text, (Point2::new(0.0, 0.0), graphics::WHITE))?;
        }

        graphics::present(ctx)
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        x: f32,
        y: f32,
    ) {
        self.point = Point2::new(x, y);
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        if ggez::input::mouse::button_pressed(ctx, MouseButton::Left) {
            self.point = Point2::new(x, y);
        }
    }
}

fn main() -> GameResult<()> {
    // Make a Context.
    let mut window_mode: WindowMode = Default::default();
    window_mode.width = WIDTH;
    window_mode.height = HEIGHT;

    let (mut ctx, mut event_loop) = ContextBuilder::new("template", "Sisso")
        .window_mode(window_mode)
        .build()
        .unwrap();

    let mut app = App::new(&mut ctx)?;

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut app) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Error occured: {}", e);
            Err(e)
        }
    }
}
