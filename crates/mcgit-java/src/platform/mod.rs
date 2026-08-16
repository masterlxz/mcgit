#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::candidate_locations;

// TODO: windows.rs and macos.rs — added when implemented/tested on those platforms.
