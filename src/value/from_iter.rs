use crate::{EmitterEvent, EmitterValue};

// -------------------------------------------------------------------------------------------------

/// Creates a EmitterValue from a plain iterator of EmitterEvents.
pub struct EmitterValueFromIter<Iter>
where
    Iter: Iterator<Item = EmitterEvent>,
{
    iter: Iter,
    initial_iter: Iter,
}

impl<Iter> EmitterValueFromIter<Iter>
where
    Iter: Iterator<Item = EmitterEvent> + Clone,
{
    pub fn new(iter: Iter) -> Self {
        let initial_iter = iter.clone();
        Self { iter, initial_iter }
    }
}

impl<Iter> Iterator for EmitterValueFromIter<Iter>
where
    Iter: Iterator<Item = EmitterEvent>,
{
    type Item = EmitterEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<Iter> EmitterValue for EmitterValueFromIter<Iter>
where
    Iter: Iterator<Item = EmitterEvent> + Clone,
{
    fn reset(&mut self) {
        self.iter = self.initial_iter.clone();
    }
}
