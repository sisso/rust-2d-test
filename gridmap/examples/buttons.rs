use commons::graphics::{GuiButton, GuiManage};
use commons::math::{p2, v2, P2, V2};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::{Color, DrawMode, DrawParam, Rect};
use ggez::{graphics, Context, ContextBuilder, GameResult};
use nalgebra::{Point2, Vector2};

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;

#[derive(Debug)]
struct App {
    gui: GuiManage,
}

impl App {
    pub fn new(ctx: &mut Context) -> GameResult<App> {
        let mut manage = GuiManage::new();

        let button_color = Color::new(1.0, 1.0, 1.0, 1.0);
        let button_color_background = Color::new(0.6, 0.6, 0.6, 1.0);
        let button_color_hover = Color::new(0.9, 0.5, 0.5, 1.0);
        let button_color_click = Color::new(0.5, 0.9, 0.6, 1.0);

        let labels = ["Start", "Stop", "End"];
        for (i, l) in labels.iter().enumerate() {
            manage.push(GuiButton {
                uid: i as u32,
                bounds: Rect::new(20.0 + i as f32 * 100.0, 20.0, 60.0, 40.0),
                text: l.to_string(),
                hover: false,
                click: false,
                color: button_color,
                color_background: button_color_background,
                color_hover: button_color_hover,
                color_click: button_color_click,
            });
        }

        Ok(App { gui: manage })
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

        self.gui.draw(ctx);
        graphics::present(ctx)
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            self.gui.on_mouse_down(Point2::new(x, y));
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            match self.gui.on_mouse_up(Point2::new(x, y)) {
                Some(button_uid) => println!("clicked at {:?}", button_uid),
                _ => {}
            }
        }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.gui.on_mouse_move(Point2::new(x, y));
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
