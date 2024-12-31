use crate::*;
use std::hash::{BuildHasher, Hasher, RandomState};

#[cfg(test)]

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
            .into_iter()
            .map(|line| { line.content })
            .collect::<Vec<String>>()
        )
    );
}

/// Tests what stderr prints
#[test]
fn stderr_content() {
    let expected = "[\"helloooooooooo\", \"hiiiiiiiiiiiii\"]";
    // `>&2` redirects to stderr
    assert_eq!(
        expected,
        format!(
            "{:?}",
            run(&mut Command::new("bash")
                .arg("-c")
                .arg("echo -n 'helloooooooooo\nhiiiiiiiiiiiii' >&2"))
            .stderr()
            .into_iter()
            .map(|line| { line.content })
            .collect::<Vec<String>>()
        )
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
            .status()
            .unwrap()
    );
}

/// Tests that the output is sorted by default
#[test]
fn test_output_is_sorted_sort_works() {
    let cmd = run(&mut Command::new("bash").arg("-c").arg("echo hi; sleep 0.01; echo hi; sleep 0.01; echo hi; sleep 0.01; echo hi; sleep 0.01; echo hi; sleep 0.01; echo hi; sleep 0.01; echo hi; sleep 0.01; echo hi; sleep 0.01; echo hi; sleep 0.01; echo hi")).stdout();
    let mut sorted = cmd.clone();
    // To avoid an accidental bogosort
    while sorted.is_sorted() {
        shuffle_vec(&mut sorted);
    }
    sorted.sort();
    assert_eq!(sorted, cmd);
}

// https://stackoverflow.com/a/78840539/16432246
// with rand::seq::SliceRandom and you can shuffle a Vec, but doing this to avoid introducing a ton of dependencies for one little test
fn shuffle_vec<T>(vec: &mut [T]) {
    let n: usize = vec.len();
    for i in 0..(n - 1) {
        // Generate random index j, such that: i <= j < n
        // The remainder (`%`) after division is always less than the divisor.
        let j = (RandomState::new().build_hasher().finish() as usize) % (n - i) + i;
        vec.swap(i, j);
    }
}