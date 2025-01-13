# Better Commands

[![Crates.io Version](https://img.shields.io/crates/v/better-commands)](https://crates.io/crates/better-commands) [![docs.rs](https://img.shields.io/docsrs/better-commands)](https://docs.rs/better-commands/latest/better_commands/)\
This crate provides the ability to more easily run a [`Command`] while also doing something with its output as it runs, as well as providing some extra functionality.

## Why?

Rust's stock [`Command`] kinda sucks, usually it only lets you get the output of a command with stdout and stderr completely separate. This crate lets you run code to handle the output as it prints, as well as providing all the lines together, with timestamps and which stream the line was printed to.

## Features

- Specifies whether a [`Line`] is printed to stderr or stderr
- Provides a timestamp for each [`Line`]
- Provides timestamps for the command as a whole (start, end, and duration)

A basic example (see [`run`]):

```rust
use better_commands::run;
use std::process::Command;

let output = run(Command::new("sleep").arg("1"));
println!("{:?}", output.duration());
```

A more complex example - this lets you provide a function to be run using the output from the command in real-time (see [`run_funcs_with_lines`]):

```rust
use better_commands::run_funcs_with_lines;
use better_commands::Line;
use std::process::Command;
let cmd = run_funcs_with_lines(&mut Command::new("echo").arg("hi"), {
    |stdout_lines| { // your function *must* return the lines
        let mut lines = Vec::new();
        for line in stdout_lines {
            lines.push(Line::from_stdout(line.unwrap()));
            /* send line to database */
            }
            return lines;
        }
    },
    {
    |stderr_lines| {
        // this code is for stderr and won't run because echo won't print anything to stderr, so we'll just put this placeholder here
        return Vec::new();
    }
});

// prints the following: [Line { printed_to: Stdout, time: Instant { tv_sec: 16316, tv_nsec: 283884648 }, content: "hi" }]
// (timestamp varies)
assert_eq!("hi", cmd.lines().unwrap()[0].content);
```
