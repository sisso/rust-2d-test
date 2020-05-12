use cgmath::{prelude::*, Point2, Vector2, VectorSpace};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::{Color, DrawMode, DrawParam, Rect};
use ggez::{graphics, Context, ContextBuilder, GameResult};

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;

type P2 = Point2<f32>;
type V2 = Vector2<f32>;

#[derive(Debug, Clone)]
pub struct GuiButton {
    pub uid: u32,
    pub bounds: Rect,
    pub text: String,
    pub hover: bool,
    pub click: bool,
    pub color: Color,
    pub color_background: Color,
    pub color_hover: Color,
    pub color_click: Color,
}

impl GuiButton {
    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let mut builder = graphics::MeshBuilder::new();

        let color = if self.click {
            self.color_click
        } else if self.hover {
            self.color_hover
        } else {
            self.color
        };

        builder.rectangle(DrawMode::fill(), self.bounds, self.color_background);
        builder.rectangle(DrawMode::stroke(1.0), self.bounds, color);

        let mesh = builder.build(ctx)?;
        graphics::draw(ctx, &mesh, DrawParam::default());

        let text = graphics::Text::new(format!("{}", self.text));
        let border_x = (self.bounds.w - text.width(ctx) as f32) / 2.0;
        let border_y = (self.bounds.h - text.height(ctx) as f32) / 2.0;
        let pos = Point2::new(self.bounds.x + border_x, self.bounds.y + border_y);

        graphics::draw(ctx, &text, (pos, graphics::WHITE))?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct GuiManage {
    buttons: Vec<GuiButton>,
}

impl GuiManage {
    pub fn new() -> Self {
        GuiManage { buttons: vec![] }
    }

    pub fn on_mouse_move(&mut self, pos: P2) {
        for button in &mut self.buttons {
            button.hover = button.bounds.contains(pos);
        }
    }

    pub fn on_mouse_down(&mut self, pos: P2) {
        for button in &mut self.buttons {
            button.click = button.bounds.contains(pos);
        }
    }

    pub fn on_mouse_up(&mut self, pos: P2) -> Option<u32> {
        let mut selected = None;
        for button in &mut self.buttons {
            if button.click {
                if button.bounds.contains(pos) {
                    selected = Some(button.uid);
                }
                button.click = false;
            }
        }
        selected
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        for button in &self.buttons {
            button.draw(ctx)?;
        }

        Ok(())
    }

    pub fn push(&mut self, button: GuiButton) -> usize {
        self.buttons.push(button);
        self.buttons.len() - 1
    }
}

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
