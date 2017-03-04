use std::sync::Mutex;
use lmdb;

pub struct Storage<'a> {
    pub db: lmdb::Database<'a>,
    pub env: & 'a lmdb::Environment,
    pub gwl: Mutex<()>
}

pub trait GlobalWriteLock {
    fn try_lock(&self) -> bool;
}

impl<'a> GlobalWriteLock for Storage<'a> {
    fn try_lock(&self) -> bool {
        self.gwl.try_lock().is_ok()
    }
}

impl<'a> Storage<'a> {
    pub fn new(env: &'a lmdb::Environment) -> Storage<'a> {
        return Storage {
            env: env,
            db: lmdb::Database::open(env,
                                     None,
                                     &lmdb::DatabaseOptions::new(lmdb::db::CREATE))
                .expect("can't open database"),
            gwl: Mutex::new(())
        }
    }
}