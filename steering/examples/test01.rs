use commons::math::*;
use ggez::conf::WindowMode;
use ggez::event::{self, Button, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::Color;
use ggez::{graphics, timer, Context, ContextBuilder, GameError, GameResult};
use nalgebra::{Point2, Vector2};
use specs::prelude::*;
use specs::{World, WorldExt};
use specs_derive::Component;
use std::ops::Deref;
use steerning::steerning::components::*;
use steerning::steerning::*;

struct App {
    update_next: bool,
    world: World,
}

impl App {
    pub fn new(ctx: &mut Context, cfg: Cfg) -> GameResult<App> {
        let mut world = create_world(cfg)?;
        initialize_world(&mut world);
        let game = App {
            update_next: true,
            world,
        };
        Ok(game)
    }

    pub fn reload(&mut self) -> GameResult<()> {
        let cfg = load_cfg()?;
        let mut world = create_world(cfg)?;
        initialize_world(&mut world);
        self.world = world;
        Ok(())
    }
}

fn draw_circle(
    ctx: &mut Context,
    pos: P2,
    size: f32,
    color: Color,
    width: f32,
    fill: bool,
) -> GameResult<()> {
    let mode = if fill {
        graphics::DrawMode::fill()
    } else {
        graphics::DrawMode::stroke(width)
    };

    let circle = graphics::Mesh::new_circle(ctx, mode, pos, size, 0.1, color)?;

    graphics::draw(ctx, &circle, graphics::DrawParam::default())
}

fn draw_line(ctx: &mut Context, p0: P2, p1: P2, color: Color, width: f32) -> GameResult<()> {
    let mesh = graphics::Mesh::new_line(ctx, &[p0, p1], width, color)?;
    graphics::draw(ctx, &mesh, graphics::DrawParam::default())
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.update_next {
            let delta = timer::delta(ctx).as_secs_f32();
            steerning::steerning::run(delta, &mut self.world);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        let color_wall = Color::new(0.0, 1.0, 0.5, 0.5);

        {
            for wall in (&self.world.read_storage::<Wall>()).join() {
                let mut mb = graphics::MeshBuilder::new();
                mb.line(
                    &[wall.pos, Point2::from(wall.pos.clone().coords + wall.vec)],
                    wall.min_distance * 2.0,
                    color_wall,
                )?;
                let mesh = mb.build(ctx)?;
                graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
            }
        }

        // {
        //     let moving_area = &self.world.read_resource::<MovingArea>();
        //
        //     for polygon in &moving_area.polygons {
        //         let points: Vec<P2> = polygon
        //             .vertices()
        //             .iter()
        //             .map(|point| Point2::new(point.x as f32, point.y as f32))
        //             .collect();
        //
        //         let walking_area_color = Color::new(1.0, 1.0, 1.0, 1.0);
        //         let mesh = graphics::Mesh::new_polygon(
        //             ctx,
        //             graphics::DrawMode::stroke(1.0),
        //             points.as_slice(),
        //             walking_area_color,
        //         )?;
        //
        //         graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
        //     }
        // }

        {
            let models = &self.world.read_storage::<Model>();

            for (model) in (models).join() {
                draw_circle(ctx, model.pos, model.size, model.color, 1.0, false)?;
                draw_line(
                    ctx,
                    model.pos,
                    Point2::from(model.pos.coords.clone() + model.dir * model.size),
                    model.color,
                    1.0,
                )?;
            }
        }

        {
            let lines = take_debug_lines(&mut self.world);
            for (a, b, color) in lines {
                draw_line(ctx, a, b, color, 1.0).unwrap();
            }
        }

        let text = graphics::Text::new(format!("ftps: {}", ggez::timer::fps(ctx) as i32,));
        graphics::draw(ctx, &text, (Point2::new(0.0, 0.0), graphics::WHITE))?;

        graphics::present(ctx)
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            move_to(&mut self.world, Point2::new(x, y)).unwrap();
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        match keycode {
            KeyCode::Space => {
                self.update_next = !self.update_next;
            }
            KeyCode::Return => {
                if let Err(e) = self.reload() {
                    println!("fail to relaod config file: {:?}", e);
                }
            }
            _ => {}
        }
    }
}

fn main() -> GameResult<()> {
    let cfg = load_cfg()?;

    // Make a Context.
    let mut window_mode: WindowMode = Default::default();
    window_mode.width = cfg.screen_width;
    window_mode.height = cfg.screen_height;

    let (mut ctx, mut event_loop) = ContextBuilder::new("my_game", "Cool Game Author")
        .window_mode(window_mode)
        .build()
        .expect("aieee, could not create ggez context!");

    let mut app = App::new(&mut ctx, cfg)?;

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut app) {
        Ok(_) => {
            println!("Exited cleanly.");
            Ok(())
        }
        Err(e) => {
            println!("Error occurred: {}", e);
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
