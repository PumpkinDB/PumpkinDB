#[derive(Debug)]
pub struct TxnStack<T> {
    max_size: usize,
    count: usize,
    elements: Vec<T>
}

#[derive(Debug, PartialEq)]
pub enum InsertError {
    Full
}

impl<T> TxnStack<T> {
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
    use script::txn_stack::{TxnStack, InsertError};

    #[test]
    fn create_txn_stack() {
        let txn_stack: TxnStack<String> = TxnStack::new(usize::max_value());
        assert_eq!(txn_stack.capacity(), usize::max_value());
    }

    #[test]
    fn max_size() {
        let mut txn_stack: TxnStack<String> = TxnStack::new(1);
        assert_eq!(txn_stack.push("a".to_owned()).unwrap(), 1);
        assert_eq!(txn_stack.push("a".to_owned()).unwrap_err(), InsertError::Full);
    }

    #[test]
    fn push_pop() {
        let mut txn_stack: TxnStack<String> = TxnStack::new(10);
        assert_eq!(txn_stack.len(), 0);
        assert_eq!(txn_stack.push("a".to_owned()).unwrap(), 1);
        assert_eq!(txn_stack.len(), 1);
        assert_eq!(txn_stack.pop().unwrap(), "a".to_owned());
        assert_eq!(txn_stack.len(), 0);
    }

}