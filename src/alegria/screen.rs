// SPDX-License-Identifier: GPL-3.0-only

pub mod bar;

pub use bar::Bar;

pub enum Screen {
    Welcome,
    Bar(Bar),
}
