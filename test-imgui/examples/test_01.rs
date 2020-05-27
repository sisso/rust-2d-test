use ggez::conf;
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics;
use ggez::nalgebra as na;
use ggez::{Context, GameResult};
use imgui::ImString;
use test_imgui::imgui_wrapper::ImGuiWrapper;

struct MainState {
    pos_x: f32,
    imgui_wrapper: ImGuiWrapper,
    buffer: ImString,
    value: f32,
}

impl MainState {
    fn new(mut ctx: &mut Context) -> GameResult<MainState> {
        let imgui_wrapper = ImGuiWrapper::new(&mut ctx);
        let s = MainState {
            pos_x: 0.0,
            imgui_wrapper,
            buffer: Default::default(),
            value: 0.0,
        };
        Ok(s)
    }

    fn render_gui(&mut self, ctx: &mut Context) -> GameResult<()> {
        use imgui::*;

        let show_popup = false;
        let buffer = &mut self.buffer;
        let value = &mut self.value;

        self.imgui_wrapper.render(ctx, 1.0, |ui| {
            // Various ui things
            {
                // Window
                ui.window(im_str!("Hello world"))
                    .size([300.0, 600.0], imgui::Condition::Always)
                    .position([100.0, 100.0], imgui::Condition::Always)
                    .build(|| {
                        ui.text(im_str!("Hello world!"));
                        ui.text(im_str!("This...is...imgui-rs!"));
                        ui.separator();
                        let mouse_pos = ui.io().mouse_pos;
                        ui.text(im_str!(
                            "Mouse Position: ({:.1},{:.1})",
                            mouse_pos[0],
                            mouse_pos[1]
                        ));
                        ui.separator();
                        if ui.input_float(im_str!("Value:"), value).build() {}
                        if ui.input_text(im_str!("Text:"), buffer).build() {}
                        ui.separator();
                        // ui.input_text_multiline(im_str!("Text:"), buffer, [200.0, 80.0])
                        //     .build();
                        // ui.separator();

                        if ui.small_button(im_str!("small button")) {
                            println!("Small button clicked");
                        }
                    });

                // Popup
                ui.popup(im_str!("popup"), || {
                    if ui.menu_item(im_str!("popup menu item 1")).build() {
                        println!("popup menu item 1 clicked");
                    }

                    if ui.menu_item(im_str!("popup menu item 2")).build() {
                        println!("popup menu item 2 clicked");
                    }
                });

                // Menu bar
                ui.main_menu_bar(|| {
                    ui.menu(im_str!("Menu 1")).build(|| {
                        if ui.menu_item(im_str!("Item 1.1")).build() {
                            println!("item 1.1 inside menu bar clicked");
                        }

                        ui.menu(im_str!("Item 1.2")).build(|| {
                            if ui.menu_item(im_str!("Item 1.2.1")).build() {
                                println!("item 1.2.1 inside menu bar clicked");
                            }
                            if ui.menu_item(im_str!("Item 1.2.2")).build() {
                                println!("item 1.2.2 inside menu bar clicked");
                            }
                        });
                    });

                    ui.menu(im_str!("Menu 2")).build(|| {
                        if ui.menu_item(im_str!("Item 2.1")).build() {
                            println!("item 2.1 inside menu bar clicked");
                        }
                    });
                });
            }

            if show_popup {
                println!("draw popup");
                ui.open_popup(im_str!("popup"));
            }
        });
        Ok(())
    }
}

impl EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        self.pos_x = self.pos_x % 800.0 + 1.0;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        // Render game stuff
        {
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                na::Point2::new(self.pos_x, 380.0),
                100.0,
                2.0,
                graphics::WHITE,
            )?;
            graphics::draw(ctx, &circle, (na::Point2::new(0.0, 0.0),))?;
        }

        // Render game ui
        {
            self.render_gui(ctx);
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.imgui_wrapper.update_mouse_pos(x, y);
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.imgui_wrapper.update_mouse_down((
            button == MouseButton::Left,
            button == MouseButton::Right,
            button == MouseButton::Middle,
        ));
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.imgui_wrapper.update_mouse_down((false, false, false));
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        self.imgui_wrapper.update_key_down(keycode);
    }

    fn text_input_event(&mut self, _ctx: &mut Context, character: char) {
        self.imgui_wrapper.update_key_char(character);
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        self.imgui_wrapper.update_key_up(keycode);
    }
}

pub fn main() -> ggez::GameResult {
    let cb = ggez::ContextBuilder::new("super_simple with imgui", "ggez")
        .window_setup(conf::WindowSetup::default().title("super_simple with imgui"))
        .window_mode(conf::WindowMode::default().dimensions(750.0, 500.0));
    let (ref mut ctx, event_loop) = &mut cb.build()?;

    let state = &mut MainState::new(ctx)?;

    event::run(ctx, event_loop, state)
}
