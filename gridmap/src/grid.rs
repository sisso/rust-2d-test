#[derive(Debug, Clone, Copy)]
pub struct GridCoord {
    pub x: u32,
    pub y: u32,
}

impl GridCoord {
    pub fn new(x: u32, y: u32) -> Self {
        GridCoord { x, y }
    }
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
