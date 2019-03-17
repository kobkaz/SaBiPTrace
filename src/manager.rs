use std::sync::mpsc::{self, Receiver, Sender};
#[derive(Debug)]
pub struct Task {
    pub chunk: usize,
    pub amount: usize,
}

#[derive(Debug)]
enum ManagerState {
    Halted,
    CycleWait(usize, usize, Vec<(usize, Sender<Option<Task>>)>),
    CycleProgress { cycle: usize, chunk: usize },
}

pub struct Manager {
    chunks: usize,
    total_amount: usize,
    max_cycle_amount: usize,
    state: ManagerState,
    thread_idle: Vec<bool>,
    on_cycle_complete: Box<dyn FnMut(usize, usize) + Send>,
}

impl Manager {
    pub fn new(
        chunks: usize,
        total_amount: usize,
        max_cycle_amount: usize,
        nthread: usize,
        on_cycle_complete: Box<dyn FnMut(usize, usize) + Send>,
    ) -> Self {
        Manager {
            chunks,
            total_amount,
            max_cycle_amount,
            state: ManagerState::CycleProgress { cycle: 0, chunk: 0 },
            thread_idle: vec![true; nthread],
            on_cycle_complete,
        }
    }

    pub fn next(&mut self, thid: usize) -> Receiver<Option<Task>> {
        use ManagerState::*;

        let (tx, rx) = mpsc::channel();
        self.thread_idle[thid] = true;
        let all_idle = self.all_idle();
        match self.state {
            Halted => {
                let _ = tx.send(None);
                if all_idle {
                    (self.on_cycle_complete)(self.total_amount, self.total_amount)
                }
            }
            CycleWait(prev_cycle, amount, ref mut txs) => {
                let cycle = prev_cycle + 1;
                txs.push((thid, tx));
                if all_idle {
                    (self.on_cycle_complete)(
                        (self.max_cycle_amount * cycle).min(self.total_amount),
                        self.total_amount,
                    );
                    for (i, (thid, tx)) in txs.into_iter().enumerate() {
                        let _ = tx.send(Some(Task { chunk: i, amount }));
                        self.thread_idle[*thid] = false;
                    }
                    self.state = CycleProgress {
                        cycle,
                        chunk: txs.len(),
                    };
                }
            }
            CycleProgress { cycle, chunk } => {
                let amount = self.cycle_amount(cycle);
                if chunk + 1 == self.chunks {
                    if (cycle + 1) * self.max_cycle_amount >= self.total_amount {
                        self.state = Halted;
                    } else {
                        self.state = CycleWait(cycle, self.cycle_amount(cycle + 1), vec![]);
                    }
                } else {
                    self.state = CycleProgress {
                        cycle,
                        chunk: chunk + 1,
                    };
                }
                let _ = tx.send(Some(Task { chunk, amount }));
                self.thread_idle[thid] = false;
            }
        }
        rx
    }

    fn all_idle(&self) -> bool {
        self.thread_idle.iter().all(|b| *b)
    }

    fn cycle_amount(&self, cycle: usize) -> usize {
        let completed = cycle * self.max_cycle_amount;
        if completed >= self.total_amount {
            0
        } else {
            (self.total_amount - completed).min(self.max_cycle_amount)
        }
    }
}
