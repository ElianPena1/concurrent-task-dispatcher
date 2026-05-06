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
    cpu_wait: Duration,
    io_wait: Duration,
    total_turnaround: Duration,
    max_wait: Duration,
    total_work_time: Duration,
}

fn make_tasks() -> Vec<Task> {
    let mut tasks = Vec::new();

    // 1000 tasks total
    // 70% IO and 30% CPU
    for i in 1..=1000 {
        let kind;
        let time_needed;

        // 3 CPU tasks out of every 10 tasks
        if i % 10 == 0 || i % 10 == 3 || i % 10 == 6 {
            kind = TaskType::CPU;

            if i % 30 == 0 {
                time_needed = Duration::from_millis(70);
            } else {
                time_needed = Duration::from_millis(45);
            }
        } else {
            kind = TaskType::IO;

            if i % 25 == 0 {
                time_needed = Duration::from_millis(25);
            } else {
                time_needed = Duration::from_millis(15);
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

    tasks
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mode = if args.len() > 1 {
        args[1].clone()
    } else {
        String::from("fifo")
    };

    let worker_count = 8;

    if mode == "optimized" {
        println!("== Optimized simulation ==");
    } else {
        println!("== FIFO simulation ==");
    }

    println!("1000 tasks, 70% IO / 30% CPU, 8 workers, cap 100%");

    let start_time = Instant::now();

    // shared queue for all workers
    let queue = Arc::new(Mutex::new(VecDeque::new()));

    // shared metrics so workers can update final results
    let metrics = Arc::new(Mutex::new(Metrics {
        total_done: 0,
        cpu_done: 0,
        io_done: 0,
        total_wait: Duration::from_millis(0),
        cpu_wait: Duration::from_millis(0),
        io_wait: Duration::from_millis(0),
        total_turnaround: Duration::from_millis(0),
        max_wait: Duration::from_millis(0),
        total_work_time: Duration::from_millis(0),
    }));

    let mut tasks = make_tasks();

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
                        let _task_id = task.id;

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
                                TaskType::CPU => {
                                    m.cpu_done += 1;
                                    m.cpu_wait += wait_time;
                                }
                                TaskType::IO => {
                                    m.io_done += 1;
                                    m.io_wait += wait_time;
                                }
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

    println!("\n— results —");
    println!("total runtime          : {} ms", makespan.as_millis());
    println!("makespan               : {} ms", makespan.as_millis());
    println!(
        "tasks completed        : {} (CPU={}, IO={})",
        m.total_done, m.cpu_done, m.io_done
    );

    if m.total_done > 0 {
        println!(
            "avg wait time          : {} ms",
            (m.total_wait / m.total_done as u32).as_millis()
        );

        println!(
            "avg turnaround time    : {} ms",
            (m.total_turnaround / m.total_done as u32).as_millis()
        );
    }

    if m.io_done > 0 {
        println!(
            "avg wait (IO only)     : {} ms",
            (m.io_wait / m.io_done as u32).as_millis()
        );
    }

    if m.cpu_done > 0 {
        println!(
            "avg wait (CPU only)    : {} ms",
            (m.cpu_wait / m.cpu_done as u32).as_millis()
        );
    }

    println!("max wait time          : {} ms", m.max_wait.as_millis());

    // worker utilization => actual work time divided by possible worker time
    let total_possible_work_time = makespan.as_secs_f64() * worker_count as f64;
    let actual_work_time = m.total_work_time.as_secs_f64();

    if total_possible_work_time > 0.0 {
        let worker_utilization = (actual_work_time / total_possible_work_time) * 100.0;
        let capped_cpu_usage = worker_utilization.min(100.0);
        let avg_workers_active = (capped_cpu_usage / 100.0) * worker_count as f64;

        println!("avg CPU usage          : {:.2} %", capped_cpu_usage);
        println!(
            "avg workers active     : {:.2} / {}",
            avg_workers_active, worker_count
        );
    }
}