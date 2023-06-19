extern crate alloc;

use alloc::boxed::Box;

use crate::common::Duration;

pub trait Task<State, Action> {
    fn run(&mut self, state: &mut State) -> (Option<Action>, NextRun);
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum NextRun {
    InOrder,
    After(Duration),
}

pub struct FnTask<State, Action> {
    f: Box<dyn FnMut(&mut State) -> (Option<Action>, NextRun)>,
}

impl<State, Action> FnTask<State, Action> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut State) -> (Option<Action>, NextRun) + 'static,
    {
        Self { f: Box::new(f) }
    }
}

impl<State, Action> Task<State, Action> for FnTask<State, Action> {
    fn run(&mut self, state: &mut State) -> (Option<Action>, NextRun) {
        (self.f)(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fn_task_test() {
        let mut state = 0;
        let mut task = FnTask::<i32, i32>::new(|state| {
            *state += 1;
            (Some(42), NextRun::InOrder)
        });

        let action = task.run(&mut state);

        assert_eq!(state, 1);
        assert_eq!(action, (Some(42), NextRun::InOrder));
    }
}
