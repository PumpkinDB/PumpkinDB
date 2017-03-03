use std::sync::Mutex;
use lmdb;

pub struct Database<'a> {
    pub db: lmdb::Database<'a>,
    pub gwl: Mutex<()>
}

pub trait GlobalWriteLock {
    fn try_lock(&self) -> bool;
}

impl<'a> GlobalWriteLock for Database<'a> {
    fn try_lock(&self) -> bool {
        match self.gwl.try_lock() {
            Ok(_) => true,
            Err(_) => false
        }
    }
}

impl<'a> Database<'a> {
    pub fn new(env: &'a lmdb::Environment) -> Database<'a> {
        return Database {
            db: lmdb::Database::open(env,
                                     None,
                                     &lmdb::DatabaseOptions::new(lmdb::db::CREATE))
                .expect("can't open database"),
            gwl: Mutex::new(())
        }
    }
}