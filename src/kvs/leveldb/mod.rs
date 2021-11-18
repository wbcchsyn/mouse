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

use super::{ReadQuery, Row, WriteQuery};
use crate::data_types::{Acid, Id};
use crate::{Config, ModuleEnvironment};
use clap::{App, Arg};
use counting_pointer::Asc;
use spin_sync::Mutex;
use std::borrow::Cow;
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

struct WriteBatch {
    result: Asc<Mutex<PutResult>>,
    intrinsic: mouse_leveldb::WriteBatch,
    extrinsic: mouse_leveldb::WriteBatch,
    len_: usize,
}

impl Default for WriteBatch {
    fn default() -> Self {
        Self {
            result: Asc::from(Mutex::new(PutResult::NotYet)),
            intrinsic: mouse_leveldb::WriteBatch::new(),
            extrinsic: mouse_leveldb::WriteBatch::new(),
            len_: 0,
        }
    }
}

impl WriteBatch {
    pub fn len(&self) -> usize {
        self.len_
    }

    pub fn put(&mut self, id: &Id, intrinsic: &[u8], extrinsic: &[u8]) -> Asc<Mutex<PutResult>> {
        let mut is_changed = false;

        if !intrinsic.is_empty() {
            self.intrinsic.put(id.as_ref(), intrinsic);
            is_changed = true;
        }
        if !extrinsic.is_empty() {
            self.extrinsic.put(id.as_ref(), extrinsic);
            is_changed = true;
        }

        if is_changed {
            self.len_ += 1;
        }

        self.result.clone()
    }

    pub fn flush(&mut self, db: &Db) {
        // Flush extrinsic batch
        {
            let db = &db.extrinsic;
            let res = mouse_leveldb::write(db, &mut self.extrinsic);
            if let Err(e) = res {
                self.set_error(e);
                self.clear();
                return;
            }
        }

        // Flush intrinsic batch
        {
            let db = &db.intrinsic;
            let res = mouse_leveldb::write(db, &mut self.intrinsic);
            if let Err(e) = res {
                self.set_error(e);
                self.clear();
                return;
            }
        }

        // Set the result
        {
            let mut r = self.result.lock().unwrap();
            *r = PutResult::Succeeded;
        }

        self.clear();
    }

    fn set_error(&mut self, e: mouse_leveldb::Error) {
        let mut r = self.result.lock().unwrap();
        *r = PutResult::Error(e);
    }

    fn clear(&mut self) {
        self.result = Asc::from(Mutex::new(PutResult::NotYet));
        self.intrinsic.clear();
        self.extrinsic.clear();
        self.len_ = 0;
    }
}

/// `Environment` implements `ModuleEnvironment` for this module.
#[derive(Default)]
pub struct Environment {
    db_path: PathBuf,
    db: Db,

    max_write_queries: usize,
    write_batch: std::sync::Mutex<WriteBatch>,
}

impl ModuleEnvironment for Environment {
    fn args(app: App<'static, 'static>) -> App<'static, 'static> {
        app.args(&[
            Arg::with_name("PATH_TO_KVS_DB_DIR")
                .help("Path to the KVS Database directory.")
                .long("--kvs-db-path")
                .required(true)
                .takes_value(true),
            Arg::with_name("MAX_WRITE_KVS_QUERIES")
                .help("The max number of writing kvs queries.")
                .long("--max-write-kvs-queries")
                .default_value("128")
                .takes_value(true),
        ])
    }

    unsafe fn check(&mut self, config: &Config) -> Result<(), Box<dyn Error>> {
        let db_path = config.args().value_of("PATH_TO_KVS_DB_DIR").unwrap();
        self.db_path = PathBuf::from(db_path);

        let max_write_queries = config.args().value_of("MAX_WRITE_KVS_QUERIES").unwrap();
        self.max_write_queries = max_write_queries.parse().map_err(|e| {
            Box::<dyn Error>::from(format!(
                "Failed to parse argument '--max-write-kvs-queries': {}",
                e
            ))
        })?;

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

    fn do_fetch(&self) -> FetchResult {
        let intrinsic_db = &self.env.db.intrinsic;
        let intrinsic = match mouse_leveldb::get(intrinsic_db, self.id.as_ref()) {
            Ok(octets) => octets,
            Err(e) => return FetchResult::Err(e),
        };

        if intrinsic.as_ref().is_empty() {
            return FetchResult::NotFound;
        }

        let extrinsic_db = &self.env.db.extrinsic;
        let extrinsic = match mouse_leveldb::get(extrinsic_db, self.id.as_ref()) {
            Ok(octets) => octets,
            Err(e) => return FetchResult::Err(e),
        };

        FetchResult::Found(intrinsic, extrinsic)
    }
}

impl ReadQuery for FetchQuery<'_> {
    fn is_finished(&self) -> bool {
        match self.result {
            FetchResult::NotYet => false,
            _ => true,
        }
    }

    fn wait(&mut self) -> Result<Option<Row>, &dyn Error> {
        if !self.is_finished() {
            self.result = self.do_fetch();
        }

        match &self.result {
            FetchResult::NotYet => panic!("Program never comes here."),
            FetchResult::NotFound => Ok(None),
            FetchResult::Found(intrinsic, extrinsic) => {
                let intrinsic: &[u8] = intrinsic.as_ref();
                let extrinsic: &[u8] = extrinsic.as_ref();
                let row = Row {
                    intrinsic: Cow::Borrowed(intrinsic),
                    extrinsic: Cow::Borrowed(extrinsic),
                };
                Ok(Some(row))
            }
            FetchResult::Err(e) => Err(e),
        }
    }

    fn error(&self) -> Option<&dyn Error> {
        match &self.result {
            FetchResult::Err(e) => Some(e),
            _ => None,
        }
    }
}

/// Returns a new `ReadQuery`
pub fn fetch<'a>(id: &Id, env: &'a Environment) -> impl ReadQuery + 'a {
    FetchQuery::new(id, env)
}

enum PutResult {
    NotYet,
    Succeeded,
    Error(mouse_leveldb::Error),
}

struct PutQuery<'a> {
    env: &'a Environment,
    result: Asc<Mutex<PutResult>>,
}

impl<'a> PutQuery<'a> {
    pub fn new(id: &Id, intrinsic: &[u8], extrinsic: &[u8], env: &'a Environment) -> Self {
        let mut batch = env.write_batch.lock().unwrap();
        let result = batch.put(id, intrinsic, extrinsic);

        if batch.len() <= env.max_write_queries {
            batch.flush(&env.db);
        }

        Self { env, result }
    }
}

impl WriteQuery for PutQuery<'_> {
    fn is_finished(&self) -> bool {
        match &*self.result.lock().unwrap() {
            PutResult::NotYet => false,
            _ => true,
        }
    }

    fn wait(&mut self) -> Result<(), &dyn Error> {
        if !self.is_finished() {
            let mut batch = self.env.write_batch.lock().unwrap();
            if !self.is_finished() {
                batch.flush(&self.env.db);
            }
        }

        match &*self.result.lock().unwrap() {
            PutResult::NotYet => panic!("Never comes here."),
            PutResult::Succeeded => Ok(()),
            PutResult::Error(e) => unsafe {
                let ptr = e as *const dyn Error;
                Err(&*ptr)
            },
        }
    }

    fn error(&self) -> Option<&dyn Error> {
        match &*self.result.lock().unwrap() {
            PutResult::Error(e) => unsafe {
                let ptr = e as *const dyn Error;
                Some(&*ptr)
            },
            _ => None,
        }
    }
}

/// Returns a new `WriteQuery` to put both the intrinsic data and extrinsic data of `acid` .
pub fn insert<'a>(acid: &dyn Acid, env: &'a Environment) -> impl WriteQuery + 'a {
    PutQuery::new(
        acid.id(),
        acid.intrinsic().as_ref(),
        acid.extrinsic().as_ref(),
        env,
    )
}

/// Returns a new `WriteQuery` to put only extrinsic data of `acid` .
///
/// Note that the acid cannot be fetched before the intrinsic data is stored, too.
/// This method is called only when the user is sure that the intrinsic data is already stored
/// to the KVS, and when the user want to update the extrinsic data.
pub fn update<'a>(acid: &dyn Acid, env: &'a Environment) -> impl WriteQuery + 'a {
    PutQuery::new(acid.id(), &[], acid.extrinsic().as_ref(), env)
}
