pub mod worldupdate;
use worldupdate::{BlockUpdate, EntityUpdate};

pub enum Update {
    BlockUpdate(BlockUpdate),
    EntityUpdate(EntityUpdate),
}
