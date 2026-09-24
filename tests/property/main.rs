//! Content tests for the example mod — property. Bolero only.
//!
//! They run natively against the same registration the wasm guest hands the host
//! (`__stormlight_registration`), so a broken declaration fails here, in a test
//! with a name, rather than as a unit that silently does nothing in a match.

mod balance;
mod loadout;
mod payload;
