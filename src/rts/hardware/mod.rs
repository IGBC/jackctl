//ifconfig is linux
mod alsa_card;

pub use alsa_card::CardId as CardId;
pub use alsa_card::ChannelId as ChannelId;
pub use alsa_card::AlsaHandle as HardwareHandle;

// ifconfig is mac
// mod coraudio;
