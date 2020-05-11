use cgmath::{prelude::*, vec2, Point2, Vector2, VectorSpace};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::Color;
use ggez::{graphics, timer, Context, ContextBuilder, GameResult};
use gridmap::ShipDesign;

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;

type P2 = cgmath::Point2<f32>;

#[derive(Debug)]
struct App {
    design: ShipDesign,
}

impl App {
    pub fn new(ctx: &mut Context) -> GameResult<App> {
        Ok(App {
            design: ShipDesign::new(),
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
        let text = graphics::Text::new(format!("cfg: {:?}", self));
        graphics::draw(ctx, &text, (Point2::new(0.0, 0.0), graphics::WHITE))?;

        draw_grid(
            ctx,
            WIDTH,
            HEIGHT,
            self.design.size.width,
            self.design.size.height,
        )?;

        graphics::present(ctx)
    }
}

fn draw_grid(
    ctx: &mut Context,
    screen_width: f32,
    screen_height: f32,
    width: u32,
    height: u32,
) -> GameResult<()> {
    let border = 80.0;
    let grid_color = graphics::WHITE;
    let grid_width = 1.0;
    let grid_size = (screen_height.min(screen_width) as f32 - border) / width.max(height) as f32;
    let max_x = grid_size * width as f32 + border;
    let max_y = grid_size * height as f32 + border;

    // vertical lines
    for i in (0..width + 1) {
        let x = border + i as f32 * grid_size;
        let p0 = Point2::new(x, border);
        let p1 = Point2::new(x, max_y);
        draw_line(ctx, p0, p1, grid_color, grid_width)?;
    }

    // horizontal lines
    for i in 0..(height + 1) {
        let y = border + i as f32 * grid_size;
        let p0 = Point2::new(border, y);
        let p1 = Point2::new(max_x, y);
        draw_line(ctx, p0, p1, grid_color, grid_width)?;
    }

    Ok(())
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
