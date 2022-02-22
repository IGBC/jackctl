//ifconfig is linux
mod alsa_card;

pub use alsa_card::AlsaHandle as HardwareHandle;
pub use alsa_card::CardId;
pub use alsa_card::ChannelId;

// ifconfig is mac
// mod coraudio;
