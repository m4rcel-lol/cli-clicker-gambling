mod app;
mod casino;
mod ui;

use std::{
    env,
    fs,
    io,
    path::PathBuf,
    sync::mpsc,
    thread,
    time::Duration,
};

use crossterm::{
    event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::SeedableRng;
use ratatui::{Terminal, backend::CrosstermBackend};

use app::GameState;
use casino::{CasinoGame, CasinoState};

// ─── Event types sent over the mpsc channel ───────────────────────────────────

enum AppEvent {
    Input(KeyEvent),
    Tick,
}

// ─── Save / Load ─────────────────────────────────────────────────────────────

const SAVE_FILE_NAME: &str = "save.json";
const APP_DIR: &str = "cookie_clicker";
/// Number of ticks between auto-saves (250 ms/tick * 240 = 60 s).
const AUTOSAVE_TICKS: u32 = 240;

fn save_path() -> PathBuf {
    let base = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config")
        });
    base.join(APP_DIR).join(SAVE_FILE_NAME)
}

fn save_game(game: &GameState) {
    if let Ok(json) = serde_json::to_string_pretty(game) {
        let path = save_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&path, json);
    }
}

fn load_game() -> GameState {
    let path = save_path();
    if path.exists() {
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(gs) = serde_json::from_str::<GameState>(&contents) {
                return gs;
            }
        }
    }
    GameState::default()
}

// ─── Entry point ──────────────────────────────────────────────────────────────

fn main() -> io::Result<()> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

// ─── Main application loop ────────────────────────────────────────────────────

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut game = load_game();
    let mut casino = CasinoState::default();
    let mut rng = rand::rngs::SmallRng::from_entropy();

    // mpsc channel for events
    let (tx, rx) = mpsc::channel::<AppEvent>();

    // Input thread: forwards crossterm key events
    let tx_input = tx.clone();
    thread::spawn(move || loop {
        if event::poll(Duration::from_millis(50)).unwrap_or(false) {
            if let Ok(CrosstermEvent::Key(key)) = event::read() {
                if tx_input.send(AppEvent::Input(key)).is_err() {
                    break;
                }
            }
        }
    });

    // Tick thread: sends a Tick every 250 ms
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(250));
        if tx.send(AppEvent::Tick).is_err() {
            break;
        }
    });

    loop {
        // Draw
        terminal.draw(|f| ui::render(f, &game, &casino))?;

        // Handle next event
        match rx.recv() {
            Ok(AppEvent::Tick) => {
                game.tick(0.25); // 250 ms tick
                if game.ticks_since_save >= AUTOSAVE_TICKS {
                    save_game(&game);
                    game.ticks_since_save = 0;
                }
            }
            Ok(AppEvent::Input(key)) => {
                if handle_input(key, &mut game, &mut casino, &mut rng) {
                    // Quit requested
                    save_game(&game);
                    break;
                }
            }
            Err(_) => break,
        }
    }

    Ok(())
}

// ─── Input handler ────────────────────────────────────────────────────────────

/// Returns `true` if the application should quit.
fn handle_input(
    key: KeyEvent,
    game: &mut GameState,
    casino: &mut CasinoState,
    rng: &mut rand::rngs::SmallRng,
) -> bool {
    // Ctrl-C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return true;
    }

    if game.casino_open {
        return handle_casino_input(key, game, casino, rng);
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return true,
        KeyCode::Char(' ') | KeyCode::Enter => game.mine_cookie(),
        KeyCode::Char('1') => {
            if !game.buy_building(0) {
                game.push_log("Not enough cookies for Cursor!".to_string());
            }
        }
        KeyCode::Char('2') => {
            if !game.buy_building(1) {
                game.push_log("Not enough cookies for Grandma!".to_string());
            }
        }
        KeyCode::Char('3') => {
            if !game.buy_building(2) {
                game.push_log("Not enough cookies for Farm!".to_string());
            }
        }
        KeyCode::Char('4') => {
            if !game.buy_building(3) {
                game.push_log("Not enough cookies for Mine!".to_string());
            }
        }
        KeyCode::Char('g') | KeyCode::Char('G') => {
            game.casino_open = !game.casino_open;
            casino.active_game = None;
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            save_game(game);
            game.push_log("Game saved!".to_string());
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            if game.ascend_available {
                game.ascend();
            }
        }
        _ => {}
    }
    false
}

fn handle_casino_input(
    key: KeyEvent,
    game: &mut GameState,
    casino: &mut CasinoState,
    rng: &mut rand::rngs::SmallRng,
) -> bool {
    match &casino.active_game.clone() {
        None => handle_casino_menu_input(key, game, casino),
        Some(CasinoGame::SlotMachine) => handle_slots_input(key, game, casino, rng),
        Some(CasinoGame::CoinFlip) => handle_coinflip_input(key, game, casino, rng),
        Some(CasinoGame::DiceWager) => handle_dice_input(key, game, casino, rng),
    }
}

fn handle_casino_menu_input(
    key: KeyEvent,
    game: &mut GameState,
    casino: &mut CasinoState,
) -> bool {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return true,
        KeyCode::Char('g') | KeyCode::Char('G') => {
            game.casino_open = false;
            casino.active_game = None;
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            casino.active_game = Some(CasinoGame::SlotMachine);
            casino.last_slot = None;
        }
        KeyCode::Char('f') | KeyCode::Char('F') => {
            casino.active_game = Some(CasinoGame::CoinFlip);
            casino.wager_input.clear();
            casino.last_coin = None;
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            casino.active_game = Some(CasinoGame::DiceWager);
            casino.wager_input.clear();
            casino.dice_guess = 0;
            casino.entering_wager = true;
            casino.entering_dice_guess = false;
            casino.last_dice = None;
        }
        _ => {}
    }
    false
}

fn handle_slots_input(
    key: KeyEvent,
    game: &mut GameState,
    casino: &mut CasinoState,
    rng: &mut rand::rngs::SmallRng,
) -> bool {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return true,
        KeyCode::Char('g') | KeyCode::Char('G') => {
            casino.active_game = None;
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            if game.spend_cookies(casino::SLOT_BET) {
                let gross = casino.spin_slots(rng);
                if gross > 0.0 {
                    game.add_cookies(gross); // gross payout (includes returned bet)
                    let net = gross - casino::SLOT_BET;
                    if net > 0.0 {
                        game.push_log(format!("Slots: WIN! +{:.0} cookies net!", net));
                    } else {
                        game.push_log(format!("Slots: Consolation! Recovered {:.0} cookies.", gross));
                    }
                } else {
                    // Full loss – cookies already spent
                    game.push_log(format!("Slots: Lost {} cookies.", casino::SLOT_BET as u64));
                }
            } else {
                game.push_log(format!(
                    "Need {:.0} cookies to spin!",
                    casino::SLOT_BET
                ));
            }
        }
        _ => {}
    }
    false
}

fn handle_coinflip_input(
    key: KeyEvent,
    game: &mut GameState,
    casino: &mut CasinoState,
    rng: &mut rand::rngs::SmallRng,
) -> bool {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return true,
        KeyCode::Char('g') | KeyCode::Char('G') => {
            casino.active_game = None;
            casino.wager_input.clear();
        }
        KeyCode::Char(c) if c.is_ascii_digit() => {
            if casino.wager_input.len() < 12 {
                casino.wager_input.push(c);
            }
        }
        KeyCode::Backspace => {
            casino.wager_input.pop();
        }
        KeyCode::Enter | KeyCode::Char('f') | KeyCode::Char('F') => {
            if let Some(wager) = casino.parsed_wager() {
                if wager < casino::COINFLIP_MIN_BET {
                    game.push_log(format!(
                        "Min coin flip bet is {:.0} cookies.",
                        casino::COINFLIP_MIN_BET
                    ));
                } else if game.spend_cookies(wager) {
                    let net = casino.flip_coin(wager, rng);
                    if net > 0.0 {
                        game.add_cookies(wager * 2.0); // return bet + winnings
                        game.push_log(format!("Coin Flip: HEADS! Won {:.0} cookies!", net));
                    } else {
                        game.push_log(format!("Coin Flip: TAILS! Lost {:.0} cookies.", wager));
                    }
                    casino.wager_input.clear();
                } else {
                    game.push_log("Not enough cookies for that bet!".to_string());
                }
            }
        }
        _ => {}
    }
    false
}

fn handle_dice_input(
    key: KeyEvent,
    game: &mut GameState,
    casino: &mut CasinoState,
    rng: &mut rand::rngs::SmallRng,
) -> bool {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return true,
        KeyCode::Char('g') | KeyCode::Char('G') => {
            casino.active_game = None;
            casino.wager_input.clear();
            casino.dice_guess = 0;
            casino.entering_wager = false;
            casino.entering_dice_guess = false;
        }
        KeyCode::Backspace => {
            if casino.entering_wager {
                casino.wager_input.pop();
            } else if casino.entering_dice_guess {
                casino.dice_guess = 0;
            }
        }
        KeyCode::Char(c) if c.is_ascii_digit() => {
            if casino.entering_wager {
                if casino.wager_input.len() < 12 {
                    casino.wager_input.push(c);
                }
            } else if casino.entering_dice_guess {
                if let Some(d) = c.to_digit(10) {
                    let d = d as u8;
                    if d >= 1 && d <= 6 {
                        casino.dice_guess = d;
                    }
                }
            }
        }
        KeyCode::Enter => {
            if casino.entering_wager {
                // Confirm wager, move to entering guess
                if casino.parsed_wager().is_some() {
                    casino.entering_wager = false;
                    casino.entering_dice_guess = true;
                }
            } else if casino.entering_dice_guess {
                // Roll
                try_dice_roll(game, casino, rng);
            } else {
                // Start entering wager
                casino.entering_wager = true;
                casino.wager_input.clear();
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if !casino.entering_wager {
                try_dice_roll(game, casino, rng);
            }
        }
        _ => {}
    }
    false
}

fn try_dice_roll(
    game: &mut GameState,
    casino: &mut CasinoState,
    rng: &mut rand::rngs::SmallRng,
) {
    let wager = match casino.parsed_wager() {
        Some(w) => w,
        None => {
            game.push_log("Enter a valid wager first.".to_string());
            return;
        }
    };
    let guess = casino.dice_guess;
    if guess < 1 || guess > 6 {
        game.push_log("Enter a guess between 1 and 6.".to_string());
        return;
    }
    if wager < casino::DICE_MIN_BET {
        game.push_log(format!(
            "Min dice bet is {:.0} cookies.",
            casino::DICE_MIN_BET
        ));
        return;
    }
    if game.spend_cookies(wager) {
        let net = casino.roll_dice(wager, guess, rng);
        if net > 0.0 {
            game.add_cookies(wager + net); // return bet + 4x profit
            game.push_log(format!("Dice: Correct! Won {:.0} cookies!", net));
        } else {
            game.push_log(format!("Dice: Wrong! Lost {:.0} cookies.", wager));
        }
        casino.wager_input.clear();
        casino.dice_guess = 0;
        casino.entering_wager = true;
        casino.entering_dice_guess = false;
    } else {
        game.push_log("Not enough cookies for that bet!".to_string());
    }
}

