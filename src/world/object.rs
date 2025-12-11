use serde::{Deserialize, Serialize};

use crate::entity::{Cabin, Item, Tree, TreeType, WoodShed};
use crate::world::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectSize {
    Small,
    Medium,
    Large,
    Massive,
}

impl ObjectSize {
    pub fn visibility_range(&self) -> i32 {
        match self {
            ObjectSize::Small | ObjectSize::Medium => 1,
            ObjectSize::Large => 2,
            ObjectSize::Massive => 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObjectSurface {
    pub items: Vec<Item>,
    pub capacity: Option<usize>,
    #[serde(default)]
    pub supports_mounts: bool,
}

impl ObjectSurface {
    pub fn add_item(&mut self, item: Item) -> bool {
        if let Some(cap) = self.capacity {
            if self.items.len() >= cap {
                return false;
            }
        }
        self.items.push(item);
        true
    }

    pub fn take_item(&mut self, item: &Item) -> bool {
        if let Some(idx) = self.items.iter().position(|i| i == item) {
            self.items.remove(idx);
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectKind {
    Cabin(Cabin),
    WoodShed(WoodShed),
    Tree(Tree),
    Table,
    Wall,
    Boulder,
    GenericStructure(String),
}

impl ObjectKind {
    pub fn name(&self) -> String {
        match self {
            ObjectKind::Cabin(_) => "cabin".to_string(),
            ObjectKind::WoodShed(_) => "wood shed".to_string(),
            ObjectKind::Tree(tree) => match tree.kind {
                TreeType::Pine => "pine tree".to_string(),
                TreeType::Birch => "birch tree".to_string(),
                TreeType::Apple => "apple tree".to_string(),
                TreeType::Bamboo => "bamboo grove".to_string(),
            },
            ObjectKind::Table => "table".to_string(),
            ObjectKind::Wall => "wall".to_string(),
            ObjectKind::Boulder => "boulder".to_string(),
            ObjectKind::GenericStructure(name) => name.clone(),
        }
    }

    pub fn default_size(&self) -> ObjectSize {
        match self {
            ObjectKind::Cabin(_) => ObjectSize::Massive,
            ObjectKind::WoodShed(_) => ObjectSize::Large,
            ObjectKind::Tree(_) => ObjectSize::Large,
            ObjectKind::Table => ObjectSize::Medium,
            ObjectKind::Wall => ObjectSize::Large,
            ObjectKind::Boulder => ObjectSize::Large,
            ObjectKind::GenericStructure(_) => ObjectSize::Large,
        }
    }

    pub fn visibility_override(&self) -> Option<i32> {
        match self {
            ObjectKind::Cabin(_) => Some(5),
            ObjectKind::Tree(_) => Some(3),
            _ => None,
        }
    }

    pub fn supports_surface(&self) -> bool {
        matches!(self, ObjectKind::Cabin(_) | ObjectKind::WoodShed(_) | ObjectKind::Table | ObjectKind::Wall)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldObject {
    pub size: ObjectSize,
    pub anchored: bool,
    #[serde(default)]
    pub surface: Option<ObjectSurface>,
    pub kind: ObjectKind,
}

impl WorldObject {
    pub fn new(kind: ObjectKind) -> Self {
        let size = kind.default_size();
        let surface = if kind.supports_surface() {
            Some(ObjectSurface::default())
        } else {
            None
        };
        Self {
            size,
            anchored: true,
            surface,
            kind,
        }
    }

    pub fn display_name(&self) -> String {
        self.kind.name()
    }

    pub fn visibility_range(&self) -> i32 {
        if let Some(override_range) = self.kind.visibility_override() {
            return override_range;
        }
        self.size.visibility_range()
    }

    pub fn as_cabin_mut(&mut self) -> Option<&mut Cabin> {
        match &mut self.kind {
            ObjectKind::Cabin(cabin) => Some(cabin),
            _ => None,
        }
    }

    pub fn as_cabin(&self) -> Option<&Cabin> {
        match &self.kind {
            ObjectKind::Cabin(cabin) => Some(cabin),
            _ => None,
        }
    }

    pub fn as_wood_shed_mut(&mut self) -> Option<&mut WoodShed> {
        match &mut self.kind {
            ObjectKind::WoodShed(shed) => Some(shed),
            _ => None,
        }
    }

    pub fn as_wood_shed(&self) -> Option<&WoodShed> {
        match &self.kind {
            ObjectKind::WoodShed(shed) => Some(shed),
            _ => None,
        }
    }

    pub fn as_tree_mut(&mut self) -> Option<&mut Tree> {
        match &mut self.kind {
            ObjectKind::Tree(tree) => Some(tree),
            _ => None,
        }
    }

    pub fn as_tree(&self) -> Option<&Tree> {
        match &self.kind {
            ObjectKind::Tree(tree) => Some(tree),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacedObject {
    pub id: String,
    pub position: Position,
    pub object: WorldObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObjectRegistry {
    pub placed: Vec<PlacedObject>,
}

impl ObjectRegistry {
    pub fn new() -> Self {
        Self { placed: Vec::new() }
    }

    pub fn add(&mut self, id: impl Into<String>, position: Position, object: WorldObject) {
        let po = PlacedObject {
            id: id.into(),
            position,
            object,
        };
        self.placed.push(po);
    }

    pub fn remove(&mut self, id: &str) -> Option<PlacedObject> {
        if let Some(idx) = self.placed.iter().position(|p| p.id == id) {
            Some(self.placed.remove(idx))
        } else {
            None
        }
    }

    pub fn find_mut(&mut self, id: &str) -> Option<&mut PlacedObject> {
        self.placed.iter_mut().find(|p| p.id == id)
    }

    pub fn find(&self, id: &str) -> Option<&PlacedObject> {
        self.placed.iter().find(|p| p.id == id)
    }

    pub fn objects_at(&self, position: &Position) -> Vec<&PlacedObject> {
        self.placed
            .iter()
            .filter(|p| &p.position == position)
            .collect()
    }

    pub fn objects_at_mut(&mut self, position: &Position) -> Vec<&mut PlacedObject> {
        self.placed
            .iter_mut()
            .filter(|p| &p.position == position)
            .collect()
    }

    pub fn visible_from(&self, origin: &Position) -> Vec<&PlacedObject> {
        self.placed
            .iter()
            .filter(|p| {
                let distance = origin.distance_to(&p.position);
                distance <= p.object.visibility_range() as f32 + 0.01
            })
            .collect()
    }

    pub fn living_tree_count(&self) -> usize {
        self.placed
            .iter()
            .filter(|p| matches!(&p.object.kind, ObjectKind::Tree(tree) if !tree.felled))
            .count()
    }

    pub fn for_each_tree_mut<F>(&mut self, mut func: F)
    where
        F: FnMut(&mut Tree, &Position),
    {
        for placed in &mut self.placed {
            if let ObjectKind::Tree(tree) = &mut placed.object.kind {
                func(tree, &placed.position);
            }
        }
    }

    pub fn find_tree_mut_at(&mut self, position: &Position) -> Option<&mut Tree> {
        self.placed
            .iter_mut()
            .find_map(|p| {
                if &p.position == position {
                    if let ObjectKind::Tree(tree) = &mut p.object.kind {
                        return Some(tree);
                    }
                }
                None
            })
    }

    pub fn find_tree_at(&self, position: &Position) -> Option<&Tree> {
        self.placed
            .iter()
            .find_map(|p| {
                if &p.position == position {
                    if let ObjectKind::Tree(tree) = &p.object.kind {
                        return Some(tree);
                    }
                }
                None
            })
    }
}
