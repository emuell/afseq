use crate::pattern::Pattern;

// -------------------------------------------------------------------------------------------------

/// Defines an empty pattern which never triggers a pulse.
#[derive(Clone, Debug)]
pub struct EmptyPattern {}

impl Pattern for EmptyPattern {
    fn is_empty(&self) -> bool {
        true
    }
    fn len(&self) -> usize {
        0
    }
    fn run(&mut self) -> f32 {
        panic!("Empty patterns should not be run");
    }
    fn reset(&mut self) {
        // nothing to do
    }
}
