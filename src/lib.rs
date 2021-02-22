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

#![deny(missing_docs)]

//! `Mouse` is a Blockchain framework.

use clap::{App, ArgMatches};

/// `Config` is a wrapper of [`clap::ArgMatches<'static>`] .
///
/// `Mouse` uses [`clap`] for an argument parser.
/// See also [`clap`] for details.
///
/// [`clap`]: /clap/index.html
/// [`clap::ArgMatches<'static>`]: /clap/struct.ArgMatches.html
pub struct Config {
    args_: ArgMatches<'static>,
    name_: String,
}

impl Config {
    /// Parses the argument and creates a new instance.
    ///
    /// This function parses arguments for `Mouse` by default.
    /// If user want to add some arguments, set them to `app` .
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[macro_use] extern crate clap;
    ///
    /// use clap::{App, Arg};
    /// use mouse::Config;
    ///
    /// // Initialize app
    /// let app = App::new(crate_name!())
    ///     .version(crate_version!())
    ///     .about(crate_description!());
    ///
    /// // Add argument '--foo'
    /// let app = app.arg(
    ///     Arg::with_name("foo")
    ///         .help("Argument 'foo' (Default is 'bar'.)")
    ///         .long("foo")
    ///         .takes_value(true),
    /// );
    ///
    /// // Creates 'Config'.
    /// let config = Config::new(app);
    /// ```
    pub fn new(app: App<'static, 'static>) -> Self {
        let name = String::from(app.get_name());
        Config {
            args_: app.get_matches(),
            name_: name,
        }
    }

    /// Provides a reference to the wrapped value.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[macro_use] extern crate clap;
    ///
    /// use clap::{App, Arg};
    /// use mouse::Config;
    ///
    /// // Initialize app
    /// let app = App::new(crate_name!())
    ///     .version(crate_version!())
    ///     .about(crate_description!());
    ///
    /// // Add argument '--foo'
    /// let app = app.arg(
    ///     Arg::with_name("foo")
    ///         .help("Argument 'foo' (Default is 'bar'.)")
    ///         .long("foo")
    ///         .takes_value(true),
    /// );
    ///
    /// let config = Config::new(app);
    /// let foo = config.args().value_of("foo");
    ///
    /// // Check the value of argument '--foo'.
    /// // If the user specifies argument '--foo', the reuslt will be 'Some' value.
    /// // ('None' means the argument is not passed.)
    /// assert_eq!(None, foo);
    /// ```
    pub fn args(&self) -> &ArgMatches<'static> {
        &self.args_
    }

    /// Provides a reference to the name of app.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[macro_use] extern crate clap;
    ///
    /// use clap::{App, Arg};
    /// use mouse::Config;
    ///
    /// // Initialize app
    /// let app = App::new(crate_name!())
    ///     .version(crate_version!())
    ///     .about(crate_description!());
    ///
    /// // Creates 'Config'.
    /// let config = Config::new(app);
    ///
    /// assert_eq!(crate_name!(), config.name());
    /// ```
    pub fn name(&self) -> &str {
        &self.name_
    }
}
