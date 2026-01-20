# Exercise 05: Multi-threaded Task Queue

**Difficulty**: 🔴 Advanced | **Estimated Time**: 4-6h | **Subsystem**: Core

## Overview

Implement a work-stealing task queue for parallel execution of independent jobs. Essential for leveraging multi-core processors in game engines.

## Learning Objectives

- Understand thread pools and work queues
- Learn work-stealing algorithms
- Handle task dependencies
- Implement lock-free data structures (optional)

## Requirements

### Functional Requirements

1. **Task Queue**
   - Submit tasks (closures/function pointers)
   - Execute tasks on worker threads
   - Wait for task completion
   - Support task priorities (optional)

2. **Thread Pool**
   - Fixed number of worker threads
   - Automatic work distribution
   - Graceful shutdown

3. **Work Stealing**
   - Each thread has local queue
   - Idle threads steal from others
   - Load balancing

### Non-Functional Requirements

- **Performance**: Minimal overhead (< 10µs per task)
- **Scalability**: Linear speedup to thread count
- **Safety**: No data races, proper synchronization

## API Design

```rust
pub struct TaskQueue {
    workers: Vec<Worker>,
    sender: Sender<Task>,
}

pub type Task = Box<dyn FnOnce() + Send + 'static>;

impl TaskQueue {
    pub fn new(num_threads: usize) -> Self;
    pub fn submit<F>(&self, task: F) where F: FnOnce() + Send + 'static;
    pub fn submit_batch(&self, tasks: Vec<Task>);
    pub fn wait(&self); // Wait for all tasks to complete
    pub fn active_count(&self) -> usize;
}
```

## Validation Criteria

### Correctness
- [ ] All submitted tasks execute
- [ ] Tasks execute on worker threads (not main thread)
- [ ] wait() blocks until all tasks complete
- [ ] No data races or deadlocks

### Performance
- [ ] Task submission overhead < 10µs
- [ ] Near-linear speedup with thread count
- [ ] Work stealing reduces idle time

## Test Cases

```rust
#[test]
fn test_basic_execution() {
    let queue = TaskQueue::new(4);
    let counter = Arc::new(AtomicUsize::new(0));
    
    for _ in 0..100 {
        let counter = counter.clone();
        queue.submit(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        });
    }
    
    queue.wait();
    assert_eq!(counter.load(Ordering::SeqCst), 100);
}

#[test]
fn test_parallel_speedup() {
    let start = Instant::now();
    
    // Sequential execution
    for _ in 0..1000 {
        expensive_computation();
    }
    let sequential_time = start.elapsed();
    
    // Parallel execution
    let queue = TaskQueue::new(4);
    let start = Instant::now();
    
    for _ in 0..1000 {
        queue.submit(|| expensive_computation());
    }
    queue.wait();
    let parallel_time = start.elapsed();
    
    // Should be at least 2x faster with 4 threads
    assert!(parallel_time < sequential_time / 2);
}

fn expensive_computation() {
    std::thread::sleep(Duration::from_millis(1));
}
```

## Performance Targets

| Metric | Target |
|--------|--------|
| Task submission | < 10µs |
| Task execution overhead | < 1µs |
| 1000 tasks on 4 threads | 3-4x speedup |

## Reference Implementation

### Rust (Primary)

<details>
<summary>Click to reveal Rust implementation</summary>

```rust
use std::sync::{Arc, Condvar, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

pub type Task = Box<dyn FnOnce() + Send + 'static>;

pub struct TaskQueue {
    workers: Vec<Worker>,
    sender: Sender<Message>,
    active_tasks: Arc<(Mutex<usize>, Condvar)>,
}

enum Message {
    NewTask(Task),
    Terminate,
}

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl TaskQueue {
    pub fn new(num_threads: usize) -> Self {
        let (sender, receiver) = channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let active_tasks = Arc::new((Mutex::new(0), Condvar::new()));
        
        let mut workers = Vec::with_capacity(num_threads);
        
        for id in 0..num_threads {
            workers.push(Worker::new(
                id,
                Arc::clone(&receiver),
                Arc::clone(&active_tasks),
            ));
        }
        
        Self {
            workers,
            sender,
            active_tasks,
        }
    }
    
    pub fn submit<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // Increment active task count
        let (lock, _) = &*self.active_tasks;
        let mut count = lock.lock().unwrap();
        *count += 1;
        drop(count);
        
        self.sender.send(Message::NewTask(Box::new(task))).unwrap();
    }
    
    pub fn wait(&self) {
        let (lock, cvar) = &*self.active_tasks;
        let mut count = lock.lock().unwrap();
        
        while *count > 0 {
            count = cvar.wait(count).unwrap();
        }
    }
    
    pub fn active_count(&self) -> usize {
        let (lock, _) = &*self.active_tasks;
        *lock.lock().unwrap()
    }
}

impl Drop for TaskQueue {
    fn drop(&mut self) {
        // Send terminate message to all workers
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        
        // Wait for all workers to finish
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<Receiver<Message>>>,
        active_tasks: Arc<(Mutex<usize>, Condvar)>,
    ) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let message = {
                    let receiver = receiver.lock().unwrap();
                    receiver.recv().unwrap()
                };
                
                match message {
                    Message::NewTask(task) => {
                        task();
                        
                        // Decrement active task count and notify
                        let (lock, cvar) = &*active_tasks;
                        let mut count = lock.lock().unwrap();
                        *count -= 1;
                        cvar.notify_all();
                    }
                    Message::Terminate => {
                        break;
                    }
                }
            }
        });
        
        Self {
            thread: Some(thread),
        }
    }
}

// Example usage
fn example() {
    let queue = TaskQueue::new(4);
    
    // Submit parallel tasks
    for i in 0..100 {
        queue.submit(move || {
            println!("Task {} running on {:?}", i, thread::current().id());
            // Do work...
        });
    }
    
    // Wait for completion
    queue.wait();
    println!("All tasks completed!");
}
```

</details>

## Related Resources

- [Rayon - Data Parallelism in Rust](https://github.com/rayon-rs/rayon)
- [Work Stealing (Wikipedia)](https://en.wikipedia.org/wiki/Work_stealing)

## Next Steps

- Add task dependencies (DAG execution)
- Implement priority queues
- Study ECS parallel system scheduling
