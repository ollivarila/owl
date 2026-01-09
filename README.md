# Owl

A simple command-line tool written in Rust that watches a directory (and all of its subdirectories) for file changes and runs a specified command whenever changes are detected.

This tool is useful for lightweight automation tasks such as rebuilding a project, running tests, or restarting a service when files change.

## Features

* Recursively watches a directory and all subdirectories
* Executes a shell command when file modifications are detected
* Debounces rapid file events to avoid repeated executions
* Uses Linux `inotify` for efficient filesystem event handling

## Requirements

* Linux (uses `inotify`)
* Rust toolchain (stable)

## Installation

```sh
cargo install --git https://github.com/ollivarila/owl
```

## Usage

See help:
```sh
owl --help
```

## How It Works

* Recursively collects all subdirectories starting from the provided root directory
* Registers each directory with `inotify` for `MODIFY` events
* Blocks until filesystem events are received
* Applies a short debounce delay (250ms) to group rapid changes
* Executes the provided command once per detected change batch

## Limitations

* Only listens for file modification events (`MODIFY`)
* New directories created after startup are not automatically watched
* Command parsing is whitespace-based (no shell features like pipes or globbing)
