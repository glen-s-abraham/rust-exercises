use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rusty_audio::Audio;
use std::{
    env,
    error::Error,
    io,
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use project_invaders::{
    frame::{self, new_frame, Drawable, Frame},
    invaders::Invaders,
    // level::Level,
    // menu::Menu,
    player::Player,
    render,
    // score::Score,
};

fn main() -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir()?;
    let mut audio = Audio::new();

    audio.add("explode", "C:/Users/iss211/Rust/project_invaders/src/explode.wav");
    audio.add("lose", "C:/Users/iss211/Rust/project_invaders/src/lose.wav");
    audio.add("move", "C:/Users/iss211/Rust/project_invaders/src/move.wav");
    audio.add("pew", "C:/Users/iss211/Rust/project_invaders/src/pew.wav");
    audio.add("startup", "C:/Users/iss211/Rust/project_invaders/src/startup.wav");
    audio.add("win", "C:/Users/iss211/Rust/project_invaders/src/win.wav");
    audio.play("startup");

     // Terminal setup
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    let mut player = Player::new();
    let mut instant = Instant::now();
    let mut invaders = Invaders::new();

    // Game loop
    'gameloop: loop {
        // Per-frame init
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();

        // Input
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Left=>player.move_left(),
                    KeyCode::Right=>player.move_right(),
                    KeyCode::Char(' ') | KeyCode::Enter=>{
                        if player.shoot(){
                            audio.play("pew");
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    _ => {}
                }
            }
        }
        // Updates
        player.update(delta);
        if invaders.update(delta){
            audio.play("move")
        }
        if player.detect_hits(&mut invaders){
            audio.play("explode");
        }

        // Draw and render section
        player.draw(&mut curr_frame);
        invaders.draw(&mut curr_frame);
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));

        // Win or Loose
        if invaders.all_killed(){
            audio.play("win");
            break 'gameloop;
        }

        if invaders.reached_bottom(){
            audio.play("lose");
            break 'gameloop;
        }

    }

    // Cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
