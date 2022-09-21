use crate::events::{PatternEvent, PatternEventIter};

// -------------------------------------------------------------------------------------------------

/// Wraps a a plain iterator of [`PatternEvent`] events into a [`PatternEventIter`].
pub struct PatternEventIterFromIter<Iter>
where
    Iter: Iterator<Item = PatternEvent>,
{
    iter: Iter,
    initial_iter: Iter,
}

impl<Iter> PatternEventIterFromIter<Iter>
where
    Iter: Iterator<Item = PatternEvent> + Clone,
{
    pub fn new(iter: Iter) -> Self {
        let initial_iter = iter.clone();
        Self { iter, initial_iter }
    }
}

impl<Iter> Iterator for PatternEventIterFromIter<Iter>
where
    Iter: Iterator<Item = PatternEvent>,
{
    type Item = PatternEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<Iter> PatternEventIter for PatternEventIterFromIter<Iter>
where
    Iter: Iterator<Item = PatternEvent> + Clone,
{
    fn reset(&mut self) {
        self.iter = self.initial_iter.clone();
    }
}
