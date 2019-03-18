use std::sync::mpsc::{self, Receiver, Sender};
#[derive(Debug)]
pub struct Task {
    pub chunk: usize,
    pub amount: usize,
}

#[derive(Debug)]
enum ManagerState {
    Halted,
    CycleProgress {
        cycle: usize,
        amount: usize,
        chunk: usize,
    },
    CycleWait {
        next_cycle: usize,
        waiters: Vec<(usize, Sender<Option<Task>>)>,
    },
}

pub struct Manager {
    chunks: usize,
    state: ManagerState,
    thread_idle: Vec<bool>,
    on_cycle_complete: Box<dyn FnMut(usize, usize) -> Option<usize> + Send>,
    total_amount: usize,
}

impl Manager {
    pub fn new(
        chunks: usize,
        nthread: usize,
        on_cycle_complete: Box<dyn FnMut(usize, usize) -> Option<usize> + Send>,
    ) -> Self {
        Manager {
            chunks,
            //total_amount,
            //max_cycle_amount,
            state: ManagerState::CycleWait {
                next_cycle: 0,
                waiters: vec![],
            },
            thread_idle: vec![true; nthread],
            on_cycle_complete,
            total_amount: 0,
        }
    }

    pub fn next(&mut self, thid: usize) -> Receiver<Option<Task>> {
        use ManagerState::*;

        let (tx, rx) = mpsc::channel();
        self.thread_idle[thid] = true;
        let all_idle = self.all_idle();
        match self.state {
            Halted => Self::send_none(&tx),
            CycleWait {
                next_cycle,
                ref mut waiters,
            } => {
                waiters.push((thid, tx));
                if all_idle {
                    if let Some(amount) = (self.on_cycle_complete)(next_cycle, self.total_amount) {
                        self.total_amount += amount;
                        for (i, (thid, tx)) in waiters.iter().enumerate() {
                            Self::send_task(
                                &mut self.thread_idle,
                                *thid,
                                tx,
                                Task { chunk: i, amount },
                            );
                        }
                        self.state = CycleProgress {
                            cycle: next_cycle,
                            amount,
                            chunk: waiters.len(),
                        };
                    } else {
                        for (_, (_, tx)) in waiters.into_iter().enumerate() {
                            Self::send_none(tx);
                        }
                        self.state = Halted;
                    }
                }
            }
            CycleProgress {
                cycle,
                amount,
                chunk,
            } => {
                if chunk + 1 == self.chunks {
                    self.state = CycleWait {
                        next_cycle: cycle + 1,
                        waiters: vec![],
                    };
                } else {
                    self.state = CycleProgress {
                        cycle,
                        amount,
                        chunk: chunk + 1,
                    };
                }
                Self::send_task(&mut self.thread_idle, thid, &tx, Task { chunk, amount });
            }
        }
        rx
    }

    fn all_idle(&self) -> bool {
        self.thread_idle.iter().all(|b| *b)
    }

    fn send_none(tx: &Sender<Option<Task>>) {
        let _ = tx.send(None);
    }

    fn send_task(thread_idle: &mut Vec<bool>, thid: usize, tx: &Sender<Option<Task>>, task: Task) {
        thread_idle[thid] = false;
        let _ = tx.send(Some(task));
    }
}
