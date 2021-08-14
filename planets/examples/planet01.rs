use commons::math::{lerp_2, map_value, p2, v2, P2, PI, TWO_PI, V2};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::{Color, DrawMode, DrawParam, Rect};
use ggez::{graphics, Context, ContextBuilder, GameResult};
use nalgebra::{Point2, Vector2};
use noise::{NoiseFn, Perlin};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;

#[derive(Debug)]
struct App {
    noise: Perlin,
    seed: f64,
}

impl App {
    pub fn new(ctx: &mut Context) -> GameResult<App> {
        Ok(App {
            noise: Default::default(),
            seed: 0.0,
        })
    }
}

fn draw_planet(app: &mut App, ctx: &mut Context, pos: P2, radius: f32) -> GameResult<()> {
    let mut rng: StdRng = SeedableRng::seed_from_u64(app.seed as u64);
    let mut mb = graphics::MeshBuilder::new();
    let segments = 50;
    let per_segment_angle = TWO_PI / segments as f32;
    let noise_radius = 2.0;

    let mut points = vec![];
    for i in 0..segments {
        let angle = i as f32 * per_segment_angle;

        let x = angle.cos() * noise_radius;
        let y = angle.sin() * noise_radius;

        let nois_pos = [x as f64, y as f64, app.seed];
        let noise = app.noise.get(nois_pos) as f32;
        let current_radius = map_value(noise, -1.0, 1.0, radius * 0.8, radius * 1.25);

        let x = angle.cos() * current_radius + pos.x;
        let y = angle.sin() * current_radius + pos.y;
        points.push(p2(x, y));

        mb.line(&[pos, p2(x, y)], 1.0, Color::new(0.0, 1.0, 0.0, 1.0));
    }

    mb.polygon(
        DrawMode::stroke(1.0),
        &points,
        Color::new(1.0, 0.0, 0.0, 1.0),
    )?;

    let mesh = mb.build(ctx)?;
    graphics::draw(ctx, &mesh, DrawParam::default())?;

    for i in 0..3 {
        let angle = rng.gen_range(0.0, TWO_PI);
        let distance = rng.gen_range(radius * 0.7, radius * 0.9);
        let size = radius - distance;

        let x = angle.cos() * distance + pos.x;
        let y = angle.sin() * distance + pos.y;

        let circle = graphics::Mesh::new_circle(
            ctx,
            DrawMode::stroke(1.0),
            p2(x, y),
            size,
            1.0,
            Color::new(0.0, 0.0, 1.0, 1.0),
        )?;

        graphics::draw(ctx, &circle, DrawParam::default())?;
    }

    Ok(())
}

impl EventHandler<ggez::GameError> for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if ggez::input::keyboard::is_key_pressed(ctx, KeyCode::Space) {
            self.seed = ggez::timer::ticks(ctx) as f64 / 100.0;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::Color::BLACK);
        draw_planet(self, ctx, p2(WIDTH / 2.0, HEIGHT / 2.0), HEIGHT * 0.4);
        graphics::present(ctx)
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {}

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {}
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
    event::run(ctx, event_loop, app);
}
