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

use super::{Master, Session, Slave};
use crate::{Config, ModuleEnvironment};
use clap::{App, Arg};
use core::cell::RefCell;
use core::convert::TryFrom;
use mouse_sqlite3::Connection;
use std::error::Error;
use std::path::PathBuf;
use std::sync::{Condvar, Mutex};
use std::thread::{self, ThreadId};

/// `Environment` implements `ModuleEnvironment` for this module.
pub struct Environment {
    session_owner: (Mutex<Option<ThreadId>>, Condvar),
    connection: RefCell<Connection>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            session_owner: (Mutex::new(None), Condvar::new()),
            connection: RefCell::new(Connection::open_memory_db().unwrap()),
        }
    }
}

impl ModuleEnvironment for Environment {
    fn args(app: App<'static, 'static>) -> App<'static, 'static> {
        app.arg(
            Arg::with_name("PATH_TO_SQLITE3_DATA_DIR")
                .help("Path to the Sqlite3 data directory.")
                .long("--sqlite3-data-path")
                .required(true)
                .takes_value(true),
        )
    }

    unsafe fn check(&mut self, config: &Config) -> Result<(), Box<dyn Error>> {
        let data_path = config.args().value_of("PATH_TO_SQLITE3_DATA_DIR").unwrap();
        let data_path = PathBuf::from(data_path).join("rdb");
        let connection = Connection::try_from(data_path.as_ref())?;
        self.connection = RefCell::new(connection);

        Ok(())
    }

    unsafe fn init(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

/// Implementation for trait `data_types::rdb::Session` .
pub struct Sqlite3Session<'a> {
    env: &'a Environment,
    connection: &'a mut Connection,
    is_transaction: bool,
}

impl Drop for Sqlite3Session<'_> {
    fn drop(&mut self) {
        // For just in case.
        // This method returns `Err` if not in transaction.
        // Ignore the error.
        let _ = self.do_rollback();

        // Release the ownership
        let (mtx, cond) = &self.env.session_owner;
        let mut guard = mtx.lock().unwrap();
        *guard = None;
        cond.notify_one()
    }
}

impl<'a> Sqlite3Session<'a> {
    /// Waits if another thread is using the connection, and creates a new session.
    ///
    /// # Panics
    ///
    /// Panic if the current thread is using the connection.
    fn new(env: &'a Environment) -> Self {
        let current_id = thread::current().id();
        let (mtx, cond) = &env.session_owner;
        let mut guard = mtx.lock().unwrap();

        // Wait for the ownership
        loop {
            if *guard == None {
                // No other thread is using the connection

                let connection = unsafe { &mut *env.connection.as_ptr() };
                let mut ret = Self {
                    env,
                    connection,
                    is_transaction: false,
                };

                // For just in case.
                // This method returns `Err` if not in transaction.
                // Ignore the error.
                let _ = ret.do_rollback();

                *guard = Some(current_id);
                return ret;
            } else if *guard == Some(current_id) {
                // The current thread itself is using the connection.
                drop(guard); // Don't poison the mutex.
                panic!("Current thread tries to acquire 2 RDB connections at the same time.");
            } else {
                guard = cond.wait(guard).unwrap();
            }
        }
    }
}

impl Session for Sqlite3Session<'_> {
    fn is_transaction(&self) -> bool {
        self.is_transaction
    }

    fn begin_transaction(&mut self) -> Result<(), Box<dyn Error>> {
        assert_eq!(false, self.is_transaction);
        self.do_begin_transaction()
    }

    fn commit(&mut self) -> Result<(), Box<dyn Error>> {
        assert_eq!(true, self.is_transaction);
        self.do_commit()
    }

    fn rollback(&mut self) -> Result<(), Box<dyn Error>> {
        assert_eq!(true, self.is_transaction);
        self.do_rollback()
    }
}

impl Slave for Sqlite3Session<'_> {}

impl Master for Sqlite3Session<'_> {}

impl Sqlite3Session<'_> {
    fn do_begin_transaction(&mut self) -> Result<(), Box<dyn Error>> {
        const SQL: &'static str = "BEGIN";
        let stmt = self.connection.stmt(SQL).map_err(Box::new)?;
        stmt.step().map_err(Box::new)?;

        self.is_transaction = true;
        Ok(())
    }

    fn do_commit(&mut self) -> Result<(), Box<dyn Error>> {
        const SQL: &'static str = "COMMIT";
        let stmt = self.connection.stmt(SQL).map_err(Box::new)?;
        stmt.step().map_err(Box::new)?;

        self.is_transaction = false;
        Ok(())
    }

    fn do_rollback(&mut self) -> Result<(), Box<dyn Error>> {
        const SQL: &'static str = "ROLLBACK";
        let stmt = self.connection.stmt(SQL).map_err(Box::new)?;
        stmt.step().map_err(Box::new)?;

        self.is_transaction = false;
        Ok(())
    }
}

/// Waits if another thread is using the connection, and creates a new session to master rdb.
///
/// # Panics
///
/// Panic if the current thread is using the connection.
pub fn master<'a>(env: &'a Environment) -> impl 'a + Master + Slave + Session {
    Sqlite3Session::new(env)
}

/// Waits if another thread is using the connection, and creates a new session to slave rdb.
///
/// # Panics
///
/// Panic if the current thread is using the connection.
pub fn slave<'a>(env: &'a Environment) -> impl 'a + Slave + Session {
    Sqlite3Session::new(env)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_session() {
        let env = Environment::default();
        let _session = Sqlite3Session::new(&env);
    }
}
