use crate::location::Vector2;

pub struct EntityMove {
    pub start: Vector2,
    pub end: Vector2,
}

pub struct EntityDel {
    pub loc: Vector2,
}

pub struct EntitySpawn {
    pub loc: Vector2,
    pub name: String,
}

pub enum WorldEntityUpdate {
    Move(EntityMove),
    Del(EntityDel),
    Spawn(EntitySpawn),
}

pub struct WorldBlockUpdate {
    pub loc: Vector2,
    pub blockname: Option<String>,
}

pub enum WorldUpdate {
    WorldEntityUpdate(WorldEntityUpdate),
    WorldBlockUpdate(WorldBlockUpdate)
}