pub struct EntityMove {
    pub x_start: u16,
    pub y_start: u16,
    pub x_to: u16,
    pub y_to: u16,
}

pub struct EntityDel {
    pub x: u16,
    pub y: u16,
}

pub struct EntitySpawn {
    pub x: u16,
    pub y: u16,
    pub name: String,
}

pub enum EntityUpdate {
    Move(EntityMove),
    Del(EntityDel),
    Spawn(EntitySpawn),
}

pub struct BlockUpdate {
    pub x: u16,
    pub y: u16,
    pub blockname: String,
}
