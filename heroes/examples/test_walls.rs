use cgmath::{prelude::*, vec2, VectorSpace, Point2, Vector2, EuclideanSpace};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::{graphics, timer, Context, ContextBuilder, GameResult};
use std::borrow::Borrow;

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;

type V2 = Vector2<f32>;

#[derive(Debug, Clone)]
struct Wall {
    p0: V2,
    p1: V2,
    min_distance: f32,
}

#[derive(Debug)]
struct App {
    point: V2,
    walls: Vec<Wall>,
}

impl App {
    pub fn new(ctx: &mut Context) -> GameResult<App> {
        Ok(App { point: vec2(300.0, 200.0), walls: vec![
            Wall {
                p0: vec2(100.0, 300.0),
                p1: vec2(600.0, 300.0),
                min_distance: 0.0
            }
        ] })
    }
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }



    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        let text = graphics::Text::new(format!("cfg: {:?}", self));
        graphics::draw(ctx, &text, (Point2::new(0.0, 0.0), graphics::WHITE))?;

        let wall_color = graphics::WHITE;
        let point_color = graphics::WHITE;

        for wall in &self.walls {
            let mesh = graphics::Mesh::new_line(ctx,
                &[
                    Point2::from_vec(wall.p0),
                    Point2::from_vec(wall.p1),
                ],
                1.0,
                wall_color
            )?;

            graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
        }

        {
            let mesh = graphics::Mesh::new_circle(ctx,
                                                graphics::DrawMode::fill(),
                                                Point2::from_vec(self.point),
                                                5.0,
                                                1.0,
                                                point_color
            )?;

            graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;

        }

        graphics::present(ctx)
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, x: f32, y: f32) {
       self.point = vec2(x, y);
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        if ggez::input::mouse::button_pressed(ctx, MouseButton::Left) {
            self.point = vec2(x, y);
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
        Ok(_) => {
            Ok(())
        }
        Err(e) => {
            println!("Error occured: {}", e);
            Err(e)
        }
    }
}
