# Job Distributor (IPC)

Should work on all (sufficiently recent) linux distributions. Empty line denotes the end of all inputs.

## Setup
1. Install Rust (https://rustup.rs/)
2. From within the directory
```
cargo run
```

3. Once the program is running, it expects first five lines from stdin to be the worker configuration data (\<worker-type\> \<worker-count\>), the next set of lines are treated as a stream of job requests (\<job-type\> \<job-duration\>) which is terminated with an empty line.