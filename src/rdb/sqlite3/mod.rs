// Copyright 2021 Shin Yoshida
//
// This file is part of Mouse.
//
// Mouse is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License.
//
// Mouse is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Mouse.  If not, see <https://www.gnu.org/licenses/>.

mod error;

use crate::{Config, ModuleEnvironment};
use clap::App;

pub use error::Error;

/// `Environment` implements `ModuleEnvironment` for this module.
#[derive(Default)]
pub struct Environment {}

impl ModuleEnvironment for Environment {
    fn args(app: App<'static, 'static>) -> App<'static, 'static> {
        app
    }

    unsafe fn check(&mut self, _config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    unsafe fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
