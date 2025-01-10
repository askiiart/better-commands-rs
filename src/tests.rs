#[cfg(test)]
use crate::*;
use std::{
    fs::remove_file,
    hash::{BuildHasher, Hasher, RandomState},
};
use std::{fs::File, os::unix::fs::FileExt, thread::sleep};
use std::process::Command;

/// Tests what stdout prints
#[test]
fn stdout_content() {
    let expected = "[\"helloooooooooo\", \"hiiiiiiiiiiiii\"]";
    assert_eq!(
        expected,
        format!(
            "{:?}",
            run(&mut Command::new("echo")
                .arg("-n")
                .arg("helloooooooooo\nhiiiiiiiiiiiii"))
            .stdout()
            .unwrap()
            .into_iter()
            .map(|line| { line.content })
            .collect::<Vec<String>>()
        )
    );
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
    let _ = thread::spawn(|| {
        let _ = run_funcs(
            Command::new("bash")
                .arg("-c")
                .arg("echo hi; >&2 echo hello"),
            {
                |stdout_lines| {
                    sleep(Duration::from_secs(1));
                    for _ in stdout_lines {
                        Command::new("bash")
                            .arg("-c")
                            .arg("echo stdout >> ./tmp-run_runcs")
                            .output()
                            .unwrap();
                    }
                }
            },
            {
                |stderr_lines| {
                    sleep(Duration::from_secs(3));
                    for _ in stderr_lines {
                        Command::new("bash")
                            .arg("-c")
                            .arg("echo stderr >> ./tmp-run_runcs")
                            .output()
                            .unwrap();
                    }
                }
            },
        );
    });
    sleep(Duration::from_secs(2));
    let f = File::open("./tmp-run_runcs").unwrap();
    let mut buf: [u8; 14] = [0u8; 14];
    f.read_at(&mut buf, 0).unwrap();
    assert_eq!(buf, [115, 116, 100, 111, 117, 116, 10, 0, 0, 0, 0, 0, 0, 0]);

    sleep(Duration::from_secs(2));
    f.read_at(&mut buf, 0).unwrap();
    assert_eq!(
        buf,
        [115, 116, 100, 111, 117, 116, 10, 115, 116, 100, 101, 114, 114, 10]
    );

    remove_file("./tmp-run_runcs").unwrap();
}

#[test]
fn test_run_funcs_with_lines() {
    let _ = thread::spawn(|| {
        let _ = run_funcs_with_lines(
            Command::new("bash")
                .arg("-c")
                .arg("echo hi; >&2 echo hello"),
            {
                |stdout_lines| {
                    sleep(Duration::from_secs(1));
                    for line in stdout_lines {
                        assert_eq!(line.unwrap(), "hi");
                        Command::new("bash")
                            .arg("-c")
                            .arg("echo stdout >> ./tmp-run_runcs_with_lines")
                            .output()
                            .unwrap();
                    }
                    return Vec::new();
                }
            },
            {
                |stderr_lines| {
                    sleep(Duration::from_secs(3));
                    for line in stderr_lines {
                        assert_eq!(line.unwrap(), "hello");
                        Command::new("bash")
                            .arg("-c")
                            .arg("echo stderr >> ./tmp-run_runcs_with_lines")
                            .output()
                            .unwrap();
                    }
                    return Vec::new();
                }
            },
        );
    });
    sleep(Duration::from_secs(2));
    let f = File::open("./tmp-run_runcs_with_lines").unwrap();
    let mut buf: [u8; 14] = [0u8; 14];
    f.read_at(&mut buf, 0).unwrap();
    assert_eq!(buf, [115, 116, 100, 111, 117, 116, 10, 0, 0, 0, 0, 0, 0, 0]);

    sleep(Duration::from_secs(2));
    f.read_at(&mut buf, 0).unwrap();
    assert_eq!(
        buf,
        [115, 116, 100, 111, 117, 116, 10, 115, 116, 100, 101, 114, 114, 10]
    );

    remove_file("./tmp-run_runcs_with_lines").unwrap();
}
