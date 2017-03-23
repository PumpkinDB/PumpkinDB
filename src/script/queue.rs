#[derive(Debug)]
pub struct Queue<T> {
    max_size: usize,
    count: usize,
    elements: Vec<T>
}

#[derive(Debug, PartialEq)]
pub enum InsertError {
    Full
}

impl<T> Queue<T> {
    pub fn new(max_size: usize) -> Self {
        Queue {
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
    use script::queue::{Queue, InsertError};

    #[test]
    fn create_queue() {
        let queue: Queue<String> = Queue::new(usize::max_value());
        assert_eq!(queue.capacity(), usize::max_value());
    }

    #[test]
    fn max_size() {
        let mut queue: Queue<String> = Queue::new(1);
        assert_eq!(queue.push("a".to_owned()).unwrap(), 1);
        assert_eq!(queue.push("a".to_owned()).unwrap_err(), InsertError::Full);
    }

    #[test]
    fn push_pop() {
        let mut queue: Queue<String> = Queue::new(10);
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.push("a".to_owned()).unwrap(), 1);
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.pop().unwrap(), "a".to_owned());
        assert_eq!(queue.len(), 0);
    }

}