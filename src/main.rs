// src/main.rs

use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    style::{Color, Stylize},
    terminal, ExecutableCommand,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{stdout, Write},
};
use tokio::{self, time::{sleep, Duration}};
use chrono::Local;

#[derive(Subcommand)]
enum Commands {
    Stats,
}

#[derive(Parser)]
#[command(name = "Pomodoro Timer")]
#[command(author = "Your Name <you@example.com>")]
#[command(version = "1.0")]
#[command(about = "A simple Pomodoro Timer CLI tool", long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "WORK_MINUTES", default_value = "25")]
    work: u64,

    #[arg(short, long, value_name = "BREAK_MINUTES", default_value = "5")]
    break_duration: u64,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Stats) => {
            display_stats().expect("Failed to display stats");
        }
        None => {
            println!("Work Duration: {} minutes", cli.work);
            println!("Break Duration: {} minutes", cli.break_duration);
            start_pomodoro(cli.work, cli.break_duration).await;
        }
    }
}

async fn start_pomodoro(work_duration: u64, break_duration: u64) {
    loop {
        println!("\nStarting work session for {} minutes...", work_duration);
        countdown_timer(work_duration * 60, "Work").await;

        println!("\nWork session completed! Time for a break.");
        log_session().expect("Failed to log session");

        println!("Starting break for {} minutes...", break_duration);
        countdown_timer(break_duration * 60, "Break").await;

        println!("\nBreak over! Ready for the next session.");
    }
}

async fn countdown_timer(mut total_seconds: u64, session_type: &str) {
    let mut stdout = stdout();

    stdout.execute(terminal::EnterAlternateScreen).unwrap();
    terminal::enable_raw_mode().unwrap();

    loop {
        // Display the timer
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        print!(
            "\r{} Time Remaining: {:02}:{:02}  [Press 'p' to pause, 'q' to quit]",
            session_type.green(),
            minutes,
            seconds
        );
        stdout.flush().unwrap();

        // Check for user input
        if event::poll(Duration::from_millis(100)).unwrap() {
            if let CEvent::Key(key_event) = event::read().unwrap() {
                match key_event.code {
                    KeyCode::Char('p') => {
                        println!("\nPaused. Press 'r' to resume, 'q' to quit.");
                        loop {
                            if let CEvent::Key(resume_key_event) = event::read().unwrap() {
                                if resume_key_event.code == KeyCode::Char('r') {
                                    println!("Resuming...");
                                    break;
                                } else if resume_key_event.code == KeyCode::Char('q') {
                                    cleanup_terminal();
                                    std::process::exit(0);
                                }
                            }
                        }
                    }
                    KeyCode::Char('q') => {
                        cleanup_terminal();
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
        }

        // Wait for one second
        sleep(Duration::from_secs(1)).await;

        if total_seconds == 0 {
            play_sound();
            break;
        }
        total_seconds -= 1;
    }

    cleanup_terminal();
}

fn cleanup_terminal() {
    terminal::disable_raw_mode().unwrap();
    stdout().execute(terminal::LeaveAlternateScreen).unwrap();
}

#[derive(Serialize, Deserialize)]
struct SessionLog {
    date: String,
    work_sessions: u32,
}

fn log_session() -> Result<(), Box<dyn std::error::Error>> {
    let log_file = "session_log.json";
    let today = Local::now().format("%Y-%m-%d").to_string();

    let mut logs: Vec<SessionLog> = if let Ok(data) = fs::read_to_string(log_file) {
        serde_json::from_str(&data)?
    } else {
        Vec::new()
    };

    if let Some(entry) = logs.iter_mut().find(|log| log.date == today) {
        entry.work_sessions += 1;
    } else {
        logs.push(SessionLog {
            date: today,
            work_sessions: 1,
        });
    }

    fs::write(log_file, serde_json::to_string_pretty(&logs)?)?;
    Ok(())
}

fn display_stats() -> Result<(), Box<dyn std::error::Error>> {
    let log_file = "session_log.json";
    if let Ok(data) = fs::read_to_string(log_file) {
        let logs: Vec<SessionLog> = serde_json::from_str(&data)?;

        println!("\nPomodoro Sessions:\n");
        for log in logs {
            println!("Date: {}, Work Sessions Completed: {}", log.date, log.work_sessions);
        }
    } else {
        println!("No session data found.");
    }
    Ok(())
}

use rodio::{Decoder, OutputStream, Sink};
use std::io::BufReader;
use std::fs::File;

fn play_sound() {
    if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
        let sink = Sink::try_new(&stream_handle).unwrap();

        let file = BufReader::new(File::open("alarm_sound.mp3").unwrap());
        let source = Decoder::new(file).unwrap();

        sink.append(source);
        sink.sleep_until_end();
    } else {
        eprintln!("Audio output device not available.");
    }
}