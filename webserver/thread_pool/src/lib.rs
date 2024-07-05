use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>; // the type of closure which ThreadPool::execute receives

struct Worker {
    // here we put unit type () because our use case doesn't return
    // if we want to expand this thread pool struct, we can use type T
    id: u32,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: u32, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("worker {id} got a job, executing.");

            job();
        });

        Worker { id, thread }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>, // sends jobs to workers
}

impl ThreadPool {
    /// Creates a new ThreadPool.
    ///
    /// argument: size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if size is zero.
    pub fn new(size: u32) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size as usize);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}
