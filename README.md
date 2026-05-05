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

The task maker creates each task with a number, a type, how long it should run, and when it was made

Then the task is put into a shared queue, and I used `VecDeque` because it makes it easy to add tasks to the back and take tasks from the front

There are 4 workers, and each worker takes a task from the queue, runs it, and updates the final results

If there are no tasks left, the worker stops

## Shared Data

The queue is shared by all workers:

```rust
Arc<Mutex<VecDeque<Task>>>
```

I used `Arc` so the 4 workers can all share the same queue

I used `Mutex` so only one worker can mess with the queue at one time, that way two workers do not grab the same task by accident

The metrics are shared too because all the workers need to add their results when they finish a task

## Scheduling

The project has two scheduling modes.

### FIFO

FIFO runs the tasks in the same order they were put into the queue

I used this because it is simple and easy to follow, but one problem is that a short task might have to wait behind a longer task

### Optimized

Optimized puts the shorter tasks first

I used this to try to lower the wait time, since smaller tasks can finish faster, but the downside is that longer tasks might have to wait more

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
- worker utilization

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

The full printed results are saved in `experiment_fifo_balanced.txt` and `experiment_optimized_stressed.txt`.

## Clean Shutdown

The workers stop when there are no tasks left in the queue

In the code, this happens when `pop_front()` gives back `None`, then the worker leaves the loop

After all the workers stop, the main thread uses `join()` to wait for them before the program ends


## Tool Use Disclosure

I used ChatGPT for small help with checking requirements and formatting the README.

I also used it to explain how some parts of the code worked, like the task maker, the queue, the workers, and the metrics.

One suggestion I accepted was using one shared queue with 4 worker threads. I used this because the project needed a queue and a worker pool, and this setup was simple enough for me to follow. The queue holds the tasks, and the workers take tasks from the queue until there are no tasks left.

One suggestion I changed was making the scheduler too complicated. At first, there were ideas like using more advanced scheduling rules, but I did not want to add something I could not explain well. I kept the scheduler a little simple with FIFO and shorter-tasks-first, so I could understand what the code was doing and explain it during the demo.

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