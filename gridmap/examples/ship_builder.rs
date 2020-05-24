use approx::assert_relative_eq;
use commons::math::{self, p2, v2, Transform2, P2, V2};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::{Color, DrawMode, DrawParam, Rect};
use ggez::{filesystem, graphics, input, Context, ContextBuilder, GameError, GameResult};
use gridmap::{ComponentId, ComponentProperties, GridCoord, ShipDesign, ShipDesignRepository};
use nalgebra::{Point2, Vector2};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;
use std::time::Instant;

// TODO: drag scale proportional of scale
// TODO: create struct EditorLocalPos ScreenPos

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GuiComponentCfg {
    code: String,
    grid_image: String,
    require_border_back: Option<bool>,
    require_border_front: Option<bool>,
    connect_rooms: Option<bool>,
    connect_outside: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GuiCfg {
    cell_size: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShipDesignCfg {
    components: Vec<GuiComponentCfg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppCfg {
    gui: GuiCfg,
    ship_design: ShipDesignCfg,
}

impl AppCfg {
    pub fn from_json_string(body: &str) -> GameResult<AppCfg> {
        let value: Value = serde_json::from_str(body).unwrap();
        Ok(serde_json::from_value(value).unwrap())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GuiId {
    Component(ComponentId),
    Remove,
}

#[derive(Debug)]
struct Gui {
    buttons_panel: commons::graphics::GuiManage<GuiId>,
    state: GuiState,
    component_images: HashMap<ComponentId, graphics::Image>,
    ghost_component: Option<(ComponentId, GridCoord)>,
    fail_component: Vec<(Instant, GridCoord)>,
}

#[derive(Debug)]
enum GuiState {
    Idle,
    ComponentSelected { component_id: ComponentId },
    RemoveSelected,
}

struct Resources {}

impl Resources {
    pub fn get_string(ctx: &mut Context, name: &str) -> GameResult<String> {
        let mut buffer = String::new();
        filesystem::open(ctx, name)?
            .read_to_string(&mut buffer)
            .map_err(|e| GameError::FilesystemError(format!("Fail to read {}: {}", name, e)))?;
        Ok(buffer)
    }
}

fn flatten_results<T>(list: Vec<GameResult<T>>) -> GameResult<Vec<T>> {
    let mut result = vec![];
    for i in list {
        result.push(i?);
    }
    Ok(result)
}

#[derive(Debug)]
struct App {
    cfg: AppCfg,
    editor_transform: math::Transform2,
    design: ShipDesign,
    repository: ShipDesignRepository,
    gui: Gui,
}

impl App {
    pub fn new(ctx: &mut Context) -> GameResult<App> {
        let cfg = AppCfg::from_json_string(Resources::get_string(ctx, "/config.json")?.as_str())?;
        let mut repository = ShipDesignRepository::new();
        let mut component_images = HashMap::new();
        let mut buttons = vec![(GuiId::Remove, "Clear")];

        for comp in &cfg.ship_design.components {
            let properties = ComponentProperties {
                require_border_back: comp.require_border_back.unwrap_or(false),
                require_border_front: comp.require_border_front.unwrap_or(false),
                connect_rooms: comp.connect_rooms.unwrap_or(false),
                connect_outside: comp.connect_outside.unwrap_or(false),
            };

            let id = repository.add_component_def(comp.code.as_str(), properties);
            let image = graphics::Image::new(ctx, comp.grid_image.as_str())?;
            component_images.insert(id, image);
            buttons.push((GuiId::Component(id), comp.code.as_str()));
        }

        let mut buttons_panel = commons::graphics::GuiManage::new();
        add_panel_buttons(
            &mut buttons_panel,
            &buttons,
            graphics::screen_coordinates(ctx),
        );

        let gui = Gui {
            buttons_panel,
            state: GuiState::Idle,
            component_images,
            ghost_component: None,
            fail_component: vec![],
        };

        let app = App {
            cfg: cfg,
            editor_transform: Transform2::identity(),
            design: ShipDesign::new(20, 8),
            repository,
            gui: gui,
        };

        Ok(app)
    }

    // TODO: move to guieditor
    pub fn get_grid_pos(&self, coords: GridCoord) -> P2 {
        let local_pos = p2(
            coords.x as f32 * self.cfg.gui.cell_size,
            coords.y as f32 * self.cfg.gui.cell_size,
        );

        local_pos
    }

    // TODO: move to guieditor
    pub fn get_editor_local_pos(&self, screen_pos: P2) -> P2 {
        self.editor_transform.point_to_local(&screen_pos)
    }

    // TODO: move to guieditor
    /// return grid coords using local coordinates
    pub fn get_grid_coords(&self, pos: P2) -> Option<GridCoord> {
        let cell_size = self.cfg.gui.cell_size;

        let index_x = pos.x / cell_size;
        let index_y = pos.y / cell_size;
        let coords = GridCoord {
            x: index_x as u32,
            y: index_y as u32,
        };

        if self.design.is_valid_coords(coords) {
            Some(coords)
        } else {
            None
        }
    }

    // TODO: move to guieditor
    pub fn move_screen(&mut self, v: V2) {
        self.editor_transform.translate(v);
    }

    // TODO: move to guieditor
    pub fn zoom_screen(&mut self, amount: f32) {
        let scale = if amount > 0.0 {
            1.1
        } else if amount < 0.0 {
            0.9
        } else {
            1.0
        };

        self.editor_transform.scale(scale);
    }

    // TODO: move to guieditor
    fn get_editor_area(&self, ctx: &Context) -> Rect {
        assert_relative_eq!(self.editor_transform.get_angle(), 0.0);

        let rect = graphics::screen_coordinates(ctx);
        let p1 = p2(rect.left(), rect.top());
        let p2 = p2(rect.right(), rect.bottom());

        let local_p1 = self.editor_transform.point_to_local(&p1);

        let local_p2 = self.editor_transform.point_to_local(&p2);

        let new_rect = Rect::new(
            local_p1.x,
            local_p1.y,
            local_p2.x - local_p1.x,
            local_p2.y - local_p1.y,
        );

        new_rect
    }

    pub fn get_component_image_by_id(&self, id: ComponentId) -> GameResult<&graphics::Image> {
        self.gui
            .component_images
            .get(&id)
            .ok_or(GameError::FilesystemError(format!(
                "image component index {} not found",
                id
            )))
    }

    /// apply changes when in editor when player click or drag
    pub fn editor_apply_state(&mut self, enabled: bool, mouse_pos: P2) {
        match self.gui.state {
            GuiState::ComponentSelected { component_id } => {
                self.editor_apply_component(enabled, mouse_pos, Some(component_id))
            }
            GuiState::RemoveSelected => self.editor_apply_component(enabled, mouse_pos, None),
            GuiState::Idle => {}
        }
    }

    fn editor_apply_component(
        &mut self,
        enabled: bool,
        mouse_pos: P2,
        component_id: Option<ComponentId>,
    ) {
        let editor_pos = self.get_editor_local_pos(mouse_pos);
        if let Some(coords) = self.get_grid_coords(editor_pos) {
            if enabled {
                match self
                    .design
                    .set_component(&self.repository, coords, component_id)
                {
                    Ok(_) => {}
                    Err(e) => {
                        self.gui.fail_component.push((Instant::now(), e.coords));

                        eprintln!(
                            "fail to place {:?} at {:?}: {:?}",
                            component_id, mouse_pos, e
                        )
                    }
                }
            }
            self.gui.ghost_component = component_id.map(|i| (i, coords));
        }
    }

    pub fn draw_cell_outlier(
        &self,
        ctx: &mut Context,
        coords: GridCoord,
        color: Color,
    ) -> GameResult<()> {
        let pos = self.get_grid_pos(coords);

        let mesh = graphics::Mesh::new_rectangle(
            ctx,
            DrawMode::stroke(2.0),
            Rect::new(pos.x, pos.y, self.cfg.gui.cell_size, self.cfg.gui.cell_size),
            color,
        )?;

        graphics::draw(ctx, &mesh, DrawParam::default())
    }
}

fn add_panel_buttons(
    gui: &mut commons::graphics::GuiManage<GuiId>,
    components: &Vec<(GuiId, &str)>,
    screen_size: Rect,
) -> GameResult<()> {
    let button_rect = Rect::new(0.0, 0.0, 60.0, 40.0);
    let panel_rect = Rect::new(
        screen_size.x,
        screen_size.h - button_rect.h,
        screen_size.w,
        button_rect.h,
    );

    for (i, (id, component)) in components.iter().enumerate() {
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
            uid: *id,
            bounds: rect,
            text: component.to_string(),
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
        let now = Instant::now();

        self.gui.fail_component.retain(|(instant, _)| {
            let delta = now.duration_since(*instant);
            delta.as_secs() == 0
        });

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        // editor
        let editor_area = self.get_editor_area(ctx);
        graphics::set_screen_coordinates(ctx, editor_area)?;

        {
            let mut images = vec![];
            for component in &self.design.list_components() {
                let value = component.map(|component| {
                    self.get_component_image_by_id(component.component_id)
                        .unwrap()
                });

                images.push(value);
            }

            draw_ship(
                ctx,
                Point2::new(0.0, 0.0),
                self.cfg.gui.cell_size,
                self.design.get_width(),
                self.design.get_height(),
                images,
            )?;
        }

        {
            if let Some((id, coords)) = self.gui.ghost_component {
                let pos = self.get_grid_pos(coords);
                let img = self.get_component_image_by_id(id)?;

                graphics::draw(ctx, img, DrawParam::new().dest(pos))?;
            }
        }

        {
            // graphics::push_transform(ctx, None);

            // graphics::push_transform(ctx, Some(self.editor_transform.get_matrix()));
            draw_grid(
                ctx,
                Point2::new(0.0, 0.0),
                self.cfg.gui.cell_size,
                self.design.get_width(),
                self.design.get_height(),
            )?;

            // graphics::pop_transform(ctx);
        }

        {
            if let Some((id, coords)) = self.gui.ghost_component {
                let pos = self.get_grid_pos(coords);
                let img = self.get_component_image_by_id(id)?;

                graphics::draw(ctx, img, DrawParam::new().dest(pos))?;
            }
        }

        {
            for (_, coords) in &self.gui.fail_component {
                self.draw_cell_outlier(ctx, *coords, Color::new(1.0, 0.0, 0.0, 1.0));
            }
        }

        // gui
        let screen_size = Rect::new(0.0, 0.0, WIDTH, HEIGHT);
        graphics::set_screen_coordinates(ctx, screen_size)?;

        {
            let fps = ggez::timer::fps(ctx);
            let text = graphics::Text::new(format!("fps: {:.0}", fps));
            graphics::draw(ctx, &text, (Point2::new(0.0, 0.0), graphics::WHITE))?;
        }

        // gui buttons
        {
            self.gui.buttons_panel.draw(ctx)?;
        }

        // show
        graphics::present(ctx)
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            if !self.gui.buttons_panel.on_mouse_down(p2(x, y)) {
                self.editor_apply_state(true, p2(x, y));
            }
        }
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        let pos = Point2::new(x, y);

        if button == MouseButton::Left {
            match self.gui.buttons_panel.on_mouse_up(pos) {
                Some(GuiId::Component(component_id)) => {
                    self.gui.state = GuiState::ComponentSelected { component_id }
                }
                Some(GuiId::Remove) => self.gui.state = GuiState::RemoveSelected,
                _ => {}
            }
        }
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        if ggez::input::mouse::button_pressed(ctx, MouseButton::Right) {
            self.move_screen(Vector2::new(-dx, -dy));
        } else if !self.gui.buttons_panel.on_mouse_move(Point2::new(x, y)) {
            let enable = input::mouse::button_pressed(ctx, MouseButton::Left);
            self.editor_apply_state(enable, p2(x, y));
        }
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
        self.zoom_screen(-y);
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
                self.move_screen(Vector2::new(0.0, -screen_speed));
            }
            KeyCode::S => {
                self.move_screen(Vector2::new(0.0, screen_speed));
            }
            KeyCode::A => {
                self.move_screen(Vector2::new(screen_speed, 0.0));
            }
            KeyCode::D => {
                self.move_screen(Vector2::new(-screen_speed, 0.0));
            }
            _ => {}
        }
    }
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
    for i in 0..width + 1 {
        let x = i as f32 * grid_size;
        let p0 = pos + Vector2::new(x, 0.0);
        let p1 = pos + Vector2::new(x, max_y);
        draw_line(ctx, p0, p1, grid_color, grid_width)?;
    }

    // horizontal lines
    for i in 0..height + 1 {
        let y = i as f32 * grid_size;
        let p0 = pos + Vector2::new(0.0, y);
        let p1 = pos + Vector2::new(max_x, y);
        draw_line(ctx, p0, p1, grid_color, grid_width)?;
    }

    Ok(())
}

fn draw_ship(
    ctx: &mut Context,
    pos: P2,
    grid_size: f32,
    width: u32,
    height: u32,
    mut images: Vec<Option<&graphics::Image>>,
) -> GameResult<()> {
    let mut index = 0;
    for i in 0..height {
        for j in 0..width {
            let comp_image = images.get_mut(index).unwrap();
            if let Some(image) = comp_image.take() {
                let x = j as f32 * grid_size + pos.x;
                let y = i as f32 * grid_size + pos.y;
                // TODO: can not draw &&image, so we hack with mut and take
                graphics::draw(ctx, image, DrawParam::new().dest(p2(x, y)))?;
            }

            index += 1;
        }
    }

    Ok(())
}

fn main() -> GameResult<()> {
    let resource_dir = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = std::path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        std::path::PathBuf::from("./resources")
    };

    // Make a Context.
    let mut window_mode: WindowMode = Default::default();
    window_mode.width = WIDTH;
    window_mode.height = HEIGHT;

    let (mut ctx, mut event_loop) = ContextBuilder::new("template", "Sisso")
        .window_mode(window_mode)
        .add_resource_path(resource_dir)
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
