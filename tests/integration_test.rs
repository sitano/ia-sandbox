extern crate ia_sandbox;
extern crate tempdir;

use std::fs::File;
use std::io::{Read, Write};

use ia_sandbox::run_info::RunInfoResult;

#[macro_use]
mod utils;
use utils::ConfigBuilder;

const HELLO_WORLD: [(&'static str, &'static str); 1] =
    [("./target/debug/examples/hello_world", "/hello_world")];

const EXIT_WITH_INPUT: [(&'static str, &'static str); 1] = [
    (
        "./target/debug/examples/exit_with_input",
        "/exit_with_input",
    ),
];

const EXIT_WITH_LAST_ARGUMENT: [(&'static str, &'static str); 1] = [
    (
        "./target/debug/examples/exit_with_last_argument",
        "/exit_with_last_argument",
    ),
];

const KILL_WITH_SIGNAL_ARG: [(&'static str, &'static str); 1] = [
    (
        "./target/debug/examples/kill_with_signal_arg",
        "/kill_with_signal_arg",
    ),
];

#[test]
fn test_basic_sandbox() {
    utils::with_setup("test_basic_sandbox", HELLO_WORLD.iter(), |dir| {
        let run_info = ConfigBuilder::new(
            dir.join(HELLO_WORLD[0].1.trim_left_matches('/'))
                .to_string_lossy(),
        ).build_and_run()
            .unwrap();
        assert!(run_info.is_success());
    });
}

#[test]
fn test_exec_failed() {
    utils::with_setup("test_exec_failed", HELLO_WORLD[..].iter(), |dir| {
        let result = ConfigBuilder::new(dir.join("missing").to_string_lossy()).build_and_run();

        use ia_sandbox::errors::*;
        assert!(matches!(
            result,
            Err(Error(ErrorKind::ChildError(ChildError::ExecError(_)), _))
        ));
    });
}

#[test]
fn test_pivot_root() {
    utils::with_setup("test_pivot_root", HELLO_WORLD[..].iter(), |dir| {
        let run_info = ConfigBuilder::new(HELLO_WORLD[0].1)
            .new_root(dir.to_string_lossy())
            .build_and_run()
            .unwrap();
        assert!(run_info.is_success());
    });
}

#[test]
fn test_unshare_net() {
    utils::with_setup("test_unshare_net", HELLO_WORLD[..].iter(), |dir| {
        let run_info = ConfigBuilder::new(HELLO_WORLD[0].1)
            .new_root(dir.to_string_lossy())
            .share_net(false)
            .build_and_run()
            .unwrap();
        assert!(run_info.is_success());
    });
}

#[test]
fn test_redirect_stdin() {
    utils::with_setup("test_redirect_stdin", EXIT_WITH_INPUT[..].iter(), |dir| {
        let mut input = File::create(dir.join("input")).unwrap();
        input.write(b"0").unwrap();

        let run_info = ConfigBuilder::new(EXIT_WITH_INPUT[0].1)
            .new_root(dir.to_string_lossy())
            .stdin(dir.join("input").to_string_lossy())
            .build_and_run()
            .unwrap();
        assert!(run_info.is_success());
    });

    utils::with_setup("test_redirect_stdin", EXIT_WITH_INPUT[..].iter(), |dir| {
        let mut input = File::create(dir.join("input")).unwrap();
        input.write(b"23").unwrap();

        let run_info = ConfigBuilder::new(EXIT_WITH_INPUT[0].1)
            .new_root(dir.to_string_lossy())
            .stdin(dir.join("input").to_string_lossy())
            .build_and_run()
            .unwrap();
        assert!(matches!(
            run_info.result(),
            &RunInfoResult::NonZeroExitStatus(23)
        ));
    });
}

#[test]
fn test_redirect_stdout() {
    utils::with_setup("test_redirect_stdout", HELLO_WORLD[..].iter(), |dir| {
        let run_info = ConfigBuilder::new(HELLO_WORLD[0].1)
            .new_root(dir.to_string_lossy())
            .stdout(dir.join("output").to_string_lossy())
            .build_and_run()
            .unwrap();
        assert!(run_info.is_success());

        let mut output = File::open(dir.join("output")).unwrap();
        let mut line = String::new();
        output.read_to_string(&mut line).unwrap();
        assert_eq!(line, "Hello World!\n");
    });
}

#[test]
fn test_redirect_stderr() {
    utils::with_setup("test_redirect_stderr", HELLO_WORLD[..].iter(), |dir| {
        let run_info = ConfigBuilder::new(HELLO_WORLD[0].1)
            .new_root(dir.to_string_lossy())
            .stderr(dir.join("stderr").to_string_lossy())
            .build_and_run()
            .unwrap();
        assert!(run_info.is_success());

        let mut output = File::open(dir.join("stderr")).unwrap();
        let mut line = String::new();
        output.read_to_string(&mut line).unwrap();
        assert_eq!(line, "Hello stderr!\n");
    });
}

#[test]
fn test_arguments() {
    utils::with_setup(
        "test_arguments",
        EXIT_WITH_LAST_ARGUMENT[..].iter(),
        |dir| {
            let run_info = ConfigBuilder::new(EXIT_WITH_LAST_ARGUMENT[0].1)
                .new_root(dir.to_string_lossy())
                .arg("0")
                .build_and_run()
                .unwrap();
            assert!(run_info.is_success());
        },
    );

    utils::with_setup(
        "test_arguments",
        EXIT_WITH_LAST_ARGUMENT[..].iter(),
        |dir| {
            let run_info = ConfigBuilder::new(EXIT_WITH_LAST_ARGUMENT[0].1)
                .new_root(dir.to_string_lossy())
                .args(vec!["24", "0", "17"])
                .build_and_run()
                .unwrap();
            assert!(matches!(
                run_info.result(),
                &RunInfoResult::NonZeroExitStatus(17)
            ));
        },
    );
}

#[test]
fn test_killed_by_signal() {
    utils::with_setup(
        "test_killed_by_signal",
        KILL_WITH_SIGNAL_ARG[..].iter(),
        |dir| {
            let run_info = ConfigBuilder::new(KILL_WITH_SIGNAL_ARG[0].1)
                .new_root(dir.to_string_lossy())
                .arg("8")
                .build_and_run()
                .unwrap();
            println!("{}", run_info);
            assert!(matches!(
                run_info.result(),
                &RunInfoResult::KilledBySignal(8)
            ));
        },
    );

    utils::with_setup(
        "test_redirect_stdin",
        KILL_WITH_SIGNAL_ARG[..].iter(),
        |dir| {
            let run_info = ConfigBuilder::new(KILL_WITH_SIGNAL_ARG[0].1)
                .new_root(dir.to_string_lossy())
                .arg("11")
                .build_and_run()
                .unwrap();
            assert!(matches!(
                run_info.result(),
                &RunInfoResult::KilledBySignal(11)
            ));
        },
    );
}
