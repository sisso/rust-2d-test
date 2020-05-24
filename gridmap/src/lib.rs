use crate::grid::Grid;
pub use crate::grid::GridCoord;

mod grid;

pub type ComponentId = u32;

#[derive(Debug, Clone)]
pub struct ComponentProperties {
    pub require_border_back: bool,
    pub require_border_front: bool,
    pub connect_rooms: bool,
    pub connect_outside: bool,
}

impl ComponentProperties {
    pub fn new() -> Self {
        ComponentProperties {
            require_border_back: false,
            require_border_front: false,
            connect_rooms: false,
            connect_outside: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComponentDef {
    pub id: ComponentId,
    pub code: String,
    pub properties: ComponentProperties,
}

#[derive(Debug, Clone)]
pub struct ComponentError {
    pub coords: GridCoord,
    pub kind: ComponentErrorKind,
}

#[derive(Debug, Clone)]
pub enum ComponentErrorKind {
    InvalidCoords,
    RequireBackBorder,
    RequireFrontBorder,
    BorderOthers,
    BorderExternal,
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

#[derive(Debug, Clone)]
pub struct ComponentAt {
    pub coords: GridCoord,
    pub component_id: ComponentId,
}

type ShipDesignGrid = Grid<ComponentAt>;

#[derive(Debug, Clone)]
pub struct ShipDesign {
    pub grid: ShipDesignGrid,
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
        repo: &ShipDesignRepository,
        coords: GridCoord,
        component_id: Option<ComponentId>,
    ) -> std::result::Result<(), ComponentError> {
        let value = component_id.map(|component_id| ComponentAt {
            coords,
            component_id,
        });

        let mut new_grid = self.grid.clone();
        new_grid.set_at(coords, value);

        ShipDesign::is_valid(&new_grid, repo)?;

        self.grid = new_grid;

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

    fn is_valid(grid: &ShipDesignGrid, repo: &ShipDesignRepository) -> Result<(), ComponentError> {
        for j in 0..grid.height {
            for i in 0..grid.width {
                let coords = (i, j).into();
                ShipDesign::is_valid_cell(grid, repo, coords)?;
            }
        }

        Ok(())
    }

    fn is_valid_cell(
        grid: &ShipDesignGrid,
        repo: &ShipDesignRepository,
        coords: GridCoord,
    ) -> Result<(), ComponentError> {
        if !grid.is_valid_coords(coords) {
            return Err(ComponentError {
                coords,
                kind: ComponentErrorKind::InvalidCoords,
            });
        }

        let component_id = match grid.get_at(coords) {
            Some(v) => v.component_id,
            _ => return Ok(()),
        };

        let comp_def = repo.get_component(component_id);

        if !comp_def.properties.connect_outside {
            if coords.y == 0 || coords.y == grid.height - 1 {
                return Err(ComponentError {
                    coords,
                    kind: ComponentErrorKind::BorderExternal,
                });
            }

            if coords.x == 0 && !comp_def.properties.require_border_back {
                return Err(ComponentError {
                    coords,
                    kind: ComponentErrorKind::BorderExternal,
                });
            }

            if coords.x == grid.width - 1 && !comp_def.properties.require_border_front {
                return Err(ComponentError {
                    coords,
                    kind: ComponentErrorKind::BorderExternal,
                });
            }
        }

        if comp_def.properties.require_border_back {
            // check that goes until back
            let amount = ShipDesign::raytrace_by_component(grid, coords, -1, 0, component_id);
            if amount != coords.x {
                return Err(ComponentError {
                    coords,
                    kind: ComponentErrorKind::RequireBackBorder,
                });
            }
        }

        if comp_def.properties.require_border_front {
            // check if goes until front
            let amount = ShipDesign::raytrace_by_component(grid, coords, 1, 0, component_id);
            let expected = grid.width - coords.x - 1;
            if amount != expected {
                return Err(ComponentError {
                    coords,
                    kind: ComponentErrorKind::RequireFrontBorder,
                });
            }
        }

        if !comp_def.properties.connect_rooms {
            let neighbours = grid.get_neighbours(coords);

            let mut invalid_neighbours = neighbours.into_iter().flat_map(|other_coord| match grid
                .get_at(other_coord)
            {
                Some(other) if other.component_id != component_id => {
                    // if is a different component, check if other component connect rooms
                    if repo
                        .get_component(other.component_id)
                        .properties
                        .connect_rooms
                    {
                        None
                    } else {
                        Some(other_coord)
                    }
                }
                _ => None,
            });

            if let Some(invalid) = invalid_neighbours.next() {
                return Err(ComponentError {
                    coords,
                    kind: ComponentErrorKind::BorderOthers,
                });
            }
        }

        Ok(())
    }

    fn raytrace_by_component(
        grid: &ShipDesignGrid,
        coords: GridCoord,
        dir_x: i32,
        dir_y: i32,
        component_id: ComponentId,
    ) -> u32 {
        grid.raytrace(coords, dir_x, dir_y)
            .into_iter()
            .flat_map(|coord| grid.get_at(coord))
            .filter(|comp| comp.component_id == component_id)
            .count() as u32
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

    pub fn add_component_def(
        &mut self,
        code: &str,
        properties: ComponentProperties,
    ) -> ComponentId {
        let next_id = self.components.len() as u32;

        self.components.push(ComponentDef {
            id: next_id,
            code: code.to_string(),
            properties,
        });

        next_id
    }

    pub fn get_component(&self, id: ComponentId) -> &ComponentDef {
        self.components.get(id as usize).unwrap()
    }

    pub fn list_components(&self) -> &Vec<ComponentDef> {
        &self.components
    }

    pub fn get_by_code(&self, code: &str) -> Option<&ComponentDef> {
        self.components.iter().find(|comp| comp.code == code)
    }

    pub fn get_id_by_code(&self, code: &str) -> Option<ComponentId> {
        self.components
            .iter()
            .find(|comp| comp.code == code)
            .map(|comp| comp.id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::grid::GridCoord;

    fn setup() -> ShipDesignRepository {
        let mut repo = ShipDesignRepository::new();

        let mut properties = ComponentProperties::new();
        properties.require_border_back = true;
        repo.add_component_def("engine", properties);

        let mut properties = ComponentProperties::new();
        properties.require_border_front = true;
        repo.add_component_def("cockpit", properties);

        let mut properties = ComponentProperties::new();
        properties.connect_rooms = true;
        properties.connect_outside = true;
        repo.add_component_def("airlock", properties);

        repo
    }

    #[test]
    fn test_set_engines() {
        let repo = setup();
        let mut design = ShipDesign::new(4, 4);
        let comp = repo.get_by_code("engine").unwrap();

        match design.set_component(&repo, GridCoord::new(1, 1), Some(comp.id)) {
            Err(ComponentError {
                kind: ComponentErrorKind::RequireBackBorder,
                ..
            }) => {}
            other => panic!("not expected to work {:?}", other),
        }

        design
            .set_component(&repo, GridCoord::new(0, 1), Some(comp.id))
            .unwrap();

        design
            .set_component(&repo, GridCoord::new(1, 1), Some(comp.id))
            .unwrap();
    }

    #[test]
    fn test_set_components_borders_between_components() {
        let repo = setup();
        let mut design = ShipDesign::new(4, 4);
        let engine_id = repo.get_id_by_code("engine").unwrap();
        let cockpit_id = repo.get_id_by_code("cockpit").unwrap();
        let airlock_id = repo.get_id_by_code("airlock").unwrap();

        for x in 0..3 {
            design
                .set_component(&repo, GridCoord::new(x, 1), Some(engine_id))
                .unwrap();
        }

        match design.set_component(&repo, GridCoord::new(3, 1), Some(cockpit_id)) {
            Err(ComponentError {
                kind: ComponentErrorKind::BorderOthers,
                ..
            }) => {}
            other => panic!("not expected to work {:?}", other),
        }

        design
            .set_component(&repo, GridCoord::new(3, 1), Some(airlock_id))
            .unwrap();
    }

    #[test]
    fn test_set_components_borders() {
        let repo = setup();
        let mut design = ShipDesign::new(4, 4);
        let engine_id = repo.get_id_by_code("engine").unwrap();
        let airlock_id = repo.get_id_by_code("airlock").unwrap();

        for (x, y) in &[(0, 0), (0, 3)] {
            match design.set_component(&repo, GridCoord::new(*x, *y), Some(engine_id)) {
                Err(ComponentError {
                    kind: ComponentErrorKind::BorderExternal,
                    ..
                }) => {}
                other => panic!("not expected to work {:?}", other),
            }
        }

        design
            .set_component(&repo, GridCoord::new(2, 0), Some(airlock_id))
            .unwrap();
    }

    #[test]
    fn test_set_cockpit() {
        let repo = setup();
        let mut design = ShipDesign::new(4, 4);
        let comp = repo.get_by_code("cockpit").unwrap();

        match design.set_component(&repo, GridCoord::new(1, 1), Some(comp.id)) {
            Err(ComponentError {
                kind: ComponentErrorKind::RequireFrontBorder,
                ..
            }) => {}
            other => panic!("not expected to work {:?}", other),
        }

        design
            .set_component(&repo, GridCoord::new(3, 1), Some(comp.id))
            .unwrap();

        design
            .set_component(&repo, GridCoord::new(2, 1), Some(comp.id))
            .unwrap();
    }

    #[test]
    fn test_remove_component_should_check_for_constraints() {
        let repo = setup();
        let mut design = ShipDesign::new(3, 3);
        let comp = repo.get_by_code("engine").unwrap();

        design
            .set_component(&repo, GridCoord::new(0, 1), Some(comp.id))
            .unwrap();

        design
            .set_component(&repo, GridCoord::new(1, 1), Some(comp.id))
            .unwrap();

        match design.set_component(&repo, GridCoord::new(0, 1), None) {
            Err(ComponentError {
                kind: ComponentErrorKind::RequireBackBorder,
                ..
            }) => {}
            other => panic!("not expected to work {:?}", other),
        }
    }
}
