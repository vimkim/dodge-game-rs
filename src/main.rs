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
    layout::Alignment,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block as WidgetBlock, Borders, Paragraph},
    Terminal,
};
use rand::Rng;

// Configuration constants
const TICK_RATE: Duration = Duration::from_millis(200);
const NEW_BLOCK_PROBABILITY: f64 = 0.1; // probability per column per tick

#[derive(Clone)]
struct FallingBlock {
    x: u16,
    y: u16,
}

struct Game {
    player_x: u16,
    player_y: u16,
    blocks: Vec<FallingBlock>,
    score: u64,
    width: u16,  // playable width (inner area)
    height: u16, // playable height (inner area)
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

        // Spawn new blocks along the top row of the playable area
        for x in 0..self.width {
            if rng.gen_bool(NEW_BLOCK_PROBABILITY) {
                self.blocks.push(FallingBlock { x, y: 0 });
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
        self.blocks
            .iter()
            .any(|b| b.x == self.player_x && b.y == self.player_y)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Get terminal size and compute playable area (subtract border: 1 on each side)
    let outer_size = terminal.size()?;
    let playable_width = outer_size.width.saturating_sub(2);
    let playable_height = outer_size.height.saturating_sub(2);

    let mut game = Game::new(playable_width, playable_height);
    let mut last_tick = Instant::now();

    'game_loop: loop {
        // Draw the game frame
        terminal.draw(|f| {
            let outer_area = f.size();
            let block = WidgetBlock::default()
                .borders(Borders::ALL)
                .title(format!("Score: {}", game.score));
            let inner_area = block.inner(outer_area);

            // Build a vector of Spans representing each row in the playable area
            let mut lines = Vec::with_capacity(inner_area.height as usize);
            for y in 0..inner_area.height {
                let mut spans = Vec::with_capacity(inner_area.width as usize);
                for x in 0..inner_area.width {
                    if y == game.player_y && x == game.player_x {
                        // Player drawn with a contrasting style
                        spans.push(Span::styled(
                            "@",
                            Style::default().fg(Color::Black).bg(Color::Yellow),
                        ));
                    } else if game.blocks.iter().any(|b| b.x == x && b.y == y) {
                        spans.push(Span::raw("#"));
                    } else {
                        spans.push(Span::raw(" "));
                    }
                }
                lines.push(Spans::from(spans));
            }

            let paragraph = Paragraph::new(lines)
                .block(block)
                .alignment(Alignment::Left);
            f.render_widget(paragraph, outer_area);
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

