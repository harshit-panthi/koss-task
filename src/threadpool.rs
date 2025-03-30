use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPoolHandle{
  tp_handle: Option<JoinHandle<()>>, // Will be Null only during the destruction of ThreadPoolHandle
  sender: Option<Sender<Job>>, // Will be Null only during the destruction of ThreadPoolHandle
}

impl ThreadPoolHandle {
  pub fn new(n: usize) -> ThreadPoolHandle {
    assert!(n > 0, "Threadpools must have atleast one worker thread");
    let (tx, rx) = channel();
    ThreadPoolHandle {
      tp_handle: Some(thread::spawn(move || ThreadPool::master(n, rx))),
      sender: Some(tx),
    }
  }

  pub fn queue<F: FnOnce() + Send + 'static>(&self, job: F){
    let sender = match &self.sender {
      Some(s) => s,
      None => panic!("queue cannot be called after the destructor for ThreadPoolHandle has been called"),
    };
    match sender.send(Box::new(job)){
      Ok(_) => {},
      Err(e) => panic!("threadpool receiver cannot be destroyed before the sender in threadpool_handle\nError: {}", e),
    }
  }
}

struct ThreadPool {
  worker_comm: Vec<Worker>,
  global_queued_jobs: Arc<Mutex<VecDeque<Job>>>,
}

impl ThreadPool {
  fn new(n: usize) -> ThreadPool {
    assert!(n > 0, "Threadpools must have atleast one worker thread");

    let glob = Arc::new(Mutex::new(VecDeque::new()));

    let mut ws = Vec::with_capacity(n);
    let mut jqs = Vec::with_capacity(n);
    
    for _ in 0..n {
      jqs.push(Arc::new(Mutex::new(VecDeque::new())));
    }

    for i in 0..n{
      let j = match jqs.get(i) {
        Some(ljs) => ljs.clone(),
        None => unreachable!(),
      };

      let n = match jqs.get(if i == n - 1 {0} else {i + 1}) {
        Some(ljs) => ljs.clone(),
        None => unreachable!(),
      };

      ws.push(Worker::new(glob.clone(), j, n));
    }

    ThreadPool {
      worker_comm: ws,
      global_queued_jobs: glob,
    }
  }

  fn queue(&mut self, job: Job){
    self.global_queued_jobs.lock().unwrap().push_back(job);
  }

  fn master(n: usize, rx: Receiver<Job>){
    let mut tp = ThreadPool::new(n);
    loop {
      if let Ok(job) = rx.recv() {
        tp.queue(job);
      }
      else {
        break;
      }
    };

    for worker in tp.worker_comm {
      let _ = worker.thread.join();
    }
  }
}

struct Worker {
  thread: JoinHandle<()>,
}

impl Worker {
  fn new(global_job_queue: Arc<Mutex<VecDeque<Job>>>,
    job_queue: Arc<Mutex<VecDeque<Job>>>,
    neighbours_job_queue: Arc<Mutex<VecDeque<Job>>>)
    -> Worker
  {
    Worker {
      thread: thread::spawn(move || Worker::handler(global_job_queue, job_queue, neighbours_job_queue)),
    }
  }

  fn handler(global_job_queue: Arc<Mutex<VecDeque<Job>>>,
    job_queue: Arc<Mutex<VecDeque<Job>>>,
    neighbours_job_queue: Arc<Mutex<VecDeque<Job>>>
  )
  {
    'mainloop: loop {
      let mut jq = job_queue.lock().unwrap();
      let len = jq.len();
      if len != 0 {
        let job = match jq.pop_front() {
          Some(j) => j,
          None => unreachable!(),
        };
        drop(jq);
        (job)();
      }
      else {
        // using lock instead of try_lock here can cause a deadlock between workers
        match neighbours_job_queue.try_lock() {
          Ok(mut njq) if (njq.len() != 0) => {
            let len = njq.len();
            jq.append(&mut njq.split_off(len / 2));
            continue 'mainloop;
          },
          njq => {
            match njq {
              Ok(njq) => drop(njq),
              Err(_) => {},
            };
            let mut gjq = global_job_queue.lock().unwrap();
            let len = gjq.len();
            match len {
              0 => {
                drop(gjq);
                drop(jq);
                thread::sleep(Duration::from_millis(10))
              },
              _ => jq.append(&mut gjq.split_off(len / 2)),
            };
            continue 'mainloop;
          },
        };
      }
    };    
  }
}

impl Drop for ThreadPoolHandle {
  fn drop(&mut self) {
      drop(self.sender.take());
      let _ = self.tp_handle.take().expect("tp_handle cant be None before the destruction of ThreadPoolHandle").join();
  }
}