use std::{path::PathBuf, thread::sleep, time::Duration};

use anyhow::{Context, Result, bail, ensure};
use clap::Parser;
use inotify::{Inotify, WatchMask};

const BUF_SIZE: usize = 4096;
const DEBOUNCE_DURATION: Duration = Duration::from_millis(250);

fn main() -> Result<()> {
    let args = Args::parse();

    let mut inotify = inotify::Inotify::init()?;

    let mut paths = Vec::with_capacity(10);
    get_sub_dirs(&args.dir, &mut paths)?;

    for path in paths {
        inotify.watches().add(path, WatchMask::MODIFY)?;
    }

    println!("Waiting for events in dir `{}`", &args.dir);
    let mut buf = [0u8; BUF_SIZE];
    loop {
        // NOTE: should implement debounce correctly
        // but this works for now
        sleep(DEBOUNCE_DURATION);

        let count = read_events(&mut inotify, &mut buf)?;

        if count > 0 {
            println!("Changes detected, running command `{}`\n", &args.command);
            run_command(&args.command)?;
        }

        // flush events that came while command was running
        // so we don't immediately run again
        let _ = inotify.read_events(&mut buf).is_ok();
    }
}

/// Watch a directory and its subdirectories and execute given command
/// when changes are made.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Command to run
    #[arg(short, long)]
    command: String,
    /// Directory to watch.
    #[arg(short, long, default_value = ".")]
    dir: String,
}

fn get_sub_dirs(path: impl Into<PathBuf>, acc: &mut Vec<PathBuf>) -> Result<()> {
    let path: PathBuf = path.into();
    ensure!(path.is_dir());

    acc.push(path.clone());

    for entry in path.read_dir()? {
        let path = entry?.path();
        if path.is_dir() {
            get_sub_dirs(path, acc)?;
        }
    }

    Ok(())
}

fn run_command(cmd: &str) -> Result<()> {
    ensure!(!cmd.is_empty(), "Command cannot be empty string");
    let mut parts = cmd.split_whitespace();
    let mut command = std::process::Command::new(parts.next().unwrap());

    for arg in parts {
        command.arg(arg);
    }

    let status = command.status().context("Failed to run command")?;

    if !status.success() {
        let code = status.code().unwrap_or(1);
        eprintln!("Command exited with status code `{code}`");
    }

    Ok(())
}

fn read_events(inotify: &mut Inotify, buf: &mut [u8]) -> Result<usize> {
    let mut count = inotify
        .read_events_blocking(buf)
        .context("Failed to read update events")
        .map(|e| e.count())?;

    // debounce
    loop {
        sleep(DEBOUNCE_DURATION);
        match inotify.read_events(buf) {
            Ok(events) => {
                let new_events_count = events.count();

                if new_events_count == 0 {
                    break;
                }

                count += new_events_count;
            }
            Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock) => {
                // no new events
                break;
            }
            Err(e) => bail!("Failed to read update events: `{e:?}`"),
        }
    }

    Ok(count)
}
