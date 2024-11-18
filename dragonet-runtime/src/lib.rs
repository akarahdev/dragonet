mod tests;

use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::task::{Context, Poll};
use futures::task;
use futures::task::{ArcWake, SpawnExt};

pub struct Runtime {
    scheduled: Receiver<Arc<Task>>,
    sender: Sender<Arc<Task>>,
}

impl Runtime {
    pub fn new() -> Self {
        let mpsc = channel();
        Runtime {
            scheduled: mpsc.1,
            sender: mpsc.0
        }
    }

    pub fn spawn<F>(&mut self, future: F)
        where F: Future<Output=()> + Send  + 'static {
        self.sender.send(Arc::new(Task {
            data: Mutex::new(TaskFutureData {
                future: Box::pin(future),
                poll: Poll::Pending,
            }),
            executor: self.sender.clone(),
        })).expect("failed to send future somehow");
    }

    pub fn run(&mut self) {
        while let Ok(task) = self.scheduled.recv() {
            println!("Task: {:?}", task);
            std::thread::spawn(move || {
                task.poll();
            });
        }
    }
}

#[derive(Debug)]
struct Task {
    data: Mutex<TaskFutureData>,
    executor: Sender<Arc<Task>>,
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.executor.send(arc_self.clone())
            .expect("failed to send task");
    }
}

impl Task {
    fn poll(self: Arc<Self>) {
        let waker = task::waker(self.clone());
        let mut cx = Context::from_waker(&waker);

        let mut future_data = self.data.try_lock().unwrap();
        let polled = future_data.poll_tfd(&mut cx);
        println!("polled: {:?}", polled);
    }
}

struct TaskFutureData {
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
    poll: Poll<()>
}

impl Debug for TaskFutureData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskFutureData")
            .field("poll", &self.poll)
            .field("future", &"<future>")
            .finish()
    }
}

impl TaskFutureData {
    fn poll_tfd(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if self.poll.is_pending() {
            self.poll = self.future.as_mut().poll(cx);
        };
        self.poll
    }
}