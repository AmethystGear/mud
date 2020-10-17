use crate::deser::item::ItemName;
use std::collections::HashMap;

pub struct Trades(pub HashMap<ItemName, ItemName>);
