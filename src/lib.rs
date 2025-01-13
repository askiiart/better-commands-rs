#![doc = include_str!("../README.md")]
use std::cmp::Ordering;
use std::io::{BufRead, BufReader, Lines};
use std::process::{ChildStderr, ChildStdout, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

mod tests;

/// Holds the output for a command
///
/// Features the lines printed (see [`Line`]), the status code, the start time, end time, and duration
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmdOutput {
    lines: Option<Vec<Line>>,
    status_code: Option<i32>,
    start_time: Instant,
    end_time: Instant,
    duration: Duration,
}

impl CmdOutput {
    /// Returns only lines printed to stdout
    ///
    /// <small>This is an [`Option`] because [`run_funcs`] cannot provide `lines`</small>
    pub fn stdout(self) -> Option<Vec<Line>> {
        self.lines.and_then(|lines| {
            Some(
                lines
                    .into_iter()
                    .filter(|line| line.printed_to == LineType::Stdout)
                    .collect(),
            )
        })
    }

    /// Returns only lines printed to stderr
    ///
    /// <small>This is an [`Option`] because [`run_funcs`] cannot provide `lines`</small>
    pub fn stderr(self) -> Option<Vec<Line>> {
        self.lines.and_then(|lines| {
            Some(
                lines
                    .into_iter()
                    .filter(|line| line.printed_to == LineType::Stderr)
                    .collect(),
            )
        })
    }

    /// Returns all lines printed by the [`Command`]\
    /// Note: All functions are *guaranteed* to return either `Some()` or `None`, not either
    ///
    /// <small>This is an [`Option`] because [`run_funcs`] cannot provide `lines`</small>
    pub fn lines(self) -> Option<Vec<Line>> {
        return self.lines;
    }

    /// Returns the exit status code, if there was one
    ///
    /// Note that if the program exited due to a signal, like SIGKILL, it's possible it didn't exit with a status code, hence this being an [`Option`].
    pub fn status_code(self) -> Option<i32> {
        return self.status_code;
    }

    /// Returns the duration the command ran for
    pub fn duration(self) -> Duration {
        return self.duration;
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

/// Specifies what a line was printed to - stdout or stderr
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LineType {
    Stdout,
    Stderr,
}

/// A single line from the output of a command
#[derive(Debug, Clone, PartialEq, Eq, Ord)]
pub struct Line {
    /// Which stream the line was printed to
    pub printed_to: LineType,
    /// When the line was printed
    pub time: Instant,
    /// The content printed to the line
    pub content: String,
}

impl Line {
    /// Creates a [`Line`] from a string printed to stdout
    pub fn from_stdout<S: AsRef<str>>(content: S) -> Self {
        return Line {
            content: content.as_ref().to_string(),
            printed_to: LineType::Stdout,
            time: Instant::now(),
        };
    }

    /// Creates a [`Line`] from a string printed to stderr
    pub fn from_stderr<S: AsRef<str>>(content: S) -> Self {
        return Line {
            content: content.as_ref().to_string(),
            printed_to: LineType::Stderr,
            time: Instant::now(),
        };
    }
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

/// Runs a command, returning a [`CmdOutput`] (which *will* contain `Some(lines)`, not a None)
///
/// Example:
///
/// ```
/// use better_commands::run;
/// use std::process::Command;
/// let cmd = run(&mut Command::new("echo").arg("hi"));
///
/// // prints the following: [Line { printed_to: Stdout, time: Instant { tv_sec: 16316, tv_nsec: 283884648 }, content: "hi" }]
/// // (timestamp varies)
/// assert_eq!("hi", cmd.lines().unwrap()[0].content);
/// ```
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

    let stdout_lines = BufReader::new(child_stdout).lines();
    let stdout_thread = thread::spawn(move || {
        let mut lines: Vec<Line> = Vec::new();
        for line in stdout_lines {
            lines.push(Line {
                content: line.unwrap(),
                printed_to: LineType::Stdout,
                time: Instant::now(),
            });
        }
        return lines;
    });

    let stderr_lines = BufReader::new(child_stderr).lines();
    let stderr_thread = thread::spawn(move || {
        let mut lines: Vec<Line> = Vec::new();
        for line in stderr_lines {
            let time = Instant::now();
            lines.push(Line {
                content: line.unwrap(),
                printed_to: LineType::Stderr,
                time: time,
            });
        }
        return lines;
    });

    let status = child.wait().unwrap().code();
    let end = Instant::now();

    let mut lines = stdout_thread.join().unwrap();
    lines.append(&mut stderr_thread.join().unwrap());
    lines.sort();

    return CmdOutput {
        lines: Some(lines),
        status_code: status,
        start_time: start,
        end_time: end,
        duration: end.duration_since(start),
    };
}

/// Runs a command while simultaneously running a provided [`Fn`] as the command prints line-by-line
///
/// The [`CmdOutput`] *will* be None; this does *not* handle the lines - if you need them, use [`run`] or [`run_funcs_with_lines`]
///
/// Example:
///
/// ```
/// use better_commands::run_funcs;
/// use better_commands::Line;
/// use std::process::Command;
/// run_funcs(&mut Command::new("echo").arg("hi"), {
///     |stdout_lines| {
///         for line in stdout_lines {
///             /* send line to database */
///             }
///         }
///     },
///     {
///     |stderr_lines| {
///         // this code is for stderr and won't run because echo won't print anything to stderr
///     }
/// });
/// ```
pub fn run_funcs(
    command: &mut Command,
    stdout_func: impl Fn(Lines<BufReader<ChildStdout>>) -> () + std::marker::Send + 'static,
    stderr_func: impl Fn(Lines<BufReader<ChildStderr>>) -> () + std::marker::Send + 'static,
) -> CmdOutput {
    // https://stackoverflow.com/a/72831067/16432246
    let start = Instant::now();
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    let stdout_lines = BufReader::new(child_stdout).lines();
    let stdout_thread = thread::spawn(move || stdout_func(stdout_lines));

    let stderr_lines = BufReader::new(child_stderr).lines();
    let stderr_thread = thread::spawn(move || stderr_func(stderr_lines));

    let status = child.wait().unwrap().code();
    let end = Instant::now();

    stdout_thread.join().unwrap();
    stderr_thread.join().unwrap();

    return CmdOutput {
        lines: None,
        status_code: status,
        start_time: start,
        end_time: end,
        duration: end.duration_since(start),
    };
}

/// Runs a command while simultaneously running a provided [`Fn`] as the command prints line-by-line, including line handling
///
/// The [`CmdOutput`] *will* contain `Some(lines)`, not a None.
///
/// Example:
///
/// ```
/// use better_commands::run_funcs_with_lines;
/// use better_commands::Line;
/// use std::process::Command;
/// let cmd = run_funcs_with_lines(&mut Command::new("echo").arg("hi"), {
///     |stdout_lines| { // your function *must* return the lines
///         let mut lines = Vec::new();
///         for line in stdout_lines {
///             lines.push(Line::from_stdout(line.unwrap()));
///             /* send line to database */
///             }
///             return lines;
///         }
///     },
///     {
///     |stderr_lines| {
///         // this code is for stderr and won't run because echo won't print anything to stderr, so we'll just put this placeholder here
///         return Vec::new();
///     }
/// });
///
/// // prints the following: [Line { printed_to: Stdout, time: Instant { tv_sec: 16316, tv_nsec: 283884648 }, content: "hi" }]
/// // (timestamp varies)
/// assert_eq!("hi", cmd.lines().unwrap()[0].content);
/// ```
///
/// In order for the built-in `lines` functionality to work, your function must return the lines like this; if this doesn't work for you, you can use [`run`] or [`run_funcs`] instead.
/// ```ignore
/// use better_commands::Line;
///
/// let mut lines = Vec::new();
/// for line in stdout_lines {
///     lines.push(Line::from_stdout(line.unwrap())); // from_stdout/from_stderr depending on which
/// }
/// return lines;
/// ```
pub fn run_funcs_with_lines(
    command: &mut Command,
    stdout_func: impl Fn(Lines<BufReader<ChildStdout>>) -> Vec<Line> + std::marker::Send + 'static,
    stderr_func: impl Fn(Lines<BufReader<ChildStderr>>) -> Vec<Line> + std::marker::Send + 'static,
) -> CmdOutput {
    // https://stackoverflow.com/a/72831067/16432246
    let start = Instant::now();
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    let stdout_lines = BufReader::new(child_stdout).lines();
    let stderr_lines = BufReader::new(child_stderr).lines();

    let stdout_thread = thread::spawn(move || stdout_func(stdout_lines));
    let stderr_thread = thread::spawn(move || stderr_func(stderr_lines));

    let mut lines = stdout_thread.join().unwrap();
    let mut lines_printed_to_stderr = stderr_thread.join().unwrap();
    lines.append(&mut lines_printed_to_stderr);
    lines.sort();

    let status = child.wait().unwrap().code();
    let end = Instant::now();

    return CmdOutput {
        lines: Some(lines),
        status_code: status,
        start_time: start,
        end_time: end,
        duration: end.duration_since(start),
    };
}
