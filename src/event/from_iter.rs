use crate::event::{Event, EventIter};

// -------------------------------------------------------------------------------------------------

/// Wraps a a plain iterator of [`Event`] events into a [`EventIter`].
pub struct FromIter<Iter>
where
    Iter: Iterator<Item = Event>,
{
    iter: Iter,
    initial_iter: Iter,
}

impl<Iter> FromIter<Iter>
where
    Iter: Iterator<Item = Event> + Clone,
{
    pub fn new(iter: Iter) -> Self {
        let initial_iter = iter.clone();
        Self { iter, initial_iter }
    }
}

impl<Iter> Iterator for FromIter<Iter>
where
    Iter: Iterator<Item = Event>,
{
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<Iter> EventIter for FromIter<Iter>
where
    Iter: Iterator<Item = Event> + Clone,
{
    fn reset(&mut self) {
        self.iter = self.initial_iter.clone();
    }
}
