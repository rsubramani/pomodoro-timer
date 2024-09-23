use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    style::Stylize, // Added Stylize here for blue() and cyan()
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, stdout, BufReader},
    time::Duration,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Style, Color as TuiColor},  // Alias Tui's Color to TuiColor
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    text::{Span, Spans},
    Terminal,
};

use clap::{Parser, Subcommand};
use tokio::time::sleep;
use chrono::Local;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;


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

struct AppState {
    remaining_time: u64,
    total_time: u64,
}

impl AppState {
    fn progress(&self) -> f64 {
        (self.total_time - self.remaining_time) as f64 / self.total_time as f64
    }
}

// Session logging struct
#[derive(Serialize, Deserialize)]
struct SessionLog {
    date: String,
    work_sessions: u32,
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
    // Setup terminal
    enable_raw_mode().unwrap();
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let total_time = work_duration * 60;
    let app_state = AppState {
        remaining_time: total_time,
        total_time,
    };

    // Run the Pomodoro timer UI
    let _ = run_app(&mut terminal, app_state).await;

    // Restore terminal
    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
    terminal.show_cursor().unwrap();

    // After work session ends, log it and start the break
    log_session().expect("Failed to log session");
    println!("{}", "Starting Break".blue());
    play_sound();

    // Start break
    let total_break_time = break_duration * 60;
    let break_app_state = AppState {
        remaining_time: total_break_time,
        total_time: total_break_time,
    };
    let _ = run_app(&mut terminal, break_app_state).await;

    println!("{}", "Break over! Ready for the next session.".cyan());
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app_state: AppState) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                .split(f.size());

            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("Pomodoro Timer"))
                .gauge_style(
                    Style::default()
                        .fg(TuiColor::Green)
                        .bg(TuiColor::Black)
                        .add_modifier(tui::style::Modifier::BOLD),
                )
                .label(format!("{:.2}%", app_state.progress() * 100.0))
                .ratio(app_state.progress());

            f.render_widget(gauge, chunks[0]);
            render_timer(f, app_state.remaining_time, chunks[1]);
        })?;

        // Poll for user input (pause, resume, quit)
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let CEvent::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('p') => {
                        println!("Paused. Press 'r' to resume.");
                        loop {
                            if let CEvent::Key(resume_key_event) = event::read()? {
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
                    _ => {}
                }
            }
        }

        // Decrease remaining time and update the UI
        if app_state.remaining_time > 0 {
            app_state.remaining_time -= 1;
        } else {
            break;
        }

        sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

// Render the timer in mm:ss format
fn render_timer<B: Backend>(f: &mut tui::Frame<B>, remaining_time: u64, area: tui::layout::Rect) {
    let minutes = remaining_time / 60;
    let seconds = remaining_time % 60;

    let text = vec![
        Spans::from(vec![Span::raw("Time Remaining: ")]),
        Spans::from(vec![Span::styled(
            format!("{:02}:{:02}", minutes, seconds),
            Style::default().fg(TuiColor::Cyan),
        )]),
    ];

    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

// Log completed Pomodoro sessions
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

// Display session stats from log
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

// Play a sound when a session is completed
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

// Clean up the terminal
fn cleanup_terminal() {
    disable_raw_mode().unwrap();
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
}
