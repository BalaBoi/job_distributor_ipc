#[derive(Debug)]
pub struct WorkerConfig {
    pub worker_type: u16,
    pub worker_count: u32,
}

pub fn parse_worker_config() -> Vec<WorkerConfig> {
    let stdin = std::io::stdin();
    let mut out = Vec::new();
    let mut line = String::new();
    for _ in 0..5 {
        stdin
            .read_line(&mut line)
            .expect("failed to read line from stdin");
        let mut parts = line.split_whitespace();
        let worker_type: u16 = parts
            .next()
            .expect("each line should have two numbers")
            .parse()
            .expect("not a number");
        let worker_count: u32 = parts
            .next()
            .expect("each line should have two numbers")
            .parse()
            .expect("not a number");
        out.push(WorkerConfig {
            worker_type,
            worker_count,
        });
        line.clear();
    }
    out
}
