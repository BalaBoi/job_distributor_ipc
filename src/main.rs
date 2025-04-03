use job_distributor::{Manager, job_stream, parse_worker_config};

fn main() {
    let worker_configs = parse_worker_config();
    let job_stream = job_stream();
    let mut manager = Manager::new(&worker_configs);
    manager.process_jobs(job_stream);
}
