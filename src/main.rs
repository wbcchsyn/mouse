// Copyright 2020 Shin Yoshida
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

#[macro_use]
extern crate clap;

use clap::{App, ArgMatches};

pub struct GlobalConfig {
    args_: ArgMatches<'static>,
}

impl GlobalConfig {
    /// Accessor to the arguments.
    pub fn args(&self) -> &ArgMatches<'static> {
        &self.args_
    }
}

/// Parses the arguments and returns GlobalConfig.
fn parse_argument() -> GlobalConfig {
    let app = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!());

    GlobalConfig {
        args_: app.get_matches(),
    }
}

fn run(_config: GlobalConfig) {}

fn main() {}
