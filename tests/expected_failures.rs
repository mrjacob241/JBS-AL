//! Executable backlog for known JBS gaps.
//!
//! These tests are intentionally ignored so the default `cargo test` remains a
//! green signal. Run them with:
//!
//! ```sh
//! cargo test --test expected_failures -- --ignored
//! ```
//!
//! When a test starts passing, remove `#[ignore]` and move it to the normal
//! kernel tests or a dedicated test262-facing suite.
//!
//! The Block 0-9 stabilization backlog is currently empty.
