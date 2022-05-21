use std::{sync::{Arc, atomic::{AtomicU32, Ordering}}, thread};

use crossbeam_channel::Sender;
use parking_lot::{Condvar, Mutex};

pub struct ThreadPool<T: Send + 'static> {
	threadcount: u32,
	send: Sender<ThreadCommand<T>>,
	dead: bool
}

impl<T: Send + 'static> ThreadPool<T> {
	pub fn new<W, C, I>(threadcount: u32, worker: W, init: I) -> ThreadPool<T>  where 
			W: Fn(T, &mut C) + Clone + Send + 'static,
			I: Fn() -> C + Clone + Send + 'static {
		let (send, recv) = crossbeam_channel::unbounded();

		for _ in 0..threadcount {
			let thread_recv = recv.clone();
			let thread_fn = worker.clone();
			let thread_init = init.clone();
			thread::spawn(move || {
				let mut context = thread_init();
				while let Ok(ThreadCommand::Command(t, countdown)) = thread_recv.recv() {
					thread_fn(t, &mut context);
					if let Some(c) = countdown {
						c.countdown();
					}
				}
			});
		}

		ThreadPool {
			send, threadcount,
			dead: false
		}
	}

	pub fn send(&self, task: T) -> Result<Wait, T> {
		if !self.dead {
			let latch = Arc::new(CountdownLatch::new(1));
			let _ = self.send.send(ThreadCommand::Command(task, Some(latch.clone())));
			Ok(Wait::Latch(latch))
		} else {
			Err(task)
		}
	}

	pub fn send_all(&self, tasks: Vec<T>) -> Result<Wait, Vec<T>> {
		if !self.dead {
			let latch = Arc::new(CountdownLatch::new(tasks.len() as u32));
			for task in tasks {
				let _ = self.send.send(ThreadCommand::Command(task, Some(latch.clone())));
			}
			Ok(Wait::Latch(latch))
		} else {
			Err(tasks)
		}
	}

	pub fn die(&mut self) {
		self.dead = true;
		for _ in 0..self.threadcount {
			let _ = self.send.send(ThreadCommand::Die);
		}
	}
}

impl<T: Send + 'static> Drop for ThreadPool<T> {
	fn drop(&mut self) {
		self.die()
	}
}

pub enum BranchedExecutor<T: Send + 'static> {
	ThisThread(Box<dyn Fn(T)>),
	Pooled(ThreadPool<T>)
}

impl<T: Send + 'static> BranchedExecutor<T> {
	pub fn exec(&self, commands: Vec<T>) -> Result<Wait, Vec<T>> {
		match self {
			Self::ThisThread(f) => {
				for cmd in commands {
					f(cmd);
				}
				Ok(Wait::Noop)
			},
			Self::Pooled(pool) => {
				pool.send_all(commands)
			}
		}
	}
}

#[derive(Clone)]
pub enum Wait {
	Latch(Arc<CountdownLatch>),
	Noop
}

impl Wait {
	pub fn wait(&self) {
		match self {
			Self::Latch(latch) => latch.wait(),
			Self::Noop => ()
		}
	}
}

pub struct CountdownLatch {
	status: Mutex<bool>,
	var: Condvar,
	count: AtomicU32 // don't use a Mutex<u32> so we can decrement without locking
}

impl CountdownLatch {
	pub fn new(count: u32) -> Self {
		Self {
			status: Mutex::new(false),
			var: Condvar::new(),
			count: AtomicU32::new(count)
		}
	}

	pub fn wait(&self) {
		if self.count.load(Ordering::Relaxed) > 0 {
			let mut status_guard = self.status.lock();
			if !*status_guard {
				self.var.wait(&mut status_guard);
			}
		}
	}

	pub fn countdown(&self) {
		let cur_count = self.count.fetch_sub(1, Ordering::Relaxed) - 1;
		if cur_count <= 0 {
			let mut status_guard = self.status.lock();
			*status_guard = true;
			self.var.notify_all();
		}
	}
}

enum ThreadCommand<T: Send + 'static> {
	Command(T, Option<Arc<CountdownLatch>>),
	Die
}