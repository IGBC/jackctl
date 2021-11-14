

//ifconfig is linux
mod alsa_card;

pub use alsa_card::AlsaHandle as HardwareHandle;

// ifconfig is mac
// mod coraudio;
