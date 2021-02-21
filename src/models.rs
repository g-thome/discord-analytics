pub struct Message {
    pub _id: u64,
    pub author_id: u64,
    pub channel_id: u64,
    pub guild_id: u64,
    pub text: String,
    pub timestamp: String
}

pub struct Channel {
    pub _id: u64,
    pub name: String,
    pub message_count: u64,
}

pub struct ChannelStats {
    pub message_count: u64
}

pub struct Guild {
    pub _id: u64,
    pub message_count: u64,
}

pub struct GuildStats {
    pub message_count: u64
}