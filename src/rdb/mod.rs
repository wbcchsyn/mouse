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

//! 'rdb' module

pub mod acids;
pub mod main_chain;
pub mod resources;
mod sqlite3;

pub use sqlite3::{Environment, Error};

/// `Session` represents a session to the RDB.
pub trait Session {
    /// Returns `true` if the current session is in transaction.
    fn is_transaction(&self) -> bool;

    /// Starts transaction if not started.
    ///
    /// # Panics
    ///
    /// Panics if `self` is in transaction.
    fn begin_transaction(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Commits transaction.
    ///
    /// # Panics
    ///
    /// Panics if `self` is not in transaction.
    fn commit(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Rollback transaction.
    ///
    /// # Panics
    ///
    /// Panics if `self` is not in transaction.
    fn rollback(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

/// Represents a session to a slave RDB.
pub trait Slave: Session {}

/// Represents a session to a master RDB.
pub trait Master: Session + Slave {}

/// Creates a new instance implementing [`Master`] .
///
/// # Panics
///
/// Panics if the current thread owns another [`Session`] instance.
/// This feature is to escape a dead lock.
pub fn master<'a>(env: &'a Environment) -> impl 'a + Master {
    sqlite3::master(env)
}

/// Creates a new instance implementing [`Slave`] .
///
/// # Panics
///
/// Panics if the current thread owns another [`Session`] instance.
/// This feature is to escape a dead lock.
pub fn slave<'a>(env: &'a Environment) -> impl 'a + Slave {
    sqlite3::slave(env)
}
