// SPDX-License-Identifier: GPL-3.0-only

pub mod bar;
pub mod hotel;
pub mod management;

pub use bar::Bar;
pub use hotel::Hotel;
pub use management::Management;

pub enum Screen {
    Welcome,
    Bar(Bar),
    Hotel(Hotel),
    Management(Management),
}
