//! Tests for individual pages
//!
//! Each submodule tests a specific page's functionality.

mod common;
mod pages {
    pub mod home;

    // Dynamic services (currently todo!() placeholders)
    pub mod trent;
    pub mod pastebin;
    pub mod passgen;
    pub mod html_sanitize;
    pub mod big_number_calculator;
    pub mod online_x86_assembler;
    pub mod quantum_time_capsule;
    pub mod ip;
    pub mod password_blocks;
}
