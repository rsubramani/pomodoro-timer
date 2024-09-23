# Pomodoro Timer CLI 🍝

## Overview

The **Pomodoro Timer CLI** is a command-line tool built with Rust that helps users manage work and break intervals using the Pomodoro technique. It features customizable timers, session logging, and optional sound notifications to keep you productive.

## Features

- Start, pause, and resume Pomodoro timers.
- Customize work and break durations.
- Session logging to track completed Pomodoro intervals.
- View session statistics at any time.
- Optional sound notifications when a session ends.

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) must be installed on your machine.
- Ensure you have `cargo`, the Rust package manager, available in your terminal.

### Build from Source

1. Clone the repository:

    ```bash
    git clone https://github.com/your-username/pomodoro_timer_cli.git
    cd pomodoro_timer_cli
    ```

2. Build the project:

    ```bash
    cargo build --release
    ```

3. Run the application:

    ```bash
    cargo run
    ```

## Usage

You can start a Pomodoro timer with the following command:

```bash
cargo run -- --work 25 --break 5
```

This starts a timer with 25 minutes of work and 5 minutes of break. Both values are customizable.


### Commands
- `--work`: Set the duration of the work period (in minutes).
- `--break`: Set the duration of the break period (in minutes).
- `stats`: View the statistics of your completed Pomodoro sessions.

Example:
```bash
cargo run -- --work 30 --break 10
```

To view session statistics:
```bash
cargo run -- stats
```

## Configuration
The default work and break durations are 25 and 5 minutes, respectively. You can change these via command-line arguments, as shown above.

## Logging
The app logs your completed Pomodoro sessions to `session_log.json` in the project directory. Each session is recorded with the date and number of completed Pomodoros.

## Optional Sound Notifications
If you want sound notifications when a timer ends, ensure you have an `alarm_sound.mp3` file in the root of the project directory. This feature can be added and customized using the `rodio` crate.

## Contributing
1. Fork the repository.
2. Create your feature branch (`git checkout -b feature/my-feature`).
3. Commit your changes (`git commit -m 'Add new feature'`).
4. Push to the branch (`git push origin feature/my-feature`).
5. Open a pull request.
