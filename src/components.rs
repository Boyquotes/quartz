use bevy::{
    ecs::system::Command,
    prelude::*};
use fundsp::hacker32::*;
// -------------------- components --------------------
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Op(pub i32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Num(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Arr(pub Vec<f32>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Radius(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Col(pub Color);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Selected;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Visible;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Order(pub usize);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct OpChanged(pub bool);

#[derive(Component)]
pub struct Network(pub Net32);

#[derive(Component)]
pub struct NetIns(pub Vec<Shared<f32>>);

#[derive(Component)]
pub struct Save;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct WhiteHole {
    pub bh: Entity,
    pub bh_parent: Entity,
    pub link_types: (i32, i32), //(black, white)
    pub new_lt: bool,
    pub open: bool,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct BlackHole {
    pub wh: Entity,
}

impl Default for WhiteHole {
    fn default() -> Self {
        WhiteHole {
            bh: Entity::PLACEHOLDER,
            bh_parent: Entity::PLACEHOLDER,
            link_types: (0, 0),
            new_lt: true,
            open: true,
        }
    }
}

impl Default for BlackHole {
    fn default() -> Self {
        BlackHole {
            wh: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component)]
pub struct CommandText;

// -------------------- states --------------------
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum Mode {
    Draw,
    Connect,
    #[default]
    Edit,
}

// -------------------- resources --------------------
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct Queue(pub Vec<Vec<Entity>>);

// initial, final, delta
#[derive(Resource, Default)]
pub struct CursorInfo {
    pub i: Vec2,
    pub f: Vec2,
    pub d: Vec2,
}

#[derive(Resource)]
pub struct Slot(pub Slot32, pub Slot32);

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct DragModes {
    pub t: bool,
    pub r: bool,
    pub n: bool,
    pub h: bool,
    pub s: bool,
    pub l: bool,
    pub a: bool,
}
impl DragModes {
    pub fn falsify(&mut self) {
        self.t = false;
        self.r = false;
        self.n = false;
        self.h = false;
        self.s = false;
        self.l = false;
        self.a = false;
    }
}

// -------------------- events --------------------
#[derive(Event, Default)]
pub struct OrderChange;

#[derive(Event)]
pub struct ColorChange(pub Entity, pub Color);

#[derive(Event)]
pub struct RadiusChange(pub Entity, pub f32);

#[derive(Event)]
pub struct OpChange(pub Entity, pub i32);

#[derive(Event, Default)]
pub struct SceneLoaded;

// -------------------- commands --------------------
pub struct DespawnCircle(pub Entity);
impl Command for DespawnCircle {
    fn apply(self, world: &mut World) {
        despawn_circle(self.0, world);
    }
}
fn despawn_circle(entity: Entity, world: &mut World) {
    if world.get_entity(entity).is_none() { return; }
    if let Some(mirror) = get_mirror_hole(entity, world) {
        world.entity_mut(entity).despawn_recursive();
        world.entity_mut(mirror).despawn_recursive();
    } else {
        if let Some(children) = world.entity(entity).get::<Children>() {
            let children = children.to_vec();
            for child in children {
                if let Some(mirror) = get_mirror_hole(child, world) {
                    world.entity_mut(child).despawn_recursive();
                    world.entity_mut(mirror).despawn_recursive();
                }
            }
        }
        world.entity_mut(entity).despawn_recursive();
    }
}
fn get_mirror_hole(entity: Entity, world: &World) -> Option<Entity> {
    let e = world.entity(entity);
    if let Some(wh) = e.get::<WhiteHole>() {
        return Some(wh.bh);
    } else if let Some(bh) = e.get::<BlackHole>() {
        return Some(bh.wh);
    } else {
        return None;
    }
}
