# concurrent-task-dispatcher

## Project Summary

This project is a Rust program that shows how tasks can run at the same time. The program makes tasks, puts them into a queue, and has 4 workers run them.

The project compares two ways of picking tasks. The first way is FIFO, which runs tasks in the order they were added. The second way is optimized, which runs shorter tasks first.

The main goal is to show how a task goes from being created, to waiting in a queue, to being run by a worker, and then being counted in the final results.

## How to Build

```bash
cargo build
```

## How to Run

The program uses two command line options:

```bash
cargo run -- <mode> <workload>
```

Modes:

```text
fifo
optimized
```

Workloads:

```text
balanced
stressed
```

## Command Examples

Run FIFO with the balanced workload:

```bash
cargo run -- fifo balanced
```

Run optimized with the stressed workload:

```bash
cargo run -- optimized stressed
```

Other runs I used for testing:

```bash
cargo run -- optimized balanced
cargo run -- fifo stressed
```

## Saving Experiment Output

To save the experiment results into text files:

```bash
cargo run -- fifo balanced > experiment_fifo_balanced.txt
cargo run -- optimized stressed > experiment_optimized_stressed.txt
```

## Design Summary

The program has a few main parts:

- task maker
- shared queue
- worker threads
- final metrics

The task maker creates tasks with an ID, a type, a run time, and the time it was created.

After that, each task gets placed into a shared queue. I used `VecDeque` for the queue because it lets me add tasks to the back and remove them from the front.

There are 4 worker threads. Each worker tries to take a task from the queue. If it gets one, it runs the task by sleeping for that task’s time. After that, it updates the metrics. If the queue is empty, the worker stops.

## Shared Data

The queue is shared by all workers:

```rust
Arc<Mutex<VecDeque<Task>>>
```

I used `Arc` so more than one worker can access the same queue.

I used `Mutex` so only one worker can use the queue at a time. Without that, two workers could try to take the same task or change the queue at the same time.

The metrics are also shared because every worker needs to update the final results.

## Scheduling

The project has two scheduling modes.

### FIFO

FIFO runs tasks in the order they were added.

This is simple and easy to understand. The downside is that a short task can get stuck waiting behind a long task.

### Optimized

Optimized sorts the tasks so the shorter ones go first.

This can make the average wait time better because smaller tasks finish sooner. The downside is that longer tasks might wait more.

## Workloads

### Balanced

The balanced workload has a mix of CPU and IO tasks.

### Stressed

The stressed workload has more CPU tasks and uneven task times. This makes the scheduler work harder.

## Metrics

The program prints:

- total tasks completed
- CPU tasks completed
- IO tasks completed
- makespan
- average wait time
- average turnaround time
- max wait time

## Experiments

### Experiment 1: FIFO Balanced

Command:

```bash
cargo run -- fifo balanced
```

This run uses FIFO with a balanced mix of tasks. It shows how the basic scheduler works.

### Experiment 2: Optimized Stressed

Command:

```bash
cargo run -- optimized stressed
```

This run uses the optimized mode with a harder workload. The stressed workload has more CPU tasks and some longer tasks.

The main thing I am comparing is how long the program takes and how the wait times change.

## Clean Shutdown

The workers stop when the queue is empty. In the code, this happens when `pop_front()` returns `None`.

After that, the worker breaks out of its loop. The main thread then waits for all workers to finish using `join()`.

## Bug or Issue I Hit

One issue I had was with workers stopping correctly. At first, I had to make sure the workers would not keep running forever after all tasks were done.

I fixed this by checking if the queue was empty. If there was no task left, the worker prints that it is shutting down and exits the loop.

Another small issue was Rust warning me that some task fields were not being used. That happened early on before I added timing and metrics. Once I used `time_needed` and `created_at`, that warning made sense and was fixed.

## Tool Use Disclosure

I used ChatGPT to help break the project into smaller steps and understand what the project was asking for.

It helped me with ideas for the queue, worker threads, and how to explain `Arc` and `Mutex`.

One suggestion I accepted was using a shared queue with 4 worker threads because it matched the project requirements.

One suggestion I changed was making the scheduler too complicated. I kept it simpler so I could explain it better during the demo.

## Files Included

```text
Cargo.toml
Cargo.lock
.gitignore
README.md
src/main.rs
experiment_fifo_balanced.txt
experiment_optimized_stressed.txt
report.pdf
```