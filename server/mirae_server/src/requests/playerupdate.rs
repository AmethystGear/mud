use crate::playerout::PlayerOut;

pub enum PlayerUpdate {
    Display(PlayerDisplay),
    Upgrade(PlayerStat),
}

pub enum PlayerStat {
    Health,
    Speed,
    Energy,
    View,
}

pub enum Display {
    PlayerDisplay(PlayerDisplay),
    PlayerBroadcast,
}

pub struct PlayerDisplay {
    pub player: u8,
    pub data: PlayerOut,
}

pub struct PlayerBroadcast {
    pub text: PlayerOut,
}
