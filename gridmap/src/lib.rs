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
pub enum SetComponentError {
    InvalidIndex,
    RequireBackBorder,
    RequireFrontBorder,
    BorderOutside,
    BorderOthers,
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
        repo: &ShipDesignRepository,
        coords: GridCoord,
        component_id: Option<ComponentId>,
    ) -> std::result::Result<(), SetComponentError> {
        self.is_valid(repo, coords, component_id)?;

        let value = component_id.map(|component_id| ComponentAt {
            coords,
            component_id,
        });

        self.grid.set_at(coords, value);

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

    fn is_valid(
        &self,
        repo: &ShipDesignRepository,
        coords: GridCoord,
        component_id: Option<ComponentId>,
    ) -> Result<(), SetComponentError> {
        if !self.grid.is_valid_coords(coords) {
            return Err(SetComponentError::InvalidIndex);
        }

        match component_id {
            Some(component_id) => {
                let comp_def = repo.get_component(component_id);

                if comp_def.properties.require_border_back {
                    let trace = self.grid.raytrace(coords, -1, 0);
                    let trace_same: Vec<_> = trace
                        .into_iter()
                        .flat_map(|coord| self.grid.get_at(coord))
                        .filter(|comp| comp.component_id == component_id)
                        .collect();

                    // check that goes until back
                    if trace_same.len() as u32 != coords.x {
                        return Err(SetComponentError::RequireBackBorder);
                    }
                }

                if comp_def.properties.require_border_front {
                    let trace = self.grid.raytrace(coords, 1, 0);
                    let trace_same: Vec<_> = trace
                        .into_iter()
                        .flat_map(|coord| self.grid.get_at(coord))
                        .filter(|comp| comp.component_id == component_id)
                        .collect();

                    // // check that goes until back
                    // println!("{:?} {:?}", trace_same.len(), self.grid.width - coords.x);

                    let expected = self.grid.width - coords.x - 1;
                    if trace_same.len() as u32 != expected {
                        return Err(SetComponentError::RequireFrontBorder);
                    }
                }

                Ok(())
            }
            None => Ok(()),
        }
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

        repo
    }

    #[test]
    fn test_set_engines() {
        let repo = setup();
        let mut design = ShipDesign::new(4, 4);
        let comp = repo.get_by_code("engine").unwrap();

        match design.set_component(&repo, GridCoord::new(1, 0), Some(comp.id)) {
            Err(SetComponentError::RequireBackBorder) => {}
            other => panic!("not expected to work {:?}", other),
        }

        design
            .set_component(&repo, GridCoord::new(0, 0), Some(comp.id))
            .unwrap();

        design
            .set_component(&repo, GridCoord::new(1, 0), Some(comp.id))
            .unwrap();
    }

    #[test]
    fn test_set_cockpit() {
        let repo = setup();
        let mut design = ShipDesign::new(4, 4);
        let comp = repo.get_by_code("cockpit").unwrap();

        match design.set_component(&repo, GridCoord::new(1, 0), Some(comp.id)) {
            Err(SetComponentError::RequireFrontBorder) => {}
            other => panic!("not expected to work {:?}", other),
        }

        design
            .set_component(&repo, GridCoord::new(3, 0), Some(comp.id))
            .unwrap();

        design
            .set_component(&repo, GridCoord::new(2, 0), Some(comp.id))
            .unwrap();
    }
}
