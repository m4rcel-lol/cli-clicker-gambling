use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{GameState, ASCENSION_THRESHOLD};
use crate::casino::{CasinoGame, CasinoState};

/// Top-level render entry point.
pub fn render(frame: &mut Frame, game: &GameState, casino: &CasinoState) {
    let area = frame.area();

    if game.casino_open {
        render_casino(frame, area, game, casino);
    } else {
        render_main(frame, area, game);
    }
}

// ─── Main game view ──────────────────────────────────────────────────────────

fn render_main(frame: &mut Frame, area: Rect, game: &GameState) {
    // Outer vertical split: top content | bottom log | bottom hotkeys
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(4),
            Constraint::Length(3),
        ])
        .split(area);

    // Top row: left panel | right panel
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(rows[0]);

    render_stats(frame, columns[0], game);
    if game.active_tab == 2 {
        render_upgrades(frame, columns[1], game);
    } else {
        render_buildings(frame, columns[1], game);
    }
    render_log(frame, rows[1], game);
    render_hotkeys(frame, rows[2], game);
}

/// Left panel: cookie count, CPS, click power, golden cookie, ASCII art.
fn render_stats(frame: &mut Frame, area: Rect, game: &GameState) {
    let cookies_fmt = format_number(game.cookies);
    let total_fmt = format_number(game.total_baked);
    let cps_fmt = format!("{:.1}", game.total_cps());
    let click_fmt = format!("{:.1}", game.click_power());

    let chip_info = if game.heavenly_chips > 0 {
        format!(
            "✨ {} Chip(s) (+{}% CPS)",
            game.heavenly_chips, game.heavenly_chips
        )
    } else {
        String::new()
    };

    let ascend_hint = if game.ascend_available {
        "🌟 [A]scend is unlocked!"
    } else {
        ""
    };

    let progress_str = if game.total_baked < ASCENSION_THRESHOLD {
        let pct = (game.total_baked / ASCENSION_THRESHOLD * 100.0).min(100.0);
        format!("Prestige: {:.1}%", pct)
    } else {
        "Prestige: READY".to_string()
    };

    let ascensions_str = if game.ascension_count > 0 {
        format!("Ascensions: {}", game.ascension_count)
    } else {
        String::new()
    };

    let cookie_art = [
        "       ______      ",
        "      /  🍪  \\    ",
        "     /  Clck  \\   ",
        "    / CLI Game \\  ",
        "   /______________\\",
    ];

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "🍪  THE COOKIE CLI",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Cookies: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                cookies_fmt,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("CPS:     ", Style::default().fg(Color::Cyan)),
            Span::styled(
                cps_fmt,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Click:   ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("+{} / click", click_fmt),
                Style::default().fg(Color::LightCyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("Baked:   ", Style::default().fg(Color::Cyan)),
            Span::styled(total_fmt, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Clicks:  ", Style::default().fg(Color::Cyan)),
            Span::styled(
                game.total_clicks.to_string(),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(Span::styled(
            progress_str,
            Style::default().fg(Color::Magenta),
        )),
        Line::from(""),
    ];

    for art_line in &cookie_art {
        lines.push(Line::from(Span::styled(
            *art_line,
            Style::default().fg(Color::Yellow),
        )));
    }
    lines.push(Line::from(""));

    if !chip_info.is_empty() {
        lines.push(Line::from(Span::styled(
            chip_info,
            Style::default().fg(Color::LightMagenta),
        )));
    }
    if !ascensions_str.is_empty() {
        lines.push(Line::from(Span::styled(
            ascensions_str,
            Style::default().fg(Color::LightMagenta),
        )));
    }
    if !ascend_hint.is_empty() {
        lines.push(Line::from(Span::styled(
            ascend_hint,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
    }

    // Golden cookie notification
    if game.golden_collect_window > 0 {
        let secs_left = (game.golden_collect_window as f32 * 0.25).ceil() as u32;
        lines.push(Line::from(Span::styled(
            format!(
                "✨ GOLDEN COOKIE! [C] ({:.0} cookies) {}s",
                game.golden_cookie_bonus, secs_left
            ),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::RAPID_BLINK),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Stats"))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

/// Right panel – Buildings tab.
fn render_buildings(frame: &mut Frame, area: Rect, game: &GameState) {
    let items: Vec<ListItem> = game
        .buildings
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let cost = b.next_cost();
            let affordable = game.cookies >= cost;
            let cost_str = format_number(cost);
            let mult = game.building_cps_multiplier(i);
            let effective_cps = b.base_cps * mult;
            let line_str = format!(
                "[{}] {:<8} (Owned: {:>3})  Cost: {}  CPS: +{:.1}{}",
                i + 1,
                b.name,
                b.owned,
                cost_str,
                effective_cps,
                if mult > 1.0 {
                    format!(" ({}x)", mult as u32)
                } else {
                    String::new()
                },
            );
            let style = if affordable {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            ListItem::new(line_str).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("▶ Buildings ◀  Upgrades  [T] Toggle Tab"),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(list, area);
}

/// Right panel – Upgrades tab.
fn render_upgrades(frame: &mut Frame, area: Rect, game: &GameState) {
    let building_names = ["Cursor", "Grandma", "Farm", "Mine"];

    let items: Vec<ListItem> = game
        .upgrades
        .iter()
        .enumerate()
        .map(|(i, u)| {
            let cost_str = format_number(u.cost);
            let bname = building_names
                .get(u.building_index)
                .copied()
                .unwrap_or("?");
            let status_tag = if u.purchased {
                "✔ OWNED"
            } else if game.cookies >= u.cost {
                "AFFORD"
            } else {
                "locked"
            };
            let header = format!(
                "[{}] {:<22} {:>10}  [{}]",
                i + 1,
                u.name,
                cost_str,
                status_tag
            );
            let detail = format!("     {} │ {}  2×", u.description, bname);

            let style = if u.purchased {
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM)
            } else if game.cookies >= u.cost {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let text = Text::from(vec![
                Line::from(header).style(style),
                Line::from(detail).style(Style::default().fg(Color::DarkGray)),
            ]);
            ListItem::new(text)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Buildings  ▶ Upgrades ◀  [T] Toggle Tab"),
    );
    frame.render_widget(list, area);
}

/// Log panel below the main area.
fn render_log(frame: &mut Frame, area: Rect, game: &GameState) {
    let items: Vec<ListItem> = game
        .log
        .iter()
        .map(|msg| {
            ListItem::new(format!("[Log]: {}", msg)).style(Style::default().fg(Color::Gray))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Activity Log"));
    frame.render_widget(list, area);
}

/// Hotkey bar at the very bottom.
fn render_hotkeys(frame: &mut Frame, area: Rect, game: &GameState) {
    let mut spans = vec![Span::styled(
        " [Space/Enter] Mine | [1-4] Buy | [T] Upgrades | [G] Casino | [S] Save | [Q] Quit",
        Style::default().fg(Color::White),
    )];

    if game.golden_collect_window > 0 {
        spans.push(Span::styled(
            " | [C] 🍪 COLLECT",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }

    if game.ascend_available {
        spans.push(Span::styled(
            " | [A] Ascend",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::ALL).title("Hotkeys"));
    frame.render_widget(paragraph, area);
}

// ─── Casino view ─────────────────────────────────────────────────────────────

fn render_casino(frame: &mut Frame, area: Rect, game: &GameState, casino: &CasinoState) {
    match &casino.active_game {
        None => render_casino_menu(frame, area, game, casino),
        Some(CasinoGame::SlotMachine) => render_slots(frame, area, game, casino),
        Some(CasinoGame::CoinFlip) => render_coinflip(frame, area, game, casino),
        Some(CasinoGame::DiceWager) => render_dice(frame, area, game, casino),
    }
}

fn render_casino_menu(frame: &mut Frame, area: Rect, game: &GameState, casino: &CasinoState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(area);

    let cookies_fmt = format_number(game.cookies);
    let net = casino.net_profit();
    let net_str = if net >= 0.0 {
        format!("+{:.0}", net)
    } else {
        format!("{:.0}", net)
    };
    let net_colour = if net >= 0.0 { Color::Green } else { Color::Red };

    let mut lines = vec![
        Line::from(Span::styled(
            "🎰  WELCOME TO THE COOKIE CASINO",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Your balance: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{} cookies", cookies_fmt),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Choose your game:",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!(
                "  [S] 🎰  Slot Machine  (Bet: {} cookies)",
                crate::casino::SLOT_BET as u64
            ),
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            "       Payouts: 3×🍒=5x | 3×🍋=3x | 3×🔔=7x | 3×💎=25x | 3×7️⃣=100x",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "  [F] 🪙  Coin Flip     (Custom bet, 2x or bust)",
            Style::default().fg(Color::Blue),
        )),
        Line::from(Span::styled(
            "  [D] 🎲  Dice Wager    (Guess 1-6, win 5x)",
            Style::default().fg(Color::Magenta),
        )),
        Line::from(""),
    ];

    // Lifetime casino stats
    if casino.total_spins > 0 || casino.total_wagered > 0.0 {
        lines.push(Line::from(Span::styled(
            "── Lifetime Stats ──────────────────────────",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(vec![
            Span::styled("  Spins: ", Style::default().fg(Color::Gray)),
            Span::styled(
                casino.total_spins.to_string(),
                Style::default().fg(Color::White),
            ),
            Span::styled("   Wagered: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format_number(casino.total_wagered),
                Style::default().fg(Color::White),
            ),
            Span::styled("   Net: ", Style::default().fg(Color::Gray)),
            Span::styled(net_str, Style::default().fg(net_colour)),
        ]));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        "⚠  Remember: gambling is for cookies, not real life!",
        Style::default().fg(Color::Red),
    )));

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("🎰 Casino"))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, rows[0]);

    let hotkeys = Paragraph::new(Line::from(Span::styled(
        " [S] Slots | [F] Coin Flip | [D] Dice | [G] Exit Casino | [Q] Quit",
        Style::default().fg(Color::White),
    )))
    .block(Block::default().borders(Borders::ALL).title("Hotkeys"));
    frame.render_widget(hotkeys, rows[1]);
}

fn render_slots(frame: &mut Frame, area: Rect, game: &GameState, casino: &CasinoState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(area);

    let balance = format_number(game.cookies);
    let mut lines = vec![
        Line::from(Span::styled(
            "🎰  SLOT MACHINE",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("Balance: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{} cookies", balance),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Bet: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{:.0} cookies", crate::casino::SLOT_BET),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
    ];

    if let Some(ref result) = casino.last_slot {
        let reel_line = format!(
            "  [ {} | {} | {} ]",
            result.reels[0].label(),
            result.reels[1].label(),
            result.reels[2].label()
        );
        lines.push(Line::from(Span::styled(
            reel_line,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        let (outcome_str, colour) = if result.payout > 0.0 {
            (
                format!("🎉 WIN! +{:.0} cookies!", result.payout),
                Color::Green,
            )
        } else if result.payout == 0.0 {
            ("😐 No payout. Try again!".to_string(), Color::Gray)
        } else {
            (
                format!("💸 Lost {:.0} cookies.", -result.payout),
                Color::Red,
            )
        };
        lines.push(Line::from(Span::styled(
            outcome_str,
            Style::default().fg(colour),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "  [ ? | ? | ? ]",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Press [S] to spin!",
            Style::default().fg(Color::Gray),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Payouts: 3x🍒=5x | 3x🍋=3x | 3x🔔=7x | 3x💎=25x | 3x7️⃣=100x bet",
        Style::default().fg(Color::Gray),
    )));
    if casino.total_spins > 0 {
        lines.push(Line::from(Span::styled(
            format!("Total spins: {}", casino.total_spins),
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🎰 Slot Machine"),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, rows[0]);

    let hotkeys = Paragraph::new(Line::from(Span::styled(
        " [S] Spin | [G] Casino Menu | [Q] Quit",
        Style::default().fg(Color::White),
    )))
    .block(Block::default().borders(Borders::ALL).title("Hotkeys"));
    frame.render_widget(hotkeys, rows[1]);
}

fn render_coinflip(frame: &mut Frame, area: Rect, game: &GameState, casino: &CasinoState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(area);

    let balance = format_number(game.cookies);
    let wager_display = if casino.wager_input.is_empty() {
        "enter wager...".to_string()
    } else {
        casino.wager_input.clone()
    };

    let mut lines = vec![
        Line::from(Span::styled(
            "🪙  COIN FLIP",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("Balance: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{} cookies", balance),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Wager: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                wager_display,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    if let Some(ref result) = casino.last_coin {
        let (outcome_str, colour) = if result.won {
            (
                format!("🎉 HEADS! You won {:.0} cookies!", result.net),
                Color::Green,
            )
        } else {
            (
                format!("💸 TAILS! You lost {:.0} cookies.", -result.net),
                Color::Red,
            )
        };
        lines.push(Line::from(Span::styled(
            outcome_str,
            Style::default().fg(colour),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("🪙 Coin Flip"))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, rows[0]);

    let hotkeys = Paragraph::new(Line::from(Span::styled(
        " [0-9] Enter wager | [Enter/F] Flip | [Backspace] Erase | [G] Casino Menu",
        Style::default().fg(Color::White),
    )))
    .block(Block::default().borders(Borders::ALL).title("Hotkeys"));
    frame.render_widget(hotkeys, rows[1]);
}

fn render_dice(frame: &mut Frame, area: Rect, game: &GameState, casino: &CasinoState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(area);

    let balance = format_number(game.cookies);
    let wager_display = if casino.wager_input.is_empty() {
        "enter wager...".to_string()
    } else {
        casino.wager_input.clone()
    };
    let guess_display = if casino.dice_guess == 0 {
        "?".to_string()
    } else {
        casino.dice_guess.to_string()
    };

    let mut lines = vec![
        Line::from(Span::styled(
            "🎲  DICE WAGER",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("Balance: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{} cookies", balance),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Wager:  ", Style::default().fg(Color::Cyan)),
            Span::styled(
                wager_display,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Guess (1-6): ", Style::default().fg(Color::Cyan)),
            Span::styled(
                guess_display,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    if let Some(ref result) = casino.last_dice {
        let die_face = die_ascii(result.rolled);
        lines.push(Line::from(Span::styled(
            format!("Rolled: {} (you guessed {})", die_face, result.guessed),
            Style::default().fg(Color::White),
        )));
        let (outcome_str, colour) = if result.won {
            (
                format!("🎉 Correct! +{:.0} cookies!", result.net),
                Color::Green,
            )
        } else {
            (
                format!("💸 Wrong! Lost {:.0} cookies.", -result.net),
                Color::Red,
            )
        };
        lines.push(Line::from(Span::styled(
            outcome_str,
            Style::default().fg(colour),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Win chance: 1/6 | Payout: 5x bet (net +4x)",
        Style::default().fg(Color::Gray),
    )));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🎲 Dice Wager"),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, rows[0]);

    let step = if casino.entering_wager {
        "[0-9] Enter wager | [Enter] Confirm wager | [Backspace] Erase"
    } else if casino.entering_dice_guess {
        "[1-6] Enter guess | [Enter/D] Roll | [Backspace] Erase"
    } else {
        "[0-9] Enter wager | [Enter] Confirm | [D] Roll | [G] Casino Menu"
    };
    let hotkeys = Paragraph::new(Line::from(Span::styled(
        format!(" {}", step),
        Style::default().fg(Color::White),
    )))
    .block(Block::default().borders(Borders::ALL).title("Hotkeys"));
    frame.render_widget(hotkeys, rows[1]);
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn format_number(n: f64) -> String {
    if n >= 1_000_000_000.0 {
        format!("{:.2}B", n / 1_000_000_000.0)
    } else if n >= 1_000_000.0 {
        format!("{:.2}M", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        let int_part = n as u64;
        let thousands = int_part / 1_000;
        let rest = int_part % 1_000;
        format!("{},{:03}", thousands, rest)
    } else {
        format!("{:.0}", n)
    }
}

fn die_ascii(n: u8) -> &'static str {
    match n {
        1 => "⚀",
        2 => "⚁",
        3 => "⚂",
        4 => "⚃",
        5 => "⚄",
        6 => "⚅",
        _ => "?",
    }
}
