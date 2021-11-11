//ifconfig is linux
mod alsa;

pub use alsa::AlsaHandle as HardwareHandle;

// ifconfig is mac
// mod coraudio;

pub enum HardwareCmd {}

pub enum HardwareEvent {}

pub enum HarwareCardAction {}
