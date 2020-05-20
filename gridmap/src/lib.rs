use ggez::GameResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentCfg {
    pub code: String,
    pub grid_image: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cfg {
    components: Vec<ComponentCfg>,
}

impl Cfg {
    pub fn load(file_path: &str) -> GameResult<Cfg> {
        let body = std::fs::read_to_string(file_path).unwrap();
        let cfg: Cfg = serde_json::from_str(body.as_str()).unwrap();
        Ok(cfg)
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    Failure,
    Exception,
    Error,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct P2 {
    pub x: i32,
    pub y: i32,
}

impl P2 {
    pub fn new(x: i32, y: i32) -> Self {
        P2 { x, y }
    }
}

#[derive(Debug, Clone)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Size { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShipComponent {
    Corridor,
    Engine,
    Cockpit,
    PowerGenerator,
    LifeSupport,
}

#[derive(Debug, Clone, Copy)]
pub struct GridCoord {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone)]
pub struct Grid<T> {
    pub list: Vec<T>,
}

impl<T> Grid<T> {
    pub fn new() -> Self {
        Grid { list: vec![] }
    }
}

#[derive(Debug, Clone)]
pub struct ComponentAt {
    pub index: u32,
    pub component: ShipComponent,
}

#[derive(Debug, Clone)]
pub struct ShipDesign {
    pub size: Size,
    pub grid: Grid<ComponentAt>,
}

impl ShipDesign {
    pub fn new() -> Self {
        ShipDesign {
            size: Size::new(20, 8),
            grid: Grid::new(),
        }
    }

    pub fn is_valid_coords(&self, coords: GridCoord) -> bool {
        coords.x < self.size.width && coords.y < self.size.height
    }
}

#[derive(Debug, Clone)]
pub struct Repository {
    cfg: Cfg,
}

impl Repository {
    pub fn new(cfg: Cfg) -> Self {
        Repository { cfg }
    }

    pub fn list_components<'a>(&'a self) -> impl Iterator<Item = &'a ComponentCfg> + 'a {
        self.cfg.components.iter()
    }
}
