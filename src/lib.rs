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
use std::error::Error;
use std::os::raw::c_int;

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

/// Initializes mouse, starts to listen to the user requests, and waits for the signal.
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    {
        let mut environment = Environment::default();
        unsafe { environment.check(&config) }?;
        unsafe { environment.init() }?;

        unsafe {
            if sigwait_() != 0 {
                let msg = errno::errno().to_string();
                return Err(Box::from(msg));
            }
        }

        // 'environment' is dropped here.
    }

    Ok(())
}

#[link(name = "mouse_signal")]
extern "C" {
    /// Waits for signals 'SIGHUP' or 'SIGTERM' or 'SIGINT' and returns `0` on success, or `1`.
    ///
    /// 'errno' will be set on error.
    fn sigwait_() -> c_int;
}

/// `ModuleEnvironment` represents a set of the followings for each module.
///
/// - Connection to the outside of the process, DataBase connection, socket to listen to the user
///   requests, files, and so on.
/// - Functions that mouse user specifies.
pub trait ModuleEnvironment: Default {
    /// Consumes `App` , adding arguments for the module uses.
    fn args(_app: App<'static, 'static>) -> App<'static, 'static> {
        panic!("Not implemented yet.");
    }

    /// Sanitises the arguments and overwrite properties.
    ///
    /// # Safety
    ///
    /// The behavior is undefined if called after method [`init`] is called.
    ///
    /// [`init`]: #method.init
    unsafe fn check(&mut self, _config: &Config) -> Result<(), Box<dyn Error>> {
        panic!("Not implemented yet.");
    }

    /// Initializes `self` and makes `self` ready for use.
    /// (Open the DataBase Connections, and so on.)
    ///
    /// # Safety
    ///
    /// The behavior is undefined if this method is called twice or more than twice.
    unsafe fn init(&mut self) -> Result<(), Box<dyn Error>> {
        panic!("Not implemented yet.");
    }
}

/// A set of `ModuleEnvironment` instances for all the module.
pub struct Environment {
    // !!! Warnings
    // !! The order of the property is important, because they are dropped in this order.
    // !! Method 'check()' and  'init()' treat the properties in the reverse order.
    // !!
    // !! See Rust-RFC 1857 for details.
    // !! https://github.com/rust-lang/rfcs/blob/master/text/1857-stabilize-drop-order.md
}

impl Default for Environment {
    fn default() -> Self {
        Self {}
    }
}

impl Environment {
    /// Calls method [`ModuleEnvironment.check`] for each property.
    ///
    /// # Safety
    ///
    /// The behavior is undefined if this method is called after method [`init`] is called.
    ///
    /// [`init`]: #method.init
    /// [`ModuleEnvironment.check`]: struct.ModuleEnvironment.html#method.check
    pub unsafe fn check(&mut self, _config: &Config) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// Calls method [`ModuleEnvironment.init`] for each property.
    ///
    /// # Safety
    ///
    /// The behavior is undefined if this method is called twice or more than twice.
    ///
    /// [`ModuleEnvironment.init`]: struct.ModuleEnvironment.html#method.init
    pub unsafe fn init(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
