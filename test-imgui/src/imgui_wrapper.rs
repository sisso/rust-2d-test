use ggez::graphics;
use ggez::Context;

use gfx_core::factory::Factory;
use gfx_core::{format, texture};
use gfx_core::{handle::RenderTargetView, memory::Typed};
use gfx_device_gl;

use imgui::*;
use imgui_gfx_renderer::*;

use ggez::event::KeyCode;
use std::time::Instant;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
struct MouseState {
    pos: (i32, i32),
    pressed: (bool, bool, bool),
    wheel: f32,
}

pub struct ImGuiWrapper {
    pub imgui: imgui::Context,
    pub renderer: Renderer<gfx_core::format::Rgba8, gfx_device_gl::Resources>,
    last_frame: Instant,
    mouse_state: MouseState,
}

impl ImGuiWrapper {
    pub fn new(ctx: &mut Context) -> Self {
        // Create the imgui object
        let mut imgui = imgui::Context::create();
        let (factory, gfx_device, _, _, _) = graphics::gfx_objects(ctx);

        // Shaders
        let shaders = {
            let version = gfx_device.get_info().shading_language;
            if version.is_embedded {
                if version.major >= 3 {
                    Shaders::GlSlEs300
                } else {
                    Shaders::GlSlEs100
                }
            } else if version.major >= 4 {
                Shaders::GlSl400
            } else if version.major >= 3 {
                Shaders::GlSl130
            } else {
                Shaders::GlSl110
            }
        };

        // Renderer
        let mut renderer = Renderer::init(&mut imgui, &mut *factory, shaders).unwrap();

        {
            // TODO: not working, for some reason Key::$key is invalid
            macro_rules! map_keys {
                ( $( $key:expr ),* ) => {
                    $(
                        io.key_map[Key::$key as usize] = KeyCode::$key as u32;
                    )*
                };
            }

            let mut io = imgui.io_mut();
            io.key_map[Key::Tab as usize] = KeyCode::Tab as u32;
            io.key_map[Key::LeftArrow as usize] = KeyCode::Left as u32;
            io.key_map[Key::RightArrow as usize] = KeyCode::Right as u32;
            io.key_map[Key::UpArrow as usize] = KeyCode::Up as u32;
            io.key_map[Key::DownArrow as usize] = KeyCode::Down as u32;
            io.key_map[Key::PageUp as usize] = KeyCode::PageUp as u32;
            io.key_map[Key::PageDown as usize] = KeyCode::PageDown as u32;
            io.key_map[Key::Home as usize] = KeyCode::Home as u32;
            io.key_map[Key::End as usize] = KeyCode::End as u32;
            io.key_map[Key::Delete as usize] = KeyCode::Delete as u32;
            io.key_map[Key::Backspace as usize] = KeyCode::Back as u32;
            io.key_map[Key::Enter as usize] = KeyCode::Return as u32;
            io.key_map[Key::Escape as usize] = KeyCode::Escape as u32;
            io.key_map[Key::Space as usize] = KeyCode::Space as u32;
            // map_keys![A, B, C, V, X, Y, Z];
            // io.key_map[Key::A as usize] = KeyCode::A as u32;
            // io.key_map[Key::C as usize] = KeyCode::C as u32;
            // io.key_map[Key::V as usize] = KeyCode::V as u32;
            // io.key_map[Key::X as usize] = KeyCode::X as u32;
            // io.key_map[Key::Y as usize] = KeyCode::Y as u32;
            // io.key_map[Key::Z as usize] = KeyCode::Z as u32;
        }

        // Create instace
        Self {
            imgui,
            renderer,
            last_frame: Instant::now(),
            mouse_state: MouseState::default(),
        }
    }

    pub fn render<Builder>(&mut self, ctx: &mut Context, hidpi_factor: f32, builder: Builder)
    where
        Builder: FnOnce(&Ui),
    {
        // Update mouse
        self.update_mouse();

        // Create new frame
        let now = Instant::now();
        let delta = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame = now;

        let (draw_width, draw_height) = graphics::drawable_size(ctx);
        self.imgui.io_mut().display_size = [draw_width, draw_height];
        self.imgui.io_mut().display_framebuffer_scale = [hidpi_factor, hidpi_factor];
        self.imgui.io_mut().delta_time = delta_s;

        let ui = self.imgui.frame();
        builder(&ui);

        // Render
        let (factory, _, encoder, _, render_target) = graphics::gfx_objects(ctx);
        let draw_data = ui.render();
        self.renderer
            .render(
                &mut *factory,
                encoder,
                &mut RenderTargetView::new(render_target.clone()),
                draw_data,
            )
            .unwrap();
    }

    fn update_mouse(&mut self) {
        self.imgui.io_mut().mouse_pos =
            [self.mouse_state.pos.0 as f32, self.mouse_state.pos.1 as f32];

        self.imgui.io_mut().mouse_down = [
            self.mouse_state.pressed.0,
            self.mouse_state.pressed.1,
            self.mouse_state.pressed.2,
            false,
            false,
        ];

        self.imgui.io_mut().mouse_wheel = self.mouse_state.wheel;
        self.mouse_state.wheel = 0.0;
    }

    pub fn update_mouse_pos(&mut self, x: f32, y: f32) {
        self.mouse_state.pos = (x as i32, y as i32);
    }

    pub fn update_mouse_down(&mut self, pressed: (bool, bool, bool)) {
        self.mouse_state.pressed = pressed;
    }

    pub fn update_key_char(&mut self, key: char) {
        println!("update {:?}", key);
        self.imgui.io_mut().add_input_character(key);
    }

    pub fn update_key_down(&mut self, key: KeyCode) {
        match key {
            KeyCode::Back => self.imgui.io_mut().keys_down[Key::Backspace as usize] = true,
            _ => {}
        }

        // self.imgui.io_mut().
        // let keys = &mut self.imgui.io_mut().keys_down;
        // keys[key as usize] = true;
        // // self.imgui.io_mut().add_input_character(key);
        // match key {
        //     KeyCode::Key1 => self.imgui.io_mut().add_input_character('1'),
        //     KeyCode::A => self.imgui.io_mut().add_input_character('a'),
        //     _ => {}
        // }
    }

    pub fn update_key_up(&mut self, key: KeyCode) {
        // let keys = &mut self.imgui.io_mut().keys_down;
        // keys[key as usize] = false;
    }
}
