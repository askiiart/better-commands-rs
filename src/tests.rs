#[cfg(test)]
use crate::*;
use std::io::Write;
use std::os::unix::fs::FileExt;
use std::process::Command;
use std::{
    fs::remove_file,
    hash::{BuildHasher, Hasher, RandomState},
};
use std::{fs::File, thread::sleep};

/// Tests what stdout prints
#[test]
fn stdout_content() {
    let expected = vec!["helloooooooooo".to_string(), "hiiiiiiiiiiiii".to_string()];

    let output = run(Command::new("echo").arg(
        "helloooooooooo
hiiiiiiiiiiiii",
    ))
    .stdout()
    .unwrap()
    .into_iter()
    .map(|line| line.content)
    .collect::<Vec<String>>();

    assert_eq!(expected, output);
}

/// Tests what stderr prints
#[test]
fn stderr_content() {
    let expected = vec!["helloooooooooo", "hiiiiiiiiiiiii"];
    // `>&2` redirects to stderr
    assert_eq!(
        expected,
        run(&mut Command::new("bash")
            .arg("-c")
            .arg("echo -n 'helloooooooooo\nhiiiiiiiiiiiii' >&2"))
        .stderr()
        .unwrap()
        .into_iter()
        .map(|line| { line.content })
        .collect::<Vec<String>>()
    );
}

/// Tests what stderr prints
#[test]
fn test_exit_code() {
    let expected = 10;
    // `>&2` redirects to stderr
    assert_eq!(
        expected,
        run(&mut Command::new("bash").arg("-c").arg("exit 10"))
            .status_code()
            .unwrap()
    );
}

/// Tests that the output is sorted by default
#[test]
fn test_output_is_sorted_sort_works() {
    let cmd = run(&mut Command::new("bash")
        .arg("-c")
        .arg("echo hi; echo hi; echo hi; echo hi; echo hi"))
    .stdout()
    .unwrap();
    let mut sorted = cmd.clone();
    // To avoid an accidental bogosort
    while sorted.is_sorted() {
        shuffle_vec(&mut sorted);
    }
    sorted.sort();
    assert_eq!(sorted, cmd);
}

// https://stackoverflow.com/a/78840539/16432246
// with rand::seq::SliceRandom you can shuffle a Vec, but doing this to avoid introducing a ton of dependencies for one little test
fn shuffle_vec<T>(vec: &mut [T]) {
    let n: usize = vec.len();
    for i in 0..(n - 1) {
        // Generate random index j, such that: i <= j < n
        // The remainder (`%`) after division is always less than the divisor.
        let j = (RandomState::new().build_hasher().finish() as usize) % (n - i) + i;
        vec.swap(i, j);
    }
}

#[test]
fn test_run_funcs() {
    // TODO: Add error handling to delete the file on exit
    File::create_new("./tmp-run_funcs").unwrap();
    let threads = thread::spawn(|| {
        return run_funcs(
            Command::new("bash")
                .arg("-c")
                .arg("echo hi; >&2 echo hello"),
            {
                |stdout_lines| {
                    sleep(Duration::from_secs(1));
                    for _ in stdout_lines {
                        let mut f = File::options()
                            .write(true)
                            .open("./tmp-run_funcs")
                            .unwrap();
                        f.write_all(b"stdout\n").unwrap();
                        drop(f);
                    }
                }
            },
            {
                |stderr_lines| {
                    sleep(Duration::from_secs(3));
                    for _ in stderr_lines {
                        let f = File::options()
                            .write(true)
                            .open("./tmp-run_funcs")
                            .unwrap();
                        f.write_at(b"stderr\n", 7).unwrap();
                        drop(f);
                    }
                }
            },
        );
    });
    sleep(Duration::from_secs(2));
    let f = File::open("./tmp-run_funcs").unwrap();
    let mut buf: [u8; 14] = [0u8; 14];
    f.read_at(&mut buf, 0).unwrap();
    assert_eq!(buf, [115, 116, 100, 111, 117, 116, 10, 0, 0, 0, 0, 0, 0, 0]);

    sleep(Duration::from_secs(2));
    f.read_at(&mut buf, 0).unwrap();
    assert_eq!(
        buf,
        [115, 116, 100, 111, 117, 116, 10, 115, 116, 100, 101, 114, 114, 10]
    );

    remove_file("./tmp-run_funcs").unwrap();

    let output = threads.join().unwrap();
    assert_eq!(output.clone().lines(), None);
}

#[test]
fn test_run_funcs_with_lines() {
    // TODO: Add error handling to delete the file on exit
    File::create_new("./tmp-run_funcs_with_lines").unwrap();
    let threads = thread::spawn(|| {
        return run_funcs_with_lines(
            &mut Command::new("bash")
                .arg("-c")
                .arg("echo hi; >&2 echo hello"),
            {
                |stdout_lines| {
                    let mut lines: Vec<Line> = Vec::new();
                    sleep(Duration::from_secs(1));
                    for line in stdout_lines {
                        let line = line.unwrap();
                        lines.push(Line::from_stdout(&line));
                        assert_eq!(&line, "hi");
                        let mut f = File::options()
                            .write(true)
                            .open("./tmp-run_funcs_with_lines")
                            .unwrap();
                        f.write_all(b"stdout\n").unwrap();
                        drop(f);
                    }
                    return lines;
                }
            },
            {
                |stderr_lines| {
                    let mut lines: Vec<Line> = Vec::new();
                    sleep(Duration::from_secs(3));
                    for line in stderr_lines {
                        let line = line.unwrap();
                        lines.push(Line::from_stdout(&line));
                        assert_eq!(line, "hello");
                        let mut f = File::options()
                            .write(true)
                            .append(true)
                            .open("./tmp-run_funcs_with_lines")
                            .unwrap();
                        f.write(b"stderr\n").unwrap();
                        drop(f);
                    }
                    return lines;
                }
            },
        );
    });
    sleep(Duration::from_secs(2));
    let read = std::fs::read_to_string("tmp-run_funcs_with_lines").unwrap();
    assert_eq!(read, "stdout\n");

    sleep(Duration::from_secs(2));
    let read = std::fs::read_to_string("tmp-run_funcs_with_lines").unwrap();
    assert_eq!(read, "stdout\nstderr\n");


    remove_file("./tmp-run_funcs_with_lines").unwrap();

    let output = threads.join().unwrap();

    println!("{:?}", output);

    assert_eq!(output.clone().lines().unwrap()[0].content, "hi");
    assert_eq!(output.lines().unwrap()[1].content, "hello");
}
