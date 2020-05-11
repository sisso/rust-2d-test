#[derive(Debug, Clone)]
pub struct Cfg {}

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

#[derive(Debug, Clone)]
pub struct Aabb {
    pub pos: P2,
    pub size: Size,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShipComponent {
    Corridor,
    Engine,
    Cockpit,
    PowerGenerator,
    LifeSupport,
}

#[derive(Debug, Clone)]
pub struct ComponentAt {
    pub aabb: Aabb,
    pub component: ShipComponent,
}

#[derive(Debug, Clone)]
pub struct ShipDesign {
    pub size: Size,
    pub components: Vec<ComponentAt>,
}

impl ShipDesign {
    pub fn new() -> Self {
        ShipDesign {
            size: Size::new(10, 4),
            components: vec![],
        }
    }

    pub fn add_component(&self, component: ShipComponent, dimension: Aabb) -> Result<()> {
        unimplemented!()
    }

    pub fn list_components<'a>(&'a self) -> impl Iterator<Item = &'a ComponentAt> + 'a {
        self.components.iter()
    }
}

pub struct Repository {}

impl Repository {
    pub fn list_components(&self) -> Vec<ComponentAt> {
        unimplemented!();
    }
}
