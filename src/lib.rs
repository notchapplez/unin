pub mod registry;
pub use registry::*;

pub mod comms;
mod uniconf;

pub use comms::*;
pub fn setup() {
    let _ = registry_exists();
}
