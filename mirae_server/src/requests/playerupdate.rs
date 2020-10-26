pub enum PlayerUpdate {
    Text(Text),
    Upgrade(Stat),
    
}

pub enum Stat {
    Health,
    Speed,
    Energy,
    View
}

pub enum Text {
    TextDisplay(TextDisplay),
    TextBroadcast(TextBroadcast)
}

pub struct TextDisplay {
    pub player : u8,
    pub text : String
}

pub struct TextBroadcast {
    pub text : String
}