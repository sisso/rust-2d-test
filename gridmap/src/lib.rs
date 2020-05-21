use serde::{Deserialize, Serialize};

pub type ComponentId = u32;

#[derive(Debug, Clone)]
pub struct ComponentDef {
    pub id: ComponentId,
    pub code: String,
}

#[derive(Debug, Clone)]
pub enum SetComponentError {
    InvalidIndex,
}

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

#[derive(Debug, Clone, Copy)]
pub struct GridCoord {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone)]
pub struct Grid<T> {
    pub width: u32,
    pub height: u32,
    pub list: Vec<Option<T>>,
}

impl<T> Grid<T> {
    pub fn new(width: u32, height: u32) -> Self {
        let mut list = vec![];
        for _ in 0..width * height {
            list.push(None);
        }

        Grid {
            width,
            height,
            list,
        }
    }

    pub fn set(&mut self, index: u32, value: Option<T>) {
        self.list[index as usize] = value;
    }

    pub fn get(&self, index: u32) -> Option<&T> {
        self.list[index as usize].as_ref()
    }

    pub fn is_valid_coords(&self, coords: GridCoord) -> bool {
        coords.x < self.width && coords.y < self.height
    }

    pub fn coords_to_index(&self, coords: GridCoord) -> u32 {
        coords.y * self.width + coords.x
    }
}

#[derive(Debug, Clone)]
pub struct ComponentAt {
    pub index: u32,
    pub component_id: ComponentId,
}

#[derive(Debug, Clone)]
pub struct ShipDesign {
    pub grid: Grid<ComponentAt>,
}

impl ShipDesign {
    pub fn new(width: u32, height: u32) -> Self {
        ShipDesign {
            grid: Grid::new(width, height),
        }
    }

    pub fn is_valid_coords(&self, coords: GridCoord) -> bool {
        self.grid.is_valid_coords(coords)
    }

    pub fn set_component(
        &mut self,
        coords: GridCoord,
        component_id: ComponentId,
    ) -> std::result::Result<(), SetComponentError> {
        if !self.is_valid_coords(coords) {
            return Err(SetComponentError::InvalidIndex);
        }

        let index = self.grid.coords_to_index(coords);
        self.grid.set(
            index,
            Some(ComponentAt {
                index,
                component_id,
            }),
        );

        Ok(())
    }

    pub fn get_width(&self) -> u32 {
        self.grid.width
    }

    pub fn get_height(&self) -> u32 {
        self.grid.height
    }

    pub fn list_components(&self) -> Vec<Option<&ComponentAt>> {
        let mut result = vec![];
        let max = self.grid.width * self.grid.height;
        for i in 0..max {
            result.push(self.grid.get(i));
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct ShipDesignRepository {
    components: Vec<ComponentDef>,
}

impl ShipDesignRepository {
    pub fn new() -> Self {
        ShipDesignRepository { components: vec![] }
    }

    pub fn add_component_def(&mut self, code: &str) -> ComponentId {
        let next_id = self.components.len() as u32;

        self.components.push(ComponentDef {
            id: next_id,
            code: code.to_string(),
        });

        next_id
    }

    pub fn get_component(&self, id: ComponentId) -> &ComponentDef {
        self.components.get(id as usize).unwrap()
    }

    pub fn list_components(&self) -> &Vec<ComponentDef> {
        &self.components
    }
}
