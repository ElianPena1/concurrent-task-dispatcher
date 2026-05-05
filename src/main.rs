use std::collections::VecDeque;
use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
enum TaskType {
    CPU,
    IO,
}

#[derive(Debug, Clone)]
struct Task {
    id: usize,
    kind: TaskType,
    time_needed: Duration,
    created_at: Instant,
}

#[derive(Debug)]
struct Metrics {
    total_done: usize,
    cpu_done: usize,
    io_done: usize,
    total_wait: Duration,
    total_turnaround: Duration,
    max_wait: Duration,
    total_work_time: Duration,
}

fn make_tasks(workload: &str) -> Vec<Task> {
    let mut tasks = Vec::new();

    if workload == "stressed" {
        // stressed workload => mostly CPU tasks with uneven times
        for i in 1..=500 {
            let kind;
            let time_needed;

            if i % 5 == 0 {
                kind = TaskType::IO;
                time_needed = Duration::from_millis(15);
            } else {
                kind = TaskType::CPU;

                if i % 10 == 0 {
                    time_needed = Duration::from_millis(90);
                } else {
                    time_needed = Duration::from_millis(45);
                }
            }

            tasks.push(Task {
                id: i,
                kind,
                time_needed,
                created_at: Instant::now(),
            });

            // small delay so tasks doesn't all arrive at the same time
            thread::sleep(Duration::from_millis(1));
        }
    } else {
        // balanced workload => mixed CPU and IO tasks
        for i in 1..=500 {
            let kind;
            let time_needed;

            if i % 2 == 0 {
                kind = TaskType::CPU;
                time_needed = Duration::from_millis(40);
            } else {
                kind = TaskType::IO;
                time_needed = Duration::from_millis(20);
            }

            tasks.push(Task {
                id: i,
                kind,
                time_needed,
                created_at: Instant::now(),
            });

            // small delay so tasks doesn't all arrive at the exact same time
            thread::sleep(Duration::from_millis(1));
        }
    }

    tasks
}

fn main() {
    println!("Concurrent Task Dispatcher started");

    let args: Vec<String> = env::args().collect();

    let mode = if args.len() > 1 {
        args[1].clone()
    } else {
        String::from("fifo")
    };

    let workload = if args.len() > 2 {
        args[2].clone()
    } else {
        String::from("balanced")
    };

    let worker_count = 4;

    println!("Scheduling mode: {}", mode);
    println!("Workload: {}", workload);
    println!("Worker count: {}", worker_count);
    println!("Total tasks created: 500");

    let start_time = Instant::now();

    // shared queue for all workers
    let queue = Arc::new(Mutex::new(VecDeque::new()));

    // shared metrics so workers can update final results
    let metrics = Arc::new(Mutex::new(Metrics {
        total_done: 0,
        cpu_done: 0,
        io_done: 0,
        total_wait: Duration::from_millis(0),
        total_turnaround: Duration::from_millis(0),
        max_wait: Duration::from_millis(0),
        total_work_time: Duration::from_millis(0),
    }));

    let mut tasks = make_tasks(&workload);

    if mode == "optimized" {
        // optimized mode puts shorter tasks first
        tasks.sort_by_key(|task| task.time_needed);
    }

    for task in tasks {
        queue.lock().unwrap().push_back(task);
    }

    let mut handles = Vec::new();

    for _worker_id in 1..=worker_count {
        let queue_clone = Arc::clone(&queue);
        let metrics_clone = Arc::clone(&metrics);

        let handle = thread::spawn(move || {
            loop {
                // lock queue, take one task, then unlock
                let task_option = {
                    let mut queue = queue_clone.lock().unwrap();
                    queue.pop_front()
                };

                match task_option {
                    Some(task) => {
                        let started_at = Instant::now();
                        let wait_time = started_at.duration_since(task.created_at);

                        thread::sleep(task.time_needed);

                        let finished_at = Instant::now();
                        let turnaround = finished_at.duration_since(task.created_at);

                        {
                            let mut m = metrics_clone.lock().unwrap();

                            m.total_done += 1;
                            m.total_wait += wait_time;
                            m.total_turnaround += turnaround;
                            m.total_work_time += task.time_needed;

                            if wait_time > m.max_wait {
                                m.max_wait = wait_time;
                            }

                            match task.kind {
                                TaskType::CPU => m.cpu_done += 1,
                                TaskType::IO => m.io_done += 1,
                            }
                        }
                    }
                    None => {
                        // no task left, so this worker can stop
                        break;
                    }
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let makespan = start_time.elapsed();
    let m = metrics.lock().unwrap();

    println!("\n===== Metrics =====");
    println!("Total tasks completed: {}", m.total_done);
    println!("CPU tasks completed: {}", m.cpu_done);
    println!("IO tasks completed: {}", m.io_done);
    println!("Makespan: {:?}", makespan);

    if m.total_done > 0 {
        println!(
            "Average wait time: {:?}",
            m.total_wait / m.total_done as u32
        );

        println!(
            "Average turnaround time: {:?}",
            m.total_turnaround / m.total_done as u32
        );
    }

    println!("Max wait time: {:?}", m.max_wait);

    // worker utilization = actual work time divided by possible worker time
    let total_possible_work_time = makespan.as_secs_f64() * worker_count as f64;
    let actual_work_time = m.total_work_time.as_secs_f64();

    if total_possible_work_time > 0.0 {
        let worker_utilization = (actual_work_time / total_possible_work_time) * 100.0;
        println!("Worker utilization: {:.2}%", worker_utilization);
    }
}