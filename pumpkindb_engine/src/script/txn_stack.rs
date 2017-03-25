use lmdb;
use lmdb::traits::{LmdbResultExt, AsLmdbBytes, FromLmdbBytes};

#[derive(PartialEq, Debug)]
pub enum TxType {
    Read,
    Write,
}

pub enum Accessor<'a> {
    Const(lmdb::ConstAccessor<'a>),
    Write(lmdb::WriteAccessor<'a>),
}

#[derive(Debug)]
pub enum Txn<'a> {
    Read(lmdb::ReadTransaction<'a>),
    Write(lmdb::WriteTransaction<'a>),
}

impl<'a> Accessor<'a> {
    pub fn get<K: AsLmdbBytes + ?Sized, V: FromLmdbBytes + ?Sized>(&self, db: &lmdb::Database, key: &K)
        -> Result<Option<&V>, lmdb::Error> {
        match self {
            &Accessor::Write(ref access) => {
                access.get::<K, V>(db, key)
            },
            &Accessor::Const(ref access) => {
                access.get::<K, V>(db, key)
            }
        }.to_opt()
    }
}

impl<'a> Txn<'a> {
    pub fn access(&self) -> Accessor {
        match self {
            &Txn::Read(ref txn) => Accessor::Const(txn.access()),
            &Txn::Write(ref txn) => Accessor::Write(txn.access()),
        }
    }
    pub fn cursor(&self, db: &'a lmdb::Database) -> Result<lmdb::Cursor, lmdb::Error> {
        match self {
            &Txn::Read(ref txn) => txn.cursor(db),
            &Txn::Write(ref txn) => txn.cursor(db),
        }
    }
}

impl<'a> TxnT for Txn<'a> {
    fn tx_type(&self) -> TxType {
        match self {
            &Txn::Read(_) => TxType::Read,
            &Txn::Write(_) => TxType::Write,
        }
    }
}

pub trait TxnT {
    fn tx_type(&self) -> TxType;
}

#[derive(Debug)]
pub struct TxnStack<T: TxnT> {
    max_size: usize,
    count: usize,
    elements: Vec<T>
}

#[derive(Debug, PartialEq)]
pub enum InsertError {
    Full
}

impl<T: TxnT> TxnStack<T> {
    pub fn new(max_size: usize) -> Self {
        TxnStack {
            max_size: max_size,
            count: 0,
            elements: Vec::new()
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn capacity(&self) -> usize {
        self.max_size
    }

    pub fn push(&mut self, el: T) -> Result<usize, InsertError> {
        if self.count >= self.max_size {
            return Err(InsertError::Full)
        }
        self.count += 1;
        self.elements.push(el);
        Ok(self.count)
    }

    pub fn peek(&self) -> Option<&T> {
        self.elements.last()
    }

    pub fn pop_tx_type(&mut self, txn_type: TxType) -> Option<T> {
        match self.peek()
            .and_then(|txn| Some(txn.tx_type() == txn_type))
            .unwrap_or(false) {
            true => self.pop(),
            false => None
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        match self.elements.pop() {
            Some(el) => {
                self.count -= 1;
                Some(el)
            },
            None => None
        }
    }
}

#[cfg(test)]
mod tests {
    use script::txn_stack::{TxnStack, InsertError, TxnT, TxType};

    impl TxnT for String {
        fn tx_type(&self) -> TxType {
            match self.as_str() {
                "read" => TxType::Read,
                "write" => TxType::Write,
                _ => panic!("Should never get here")
            }
        }
    }

    #[test]
    fn create_txn_stack() {
        let txn_stack: TxnStack<String> = TxnStack::new(usize::max_value());
        assert_eq!(txn_stack.capacity(), usize::max_value());
    }

    #[test]
    fn max_size() {
        let mut txn_stack: TxnStack<String> = TxnStack::new(1);
        assert_eq!(txn_stack.push("read".to_owned()).unwrap(), 1);
        assert_eq!(txn_stack.push("read".to_owned()).unwrap_err(), InsertError::Full);
    }

    #[test]
    fn push_pop() {
        let mut txn_stack: TxnStack<String> = TxnStack::new(10);
        assert_eq!(txn_stack.len(), 0);
        assert_eq!(txn_stack.push("read".to_owned()).unwrap(), 1);
        assert_eq!(txn_stack.len(), 1);
        assert_eq!(txn_stack.pop().unwrap(), "read".to_owned());
        assert_eq!(txn_stack.len(), 0);
    }

    #[test]
    fn pop_tx_type() {
        let mut txn_stack: TxnStack<String> = TxnStack::new(10);
        assert_eq!(txn_stack.push("read".to_owned()).unwrap(), 1);
        assert_eq!(txn_stack.push("write".to_owned()).unwrap(), 2);
        assert_eq!(txn_stack.pop_tx_type(TxType::Write).unwrap(), "write".to_owned());
        assert_eq!(txn_stack.pop_tx_type(TxType::Read).unwrap(), "read".to_owned());
    }
}