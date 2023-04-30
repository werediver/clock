extern crate alloc;

use core::cmp::Ordering;

use alloc::{boxed::Box, vec::Vec};

use crate::{
    common::Instant,
    task::{NextRun, Task},
};

pub struct Scheduler<State, Action> {
    tasks: Vec<Scheduled<Box<dyn Task<State, Action>>>>,
    next_in_order: usize,
}

impl<State, Action> Default for Scheduler<State, Action> {
    fn default() -> Self {
        Self {
            tasks: Default::default(),
            next_in_order: 0,
        }
    }
}

impl<State, Action> Scheduler<State, Action> {
    pub fn new(tasks: impl IntoIterator<Item = Box<dyn Task<State, Action>>>) -> Self {
        Self {
            tasks: tasks
                .into_iter()
                .map(|task| Scheduled {
                    at: SchedulePoint::InOrder,
                    task,
                })
                .collect::<Vec<_>>(),
            next_in_order: 0,
        }
    }

    pub fn run(&mut self, now: Instant, state: &mut State) -> Option<Action> {
        if let Some(timed) = self.pick_task(now) {
            let (action, next_run) = timed.task.run(state);
            timed.at = match next_run {
                NextRun::InOrder => SchedulePoint::InOrder,
                NextRun::After(delay) => SchedulePoint::At(now + delay),
            };

            action
        } else {
            None
        }
    }

    fn pick_task(&mut self, now: Instant) -> Option<&mut Scheduled<Box<dyn Task<State, Action>>>> {
        let task_i = self.pick_timed(now).or_else(|| self.pick_in_order());

        task_i.map(|i| &mut self.tasks[i])
    }

    fn pick_timed(&mut self, now: Instant) -> Option<usize> {
        self.tasks
            .iter_mut()
            .enumerate()
            .fold(None, |result, candidate| match result {
                None => Some(candidate).filter(|(_, scheduled)| scheduled.is_timed()),
                Some(result) => Some(candidate)
                    .filter(|(_, scheduled)| scheduled.is_timed_before(result.1.at))
                    .or(Some(result)),
            })
            .and_then(|(i, result)| {
                if result.is_timed_no_later(SchedulePoint::At(now)) {
                    Some(i)
                } else {
                    None
                }
            })
    }

    fn pick_in_order(&mut self) -> Option<usize> {
        let task_count = self.tasks.len();
        let (tail, head) = self.tasks.split_at_mut(self.next_in_order);

        let next_in_order = head.iter_mut().chain(tail.iter_mut()).enumerate().fold(
            None,
            |result, (i, candidate)| {
                if result.is_none() && !candidate.is_timed() {
                    Some((self.next_in_order + i) % task_count)
                } else {
                    result
                }
            },
        );

        self.next_in_order = (next_in_order.unwrap_or(self.next_in_order) + 1) % task_count;

        next_in_order
    }
}

struct Scheduled<Task> {
    at: SchedulePoint,
    task: Task,
}

#[derive(Copy, Clone, Debug)]
enum SchedulePoint {
    InOrder,
    At(Instant),
}

impl SchedulePoint {
    fn cmp_instant(&self, other: Self) -> Option<Ordering> {
        use SchedulePoint::*;

        match (self, other) {
            (InOrder, _) => None,
            (_, InOrder) => None,
            (At(a), At(b)) => Some(a.cmp(&b)),
        }
    }
}

impl<Task> Scheduled<Task> {
    fn is_timed(&self) -> bool {
        matches!(self.at, SchedulePoint::At(_))
    }

    fn is_timed_before(&self, pt: SchedulePoint) -> bool {
        matches!(self.at.cmp_instant(pt), Some(Ordering::Less))
    }

    fn is_timed_no_later(&self, pt: SchedulePoint) -> bool {
        matches!(
            self.at.cmp_instant(pt),
            Some(Ordering::Less) | Some(Ordering::Equal)
        )
    }
}
