//! SigmaOS Nightlight
//! Nightlight blue light filter

pub mod nightlight;

pub use nightlight::{
    ColorTemperature, NightlightManager, NightlightManager as Nightlight, NightlightSchedule,
};
