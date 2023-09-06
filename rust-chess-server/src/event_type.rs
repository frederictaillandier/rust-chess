pub enum EventType {
    PlayerConnect(Uid),
    PlayerDisconnect(Uid),
    PlayerSay(Uid, String),
    PlayerPlay(Uid, u8, u8, u8, u8),
}

type Uid = u32;
