// SPDX-License-Identifier: GPL-3.0-only

/// Holds the pagination state (generic, for various entities)
#[derive(Debug, Clone)]
pub struct PaginationConfig {
    pub items_per_page: i32,
    pub current_page: i32,
}

impl Default for PaginationConfig {
    fn default() -> Self {
        PaginationConfig {
            items_per_page: 13,
            current_page: 0,
        }
    }
}

/// Identifies a pagination action
#[derive(Debug, Clone, PartialEq)]
pub enum PaginationAction {
    Up,
    Down,

    Back,
    Forward,
}
