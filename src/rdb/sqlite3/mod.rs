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
use clap::App;
use core::cell::RefCell;
use mouse_sqlite3::Connection;
use std::error::Error;
use std::sync::{Condvar, Mutex};
use std::thread::ThreadId;

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
        app
    }

    unsafe fn check(&mut self, _config: &Config) -> Result<(), Box<dyn Error>> {
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
