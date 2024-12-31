use std::cmp::Ordering;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

mod tests;

/// Holds the output for a command
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmdOutput {
    lines: Vec<Line>,
    status: Option<i32>,
    start_time: Instant,
    end_time: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord)]
pub struct Line {
    pub stdout: bool,
    pub time: Instant,
    pub content: String,
}

impl CmdOutput {
    /// Returns only stdout
    pub fn stdout(self) -> Vec<Line> {
        return self
            .lines
            .into_iter()
            .filter(|l| {
                if l.stdout {
                    return true;
                }
                return false;
            })
            .collect();
    }

    /// Returns only stdout
    pub fn stderr(self) -> Vec<Line> {
        return self
            .lines
            .into_iter()
            .filter(|l| {
                if !l.stdout {
                    return true;
                }
                return false;
            })
            .collect();
    }

    /// Returns all output
    pub fn lines(self) -> Vec<Line> {
        return self.lines;
    }

    /// Returns the exit status code, if there was one
    pub fn status(self) -> Option<i32> {
        return self.status;
    }

    /// Returns the duration the command ran for
    pub fn duration(self) -> Duration {
        return self.end_time.duration_since(self.start_time);
    }

    /// Returns the time the command was started at
    pub fn start_time(self) -> Instant {
        return self.start_time;
    }

    /// Returns the time the command finished at
    pub fn end_time(self) -> Instant {
        return self.end_time;
    }
}

pub fn run(command: &mut Command) -> CmdOutput {
    // https://stackoverflow.com/a/72831067/16432246
    let start = Instant::now();
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    let (stdout_tx, stdout_rx) = std::sync::mpsc::channel();
    let (stderr_tx, stderr_rx) = std::sync::mpsc::channel();

    let stdout_lines = BufReader::new(child_stdout).lines();
    thread::spawn(move || {
        for line in stdout_lines {
            stdout_tx
                .send(Line {
                    content: line.unwrap(),
                    stdout: true,
                    time: Instant::now(),
                })
                .unwrap();
        }
    });

    let stderr_lines = BufReader::new(child_stderr).lines();
    thread::spawn(move || {
        for line in stderr_lines {
            let time = Instant::now();
            stderr_tx
                .send(Line {
                    content: line.unwrap(),
                    stdout: false,
                    time: time,
                })
                .unwrap();
        }
    });

    let status = child.wait().unwrap().code();
    let end = Instant::now();

    let mut lines = stdout_rx.into_iter().collect::<Vec<Line>>();
    lines.append(&mut stderr_rx.into_iter().collect::<Vec<Line>>());
    //lines.sort();

    return CmdOutput {
        lines: lines,
        status: status,
        start_time: start,
        end_time: end,
    };
}

impl PartialOrd for Line {
    fn ge(&self, other: &Line) -> bool {
        if self.time >= other.time {
            return true;
        }
        return false;
    }

    fn gt(&self, other: &Self) -> bool {
        if self.time > other.time {
            return true;
        }
        return false;
    }

    fn le(&self, other: &Self) -> bool {
        if self.time <= other.time {
            return true;
        }
        return false;
    }

    fn lt(&self, other: &Self) -> bool {
        if self.time < other.time {
            return true;
        }
        return false;
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self < other {
            return Some(Ordering::Less);
        }
        if self > other {
            return Some(Ordering::Greater);
        }
        return Some(Ordering::Equal);
    }
}
