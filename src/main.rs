use std::error::Error;
use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use rand::Rng;

// Configuration constants
const TICK_RATE: Duration = Duration::from_millis(200);
const NEW_BLOCK_PROBABILITY: f64 = 0.1; // probability per column per tick

struct Block {
    x: u16,
    y: u16,
}

struct Game {
    player_x: u16,
    player_y: u16,
    blocks: Vec<Block>,
    score: u64,
    width: u16,
    height: u16,
}

impl Game {
    fn new(width: u16, height: u16) -> Self {
        Self {
            player_x: width / 2,
            player_y: height.saturating_sub(2),
            blocks: Vec::new(),
            score: 0,
            width,
            height,
        }
    }

    // Update game state on each tick
    fn update(&mut self) {
        let mut rng = rand::thread_rng();

        // Spawn new blocks along the top row
        for x in 0..self.width {
            if rng.gen_bool(NEW_BLOCK_PROBABILITY) {
                self.blocks.push(Block { x, y: 0 });
            }
        }

        // Move blocks down and remove those off-screen
        for block in &mut self.blocks {
            block.y += 1;
        }
        self.blocks.retain(|block| block.y < self.height);

        // Increase score as you survive
        self.score += 1;
    }

    // Check for collision between the player and any block
    fn check_collision(&self) -> bool {
        self.blocks.iter().any(|b| b.x == self.player_x && b.y == self.player_y)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Get terminal size and initialize game state
    let size = terminal.size()?;
    let mut game = Game::new(size.width, size.height);
    let mut last_tick = Instant::now();

    'game_loop: loop {
        // Draw the game frame
        terminal.draw(|f| {
            let area = f.size();

            // Build a vector of lines representing each row
            let mut lines = Vec::with_capacity(area.height as usize);
            for y in 0..area.height {
                let mut line = String::with_capacity(area.width as usize);
                for x in 0..area.width {
                    if y == game.player_y && x == game.player_x {
                        line.push('@');
                    } else if game.blocks.iter().any(|b| b.x == x && b.y == y) {
                        line.push('#');
                    } else {
                        line.push(' ');
                    }
                }
                lines.push(Line::from(Span::raw(line)));
            }

            // Create a Paragraph widget to display the game area with a border and score title
            let block = Block::default().borders(Borders::ALL).title(format!("Score: {}", game.score));
            let paragraph = Paragraph::new(lines)
                .block(block)
                .alignment(Alignment::Left);

            f.render_widget(paragraph, area);
        })?;

        // Input handling with non-blocking poll
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Left => {
                        if game.player_x > 0 {
                            game.player_x -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if game.player_x < game.width.saturating_sub(1) {
                            game.player_x += 1;
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Esc => break 'game_loop,
                    _ => {}
                }
            }
        }

        // Update game state based on tick rate
        if last_tick.elapsed() >= TICK_RATE {
            game.update();
            if game.check_collision() {
                break 'game_loop;
            }
            last_tick = Instant::now();
        }
    }

    // Clean up terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    println!("Game Over! Final Score: {}", game.score);

    Ok(())
}

