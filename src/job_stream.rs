use std::io::{BufRead, StdinLock, stdin};

pub fn job_stream() -> JobStream<StdinLock<'static>> {
    JobStream::new(stdin().lock())
}

pub struct JobStream<R> {
    reader: R,
}

impl<R> JobStream<R> {
    fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R> Iterator for JobStream<R>
where
    R: BufRead,
{
    type Item = Job;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();

        match self.reader.read_line(&mut line) {
            Ok(0) => None,
            Ok(_) => {
                if line.trim().is_empty() {
                    println!("job stream ended");
                    return None;
                }

                let mut parts = line.split_whitespace();

                let job_type: u16 = parts
                    .next()
                    .expect("there should be two numbers in each job line")
                    .parse()
                    .expect("should be a number");
                let job_duration: u32 = parts
                    .next()
                    .expect("there should be two numbers in each job line")
                    .parse()
                    .expect("should be a number");

                Some(Job {
                    job_type,
                    job_duration,
                })
            }
            Err(error) => panic!("error while reading job line: {}", error),
        }
    }
}

#[derive(Debug)]
pub struct Job {
    pub job_type: u16,
    pub job_duration: u32,
}
