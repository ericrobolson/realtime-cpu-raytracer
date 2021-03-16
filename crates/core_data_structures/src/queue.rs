#[derive(Debug)]
/// Creates a new Queue
pub struct Queue<T> {
    items: Vec<T>,
}

impl<T> Queue<T> {
    /// Creates a new queue
    pub fn new(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }

    /// Pushes an item onto the queue
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// Returns a slice of the queued items
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Pops an item off the queue
    pub fn pop(&mut self) -> Option<T> {
        if self.items.len() == 0 {
            None
        } else {
            let item = self.items.remove(0);
            Some(item)
        }
    }

    /// Clears the collection.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Returns whether the queue is empty or not.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the length of the collection
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_clear() {
        let capacity = 123;
        let mut queue: Queue<bool> = Queue::new(capacity);

        queue.push(true);
        queue.push(false);
        queue.push(true);
        queue.push(false);

        queue.clear();
        let expected_items: Vec<bool> = vec![];
        assert_eq!(expected_items, queue.items);

        assert_eq!(0, queue.len());
    }

    #[test]
    fn queue_len() {
        let capacity = 123;
        let mut queue: Queue<bool> = Queue::new(capacity);
        let expected_items: Vec<bool> = vec![true, false, true, false];

        queue.push(true);
        queue.push(false);
        queue.push(true);
        queue.push(false);

        assert_eq!(4, queue.len());
    }

    #[test]
    fn queue_new() {
        let capacity = 123;
        let queue: Queue<bool> = Queue::new(capacity);
        let expected_items: Vec<bool> = vec![];

        assert_eq!(capacity, queue.items.capacity());
        assert_eq!(expected_items, queue.items);
    }

    #[test]
    fn queue_push() {
        let capacity = 123;
        let mut queue: Queue<bool> = Queue::new(capacity);
        let expected_items: Vec<bool> = vec![true, false, true, false];

        queue.push(true);
        queue.push(false);
        queue.push(true);
        queue.push(false);

        assert_eq!(expected_items, queue.items);
    }

    #[test]
    fn queue_pop() {
        let capacity = 123;
        let mut queue: Queue<bool> = Queue::new(capacity);

        queue.push(true);
        queue.push(false);
        queue.push(true);
        queue.push(false);

        assert_eq!(vec![true, false, true, false], queue.items);

        let item = queue.pop();
        assert_eq!(Some(true), item);
        assert_eq!(vec![false, true, false], queue.items);

        let item = queue.pop();
        assert_eq!(Some(false), item);
        assert_eq!(vec![true, false], queue.items);

        let item = queue.pop();
        assert_eq!(Some(true), item);
        assert_eq!(vec![false], queue.items);

        let item = queue.pop();
        assert_eq!(Some(false), item);
        assert_eq!(0, queue.items.len());

        let item = queue.pop();
        assert_eq!(None, item);
        assert_eq!(0, queue.items.len());
    }
}
