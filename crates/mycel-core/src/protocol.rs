//! Protocol-facing placeholders for the first Rust workspace cut.
//!
//! The real object model, canonical serialization, and validation engine should
//! grow here over time.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolVersion {
    pub core: &'static str,
    pub wire: &'static str,
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self {
            core: "mycel/0.1",
            wire: "mycel-wire/0.1",
        }
    }
}
