use std::{
    collections::{HashMap, hash_map::Entry},
    io::BufRead,
    os::fd::{AsFd, AsRawFd, OwnedFd},
    process::exit,
    time::Duration,
};

use nix::{
    errno::Errno,
    sys::{
        select::{FdSet, select},
        wait::{WaitStatus, wait},
    },
    unistd::{self, ForkResult, Pid, fork, pipe},
};

use crate::{
    config::WorkerConfig,
    job_stream::{Job, JobStream},
};

pub struct Manager {
    workers: HashMap<u16, Vec<Worker>>,
}

impl Manager {
    pub fn new(worker_configs: &[WorkerConfig]) -> Self {
        let mut map: HashMap<u16, Vec<Worker>> = HashMap::new();
        for &WorkerConfig {
            worker_type,
            worker_count,
        } in worker_configs
        {
            for _ in 0..worker_count {
                let worker = Manager::spawn_worker(worker_type);

                match map.entry(worker_type) {
                    Entry::Occupied(mut occ) => {
                        occ.get_mut().push(worker);
                    }
                    Entry::Vacant(vac) => {
                        vac.insert(vec![worker]);
                    }
                }
            }
        }
        Self { workers: map }
    }

    fn spawn_worker(worker_type: u16) -> Worker {
        let (to_worker_read, to_worker_write) = pipe().expect("failed to make unidirectional pipe");
        let (from_worker_read, from_worker_write) =
            pipe().expect("failed to make unidirectional pipe");
        //SAFETY: this is not a multithreaded program, so it's fine to fork
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                drop(from_worker_write);
                drop(to_worker_read);
                println!(
                    "Created worker process with pid {} of type {}",
                    child, worker_type
                );
                Worker {
                    pid: child,
                    worker_type,
                    ready_receiver: from_worker_read,
                    job_sender: to_worker_write,
                }
            }
            Ok(ForkResult::Child) => {
                drop(from_worker_read);
                drop(to_worker_write);
                run_worker(worker_type, from_worker_write, to_worker_read);
                exit(0);
            }
            Err(error) => {
                panic!("fork failed with error: {}", error);
            }
        }
    }

    pub fn process_jobs<R: BufRead>(&mut self, jobs: JobStream<R>) {
        for Job {
            job_type,
            job_duration,
        } in jobs
        {
            println!(
                "manager distributing job of type {} with duration {}",
                job_type, job_duration
            );
            let mut fd_set = FdSet::new();
            for worker in self
                .workers
                .get(&job_type)
                .expect("no workers of this job type")
            {
                fd_set.insert(worker.ready_receiver.as_fd());
            }

            select(None, Some(&mut fd_set), None, None, None).expect("error in select call");

            let completion_receiver = fd_set
                .fds(None)
                .into_iter()
                .next()
                .expect("there should be a free fd after the select call")
                .as_raw_fd();

            let free_worker = self
                .workers
                .get(&job_type)
                .unwrap()
                .iter()
                .find(|worker| worker.ready_receiver.as_raw_fd() == completion_receiver)
                .unwrap();

            println!(
                "worker(type: {}, pid: {}) is free",
                free_worker.worker_type, free_worker.pid
            );
            let mut buf = [0u8; 4];
            unistd::read(completion_receiver, &mut buf).expect("failed in read call"); //did select call before this so it shouldn't block

            let job_duration_bytes = job_duration.to_ne_bytes();
            unistd::write(free_worker.job_sender.as_fd(), &job_duration_bytes)
                .expect("failed in write call");
        }
        println!("manager finished with handling jobs");
        self.stop_workers();
    }

    fn stop_workers(&mut self) {
        let workers = std::mem::take(&mut self.workers);
        let _ready_receivers = workers
            .into_values()
            .flatten()
            .map(|w| w.ready_receiver)
            .collect::<Vec<_>>(); //this will close all the job senders, leading the workers to eventually stop after finishing their work
        loop {
            match wait() {
                Ok(WaitStatus::Exited(pid, status)) => {
                    println!("Worker with pid {} exited with status {}", pid, status);
                }
                Ok(_) => continue,
                Err(Errno::ECHILD) => {
                    println!("All children have exited");
                    break;
                }
                Err(error) => panic!(
                    "Unexpected error when waiting for workers to exit: {}",
                    error
                ),
            }
        }
    }
}

#[derive(Debug)]
struct Worker {
    pid: Pid,
    worker_type: u16,
    pub ready_receiver: OwnedFd,
    pub job_sender: OwnedFd,
}

fn run_worker(worker_type: u16, ready_sender: OwnedFd, job_receiver: OwnedFd) {
    let ready_signal = 0u32.to_ne_bytes();
    //send ready signal to let the manager know that the worker is ready
    unistd::write(ready_sender.as_fd(), &ready_signal)
        .expect("failed to send ready signal from worker");
    let mut buf = [0u8; 4];
    loop {
        println!(
            "worker(type: {}, pid: {}) is free, waiting for jobs",
            worker_type,
            std::process::id()
        );
        match unistd::read(job_receiver.as_raw_fd(), &mut buf) {
            Ok(0) => {
                println!(
                    "worker(type: {}, pid: {}) job receiving pipe closed, exiting",
                    worker_type,
                    std::process::id()
                );
                break;
            }
            Ok(_) => {
                let job_duration = u32::from_ne_bytes(buf);

                println!(
                    "worker(type: {}, pid: {}) processing job for {}s",
                    worker_type,
                    std::process::id(),
                    job_duration
                );
                std::thread::sleep(Duration::from_secs(job_duration as u64));

                unistd::write(ready_sender.as_fd(), &ready_signal)
                    .expect("failed to send ready signal from worker");
            }
            Err(error) => panic!("read call errored in worker with {}", error),
        }
    }
}
