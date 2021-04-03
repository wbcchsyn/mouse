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

use crate::data_types::Id;
use crate::{Config, ModuleEnvironment};
use clap::{App, Arg};
use std::error::Error;
use std::ffi::CString;
use std::path::PathBuf;

struct Db {
    intrinsic: mouse_leveldb::Database,
    extrinsic: mouse_leveldb::Database,
}

impl Default for Db {
    fn default() -> Self {
        Self {
            intrinsic: mouse_leveldb::Database::new(),
            extrinsic: mouse_leveldb::Database::new(),
        }
    }
}

impl Db {
    pub fn open(&mut self, path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let mut path = path.clone();
        {
            path.push("intrinsic");
            let path = path.to_string_lossy().into_owned().into_bytes();
            let path = CString::new(path).or_else(|e| {
                let err: Box<dyn Error> = Box::from(format!("Failed to open KVS: {}", e));
                Err(err)
            })?;
            self.intrinsic.open(&path)?;
        }

        {
            path.pop();
            path.push("extrinsic");
            let path = path.to_string_lossy().into_owned().into_bytes();
            let path = CString::new(path).or_else(|e| {
                let err: Box<dyn Error> = Box::from(format!("Failed to open KVS: {}", e));
                Err(err)
            })?;
            self.extrinsic.open(&path)?;
        }

        Ok(())
    }
}

/// `Environment` implements `ModuleEnvironment` for this module.
#[derive(Default)]
pub struct Environment {
    db_path: PathBuf,
    db: Db,
}

impl ModuleEnvironment for Environment {
    fn args(app: App<'static, 'static>) -> App<'static, 'static> {
        app.args(&[Arg::with_name("PATH_TO_KVS_DB_DIR")
            .help("Path to the KVS Database directory.")
            .long("--kvs-db-path")
            .required(true)
            .takes_value(true)])
    }

    unsafe fn check(&mut self, config: &Config) -> Result<(), Box<dyn Error>> {
        let db_path = config.args().value_of("PATH_TO_KVS_DB_DIR").unwrap();
        self.db_path = PathBuf::from(db_path);

        Ok(())
    }

    unsafe fn init(&mut self) -> Result<(), Box<dyn Error>> {
        self.db.open(&self.db_path)?;

        Ok(())
    }
}

enum FetchResult {
    NotYet,
    NotFound,
    Found(mouse_leveldb::Octets, mouse_leveldb::Octets),
    Err(mouse_leveldb::Error),
}

struct FetchQuery<'a> {
    env: &'a Environment,
    id: Id,
    result: FetchResult,
}

impl<'a> FetchQuery<'a> {
    pub fn new(id: &Id, env: &'a Environment) -> Self {
        Self {
            id: *id,
            env,
            result: FetchResult::NotYet,
        }
    }
}
