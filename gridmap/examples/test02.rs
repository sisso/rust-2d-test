use cgmath::{prelude::*, vec2, Point2, Vector2, VectorSpace};
use commons::math::V2;
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::{Color, DrawParam, Rect};
use ggez::{graphics, timer, Context, ContextBuilder, GameResult};
use gridmap::{Cfg, Repository, ShipDesign};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;
const CELL_SIZE: f32 = 15.0;

type P2 = cgmath::Point2<f32>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppCfg {}

#[derive(Debug)]
struct Gui {
    bottom_panel: commons::graphics::GuiManage,
}

#[derive(Debug)]
struct App {
    cfg: AppCfg,
    screen_size: Rect,
    design: ShipDesign,
    repository: Repository,
    gui: Gui,
}

impl App {
    pub fn new(ctx: &mut Context) -> GameResult<App> {
        let (app_cfg, cfg) = load_cfg("gridmap/resources/config.json")?;
        let repository = Repository::new(cfg);

        let mut buttons = commons::graphics::GuiManage::new();
        add_panel_buttons(&mut buttons, &repository, graphics::screen_coordinates(ctx));

        let gui = Gui {
            bottom_panel: buttons,
        };

        Ok(App {
            cfg: app_cfg,
            screen_size: graphics::screen_coordinates(ctx),
            design: ShipDesign::new(),
            repository,
            gui: gui,
        })
    }
}

fn add_panel_buttons(
    gui: &mut commons::graphics::GuiManage,
    repository: &Repository,
    screen_size: Rect,
) -> GameResult<()> {
    let button_rect = Rect::new(0.0, 0.0, 60.0, 40.0);
    let panel_rect = Rect::new(
        screen_size.x,
        screen_size.h - button_rect.h,
        screen_size.w,
        button_rect.h,
    );

    for (i, component) in repository.list_components().enumerate() {
        let desl_x = i as f32 * button_rect.w;

        let rect = Rect::new(
            panel_rect.x + desl_x,
            panel_rect.y,
            button_rect.w,
            button_rect.h,
        );

        let button_color = Color::new(1.0, 1.0, 1.0, 1.0);
        let button_color_background = Color::new(0.6, 0.6, 0.6, 1.0);
        let button_color_hover = Color::new(0.9, 0.5, 0.5, 1.0);
        let button_color_click = Color::new(0.5, 0.9, 0.6, 1.0);

        let button = commons::graphics::GuiButton {
            uid: i as u32,
            bounds: rect,
            text: component.code.clone(),
            hover: false,
            click: false,
            color: button_color,
            color_background: button_color_background,
            color_hover: button_color_hover,
            color_click: button_color_click,
        };

        gui.push(button);
    }

    Ok(())
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

        // screen
        {
            graphics::set_screen_coordinates(ctx, self.screen_size);
            draw_grid(
                ctx,
                Point2::new(80.0, 80.0),
                CELL_SIZE,
                self.design.size.width,
                self.design.size.height,
            )?;
        }

        let screen_size = Rect::new(0.0, 0.0, WIDTH, HEIGHT);

        // gui
        {
            graphics::set_screen_coordinates(ctx, screen_size);
            let text = graphics::Text::new(format!("cfg: {:?}", self));
            graphics::draw(ctx, &text, (Point2::new(0.0, 0.0), graphics::WHITE))?;
        }

        // gui buttons
        {
            self.gui.bottom_panel.draw(ctx);
        }

        // show
        graphics::present(ctx)
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            self.gui.bottom_panel.on_mouse_down(Point2::new(x, y));
        }
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            self.gui.bottom_panel.on_mouse_up(Point2::new(x, y));
        }
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        if ggez::input::mouse::button_pressed(ctx, MouseButton::Right) {
            move_screen(&mut self.screen_size, Vector2::new(-dx, -dy));
        } else {
            self.gui.bottom_panel.on_mouse_move(Point2::new(x, y));
        }
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
        zoom_screen(&mut self.screen_size, -y);
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        let screen_speed = 20.0;

        match keycode {
            KeyCode::W => {
                move_screen(&mut self.screen_size, Vector2::new(0.0, -screen_speed));
            }
            KeyCode::S => {
                move_screen(&mut self.screen_size, Vector2::new(0.0, screen_speed));
            }
            KeyCode::A => {
                move_screen(&mut self.screen_size, Vector2::new(screen_speed, 0.0));
            }
            KeyCode::D => {
                move_screen(&mut self.screen_size, Vector2::new(-screen_speed, 0.0));
            }
            _ => {}
        }
    }
}

fn move_screen(rect: &mut Rect, v: V2) {
    rect.x += v.x;
    rect.y += v.y;
}

fn zoom_screen(rect: &mut Rect, amount: f32) {
    let scale = if amount > 0.0 {
        1.1
    } else if amount < 0.0 {
        0.9
    } else {
        1.0
    };
    rect.scale(scale, scale);
}

fn draw_grid(
    ctx: &mut Context,
    pos: P2,
    grid_size: f32,
    width: u32,
    height: u32,
) -> GameResult<()> {
    let grid_color = graphics::WHITE;
    let grid_width = 1.0;
    let max_x = grid_size * width as f32;
    let max_y = grid_size * height as f32;

    // vertical lines
    for i in (0..width + 1) {
        let x = i as f32 * grid_size;
        let p0 = pos + Vector2::new(x, 0.0);
        let p1 = pos + Vector2::new(x, max_y);
        draw_line(ctx, p0, p1, grid_color, grid_width)?;
    }

    // horizontal lines
    for i in 0..(height + 1) {
        let y = i as f32 * grid_size;
        let p0 = pos + Vector2::new(0.0, y);
        let p1 = pos + Vector2::new(max_x, y);
        draw_line(ctx, p0, p1, grid_color, grid_width)?;
    }

    Ok(())
}

fn load_cfg(file_path: &str) -> GameResult<(AppCfg, Cfg)> {
    let body = std::fs::read_to_string(file_path).unwrap();
    let value: Value = serde_json::from_str(body.as_str()).unwrap();

    Ok((
        serde_json::from_value(value["gui"].clone()).unwrap(),
        serde_json::from_value(value["game"].clone()).unwrap(),
    ))
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
