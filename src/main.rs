#![allow(unused)]

use std::{
    collections::HashMap,
    env, fs, io,
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};

use anyhow::{bail, Context, Result};
use chrono::Local;
use clap::Parser;

mod from_grcov;
mod util;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(long)]
    path: PathBuf,
}

fn main() -> Result<()> {
    #[rustfmt::skip]
    let Cli {
        path,
    } = Cli::try_parse()?;

    // TODO: check grcov binary validity (existance and version)

    // TODO: use Path{,Buf}
    const TARGET_DIR: &str = "./target/coverage";

    env::set_current_dir(path)?;

    // NOTE: prepare TARGET_DIR
    {
        let res = fs::create_dir_all(TARGET_DIR);

        match res {
            Ok(_) => {}
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                eprintln!("{} already exists, attempting to remove", TARGET_DIR);
                util::remove_contents(TARGET_DIR)?;
            }
            Err(err) => {
                bail!(format!("Could not create {}: {}", TARGET_DIR, err))
            }
        }
    }

    // NOTE: run tests (with coverage info)
    let before_tests_time = SystemTime::now();
    {
        let mut cmd = Command::new("cargo")
            .args(["+nightly", "test", "--target-dir", TARGET_DIR])
            .envs(HashMap::from([
                // TODO: inherit old `${RUSTFLAGS}`
                ("RUSTFLAGS", "-Zprofile"),
                ("CARGO_INCREMENTAL", "0"),
                ("RUST_BACKTRACE", "1"),
                ("RUST_MIN_STACK", "8388608"),
            ]))
            .spawn()?;

        cmd.wait()?;
    }
    let after_tests_time = SystemTime::now();

    // TODO: `{before,after}-test` shenanigans/optimizations

    // NOTE: run grcov
    {
        // NOTE: extrapolated from running the original `grcov` CLI with appropriate arguments
        let opt = from_grcov::Opt {
            paths: vec![TARGET_DIR.to_string()],
            binary_path: None,
            llvm_path: None,
            output_types: vec![from_grcov::OutputType::Html],
            output_path: Some(PathBuf::from(TARGET_DIR)),
            output_config_file: None,
            source_dir: None,
            prefix_dir: None,
            ignore_not_existing: false,
            ignore_dir: vec![],
            keep_dir: vec![],
            path_mapping: None,
            branch: false,
            filter: None,
            sort_output_types: vec![from_grcov::OutputType::Markdown],
            llvm: true,
            token: None,
            commit_sha: None,
            service_name: None,
            service_number: None,
            service_job_id: None,
            service_pull_request: None,
            service_flag_name: None,
            parallel: false,
            threads: None,
            precision: 2,
            guess_directory: false,
            vcs_branch: "master".to_string(),
            log: PathBuf::from("stderr"),
            log_level: from_grcov::LevelFilterArg(log::LevelFilter::Error),
            excl_line: None,
            excl_start: None,
            excl_stop: None,
            excl_br_line: None,
            excl_br_start: None,
            excl_br_stop: None,
            no_demangle: false,
        };

        from_grcov::main(opt);
    }

    // NOTE: run grcov (old way, through CLI)
    // {
    //     // grcov target/coverage --llvm -t html -o target/coverage
    //     let mut cmd = Command::new("grcov")
    //         .args([TARGET_DIR, "--llvm", "-t", "html", "-o", TARGET_DIR])
    //         .spawn()?;
    //
    //     cmd.wait()?;
    // }

    // NOTE: open resulting report
    eprintln!("Successfully generated report, opening...");
    open::that(format!("{}/html/index.html", TARGET_DIR))?;

    Ok(())
}
