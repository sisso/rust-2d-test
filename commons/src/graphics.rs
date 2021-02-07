use crate::math::{p2, v2, P2, V2};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::{Color, DrawMode, DrawParam, Rect};
use ggez::{graphics, Context, GameResult};

#[derive(Debug, Clone)]
pub struct GuiButton<ID: Copy> {
    pub uid: ID,
    pub bounds: Rect,
    pub text: String,
    pub hover: bool,
    pub click: bool,
    pub color: Color,
    pub color_background: Color,
    pub color_hover: Color,
    pub color_click: Color,
}

impl<ID: Copy> GuiButton<ID> {
    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
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
        graphics::draw(ctx, &mesh, DrawParam::default())?;

        let text = graphics::Text::new(format!("{}", self.text));
        let border_x = (self.bounds.w - text.width(ctx) as f32) / 2.0;
        let border_y = (self.bounds.h - text.height(ctx) as f32) / 2.0;
        let pos = p2(self.bounds.x + border_x, self.bounds.y + border_y);

        graphics::draw(ctx, &text, (pos, graphics::WHITE))?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct GuiManage<ID: Copy> {
    buttons: Vec<GuiButton<ID>>,
}

impl<ID: Copy> GuiManage<ID> {
    pub fn new() -> Self {
        GuiManage { buttons: vec![] }
    }

    pub fn on_mouse_move(&mut self, pos: P2) -> bool {
        let mut result = false;

        for button in &mut self.buttons {
            button.hover = button.bounds.contains(pos);
            result |= button.hover;
        }

        result
    }

    pub fn on_mouse_down(&mut self, pos: P2) -> bool {
        let mut result = false;
        for button in &mut self.buttons {
            button.click = button.bounds.contains(pos);
            result |= button.click;
        }
        result
    }

    pub fn on_mouse_up(&mut self, pos: P2) -> Option<ID> {
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

    pub fn push(&mut self, button: GuiButton<ID>) -> usize {
        self.buttons.push(button);
        self.buttons.len() - 1
    }
}
