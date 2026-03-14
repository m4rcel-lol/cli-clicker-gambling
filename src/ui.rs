use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{GameState, ASCENSION_THRESHOLD, UPGRADES_PER_PAGE};
use crate::casino::{CasinoGame, CasinoState};
use crate::chat::ChatState;

// ═══════════════════════════════════════════════════════════════════════════════
//  btop-inspired colour palette
// ═══════════════════════════════════════════════════════════════════════════════

const CYAN_BRIGHT: Color = Color::Rgb(0x00, 0xff, 0xff);
const CYAN_MID: Color = Color::Rgb(0x00, 0xcc, 0xff);
const CYAN_DIM: Color = Color::Rgb(0x00, 0x88, 0xff);
const GREEN_BRIGHT: Color = Color::Rgb(0x44, 0xff, 0x44);
const GREEN_MID: Color = Color::Rgb(0x00, 0xff, 0x88);
const ORANGE: Color = Color::Rgb(0xff, 0xaa, 0x00);
const ORANGE_DIM: Color = Color::Rgb(0xff, 0x66, 0x00);
const RED_BRIGHT: Color = Color::Rgb(0xff, 0x44, 0x44);
const RED_DIM: Color = Color::Rgb(0xff, 0x00, 0x00);
const GOLD: Color = Color::Rgb(0xff, 0xd7, 0x00);
const GOLD_DIM: Color = Color::Rgb(0xff, 0xaa, 0x00);
const MAGENTA_BRIGHT: Color = Color::Rgb(0xff, 0x44, 0xff);
const MAGENTA_DIM: Color = Color::Rgb(0xcc, 0x44, 0xcc);
const GRAY_LIGHT: Color = Color::Rgb(0x88, 0x88, 0x88);
const GRAY_MID: Color = Color::Rgb(0x66, 0x66, 0x66);
const GRAY_DIM: Color = Color::Rgb(0x44, 0x44, 0x44);
const WHITE: Color = Color::Rgb(0xee, 0xee, 0xee);
const NEON_PINK: Color = Color::Rgb(0xff, 0x00, 0x88);
const NEON_GREEN: Color = Color::Rgb(0x00, 0xff, 0x66);
const NEON_PURPLE: Color = Color::Rgb(0xaa, 0x44, 0xff);

const BUILDING_ICONS: [&str; 8] = [
    "\u{1f5b1}\u{fe0f}", // 🖱️
    "\u{1f475}",          // 👵
    "\u{1f33e}",          // 🌾
    "\u{26cf}\u{fe0f}",   // ⛏️
    "\u{1f3ed}",          // 🏭
    "\u{1f3e6}",          // 🏦
    "\u{1f3db}\u{fe0f}",  // 🏛️
    "\u{1f9d9}",          // 🧙
];

const CLOSE_TO_AFFORD_RATIO: f64 = 0.5;
const PROGRESS_BAR_WIDTH: usize = 20;

// ═══════════════════════════════════════════════════════════════════════════════
//  Public entry point
// ═══════════════════════════════════════════════════════════════════════════════

/// Top-level render entry point.
pub fn render(frame: &mut Frame, game: &GameState, casino: &CasinoState, chat: &ChatState) {
    let area = frame.area();
    if chat.chat_open {
        render_chat(frame, area, chat);
    } else if game.casino_open {
        render_casino(frame, area, game, casino);
    } else {
        render_main(frame, area, game);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Main game view
// ═══════════════════════════════════════════════════════════════════════════════

fn render_main(frame: &mut Frame, area: Rect, game: &GameState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(6),
            Constraint::Length(3),
        ])
        .split(area);

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

// ─── Stats panel ─────────────────────────────────────────────────────────────

fn render_stats(frame: &mut Frame, area: Rect, game: &GameState) {
    let cookies_fmt = format_number(game.cookies);
    let total_fmt = format_number(game.total_baked);
    let cps = game.total_cps();
    let cps_fmt = format_number(cps);
    let click_fmt = format!("{:.1}", game.click_power());

    // Cookie ASCII art with animation frames (chocolate chip positions shift)
    let frame_idx = (game.animation_tick / 4) as usize % 4;
    let cookie_frames: [[&str; 7]; 4] = [
        [
            r"       _.---._       ",
            r"     .'  o    '.     ",
            r"    /  o    o   \    ",
            r"   |    o     o  |   ",
            r"    \  o    o   /    ",
            r"     '._  o  _.'     ",
            r"        '---'        ",
        ],
        [
            r"       _.---._       ",
            r"     .'    o   '.    ",
            r"    / o      o   \   ",
            r"   |   o   o      |  ",
            r"    \    o     o /   ",
            r"     '._ o   _.'     ",
            r"        '---'        ",
        ],
        [
            r"       _.---._       ",
            r"     .'   o    '.    ",
            r"    /    o   o   \   ",
            r"   | o      o    |   ",
            r"    \   o  o    /    ",
            r"     '._   o _.'     ",
            r"        '---'        ",
        ],
        [
            r"       _.---._       ",
            r"     .' o      '.    ",
            r"    /     o  o   \   ",
            r"   |  o    o     |   ",
            r"    \ o      o  /    ",
            r"     '._o    _.'     ",
            r"        '---'        ",
        ],
    ];
    let cookie_art = &cookie_frames[frame_idx];
    let art_color = match frame_idx {
        0 => GOLD,
        1 => GOLD_DIM,
        2 => ORANGE,
        _ => GOLD,
    };

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled(
                " (::) COOKIE CLICKER ",
                Style::default()
                    .fg(GOLD)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Cookies  ", Style::default().fg(CYAN_MID)),
            Span::styled(
                &cookies_fmt,
                Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(" CPS      ", Style::default().fg(CYAN_MID)),
            Span::styled(
                format!("{}/s", cps_fmt),
                Style::default()
                    .fg(GREEN_BRIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Click    ", Style::default().fg(CYAN_MID)),
            Span::styled(
                format!("+{}", click_fmt),
                Style::default().fg(CYAN_BRIGHT),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Baked    ", Style::default().fg(CYAN_DIM)),
            Span::styled(&total_fmt, Style::default().fg(GRAY_LIGHT)),
        ]),
        Line::from(vec![
            Span::styled(" Clicks   ", Style::default().fg(CYAN_DIM)),
            Span::styled(
                format_number(game.total_clicks as f64),
                Style::default().fg(GRAY_LIGHT),
            ),
        ]),
        Line::from(""),
    ];

    // Cookie art
    for art_line in cookie_art.iter() {
        lines.push(Line::from(Span::styled(
            *art_line,
            Style::default().fg(art_color),
        )));
    }

    // Click animation burst
    if game.click_animation > 0 {
        let power = format!("+{}", click_fmt);
        let burst = match game.click_animation {
            3 => format!(" \u{2726} {}! \u{2726} ", power),
            2 => format!("  \u{2727} {} \u{2727}  ", power),
            _ => format!("   \u{00b7} {} \u{00b7}   ", power),
        };
        lines.push(Line::from(Span::styled(
            burst,
            Style::default()
                .fg(GOLD)
                .add_modifier(Modifier::BOLD),
        )));
    } else {
        lines.push(Line::from(""));
    }

    // Prestige progress
    let progress_str = if game.total_baked < ASCENSION_THRESHOLD {
        let pct = (game.total_baked / ASCENSION_THRESHOLD * 100.0).min(100.0);
        let filled = ((pct / (100.0 / PROGRESS_BAR_WIDTH as f64)) as usize).min(PROGRESS_BAR_WIDTH);
        let bar: String = "▓".repeat(filled) + &"░".repeat(PROGRESS_BAR_WIDTH - filled);
        format!(" Prestige [{}] {:.1}%", bar, pct)
    } else {
        " Prestige [▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓] READY!".to_string()
    };
    lines.push(Line::from(Span::styled(
        &progress_str,
        Style::default().fg(MAGENTA_DIM),
    )));

    // Heavenly chips
    if game.heavenly_chips > 0 {
        lines.push(Line::from(vec![
            Span::styled(" ✨ Heavenly: ", Style::default().fg(MAGENTA_BRIGHT)),
            Span::styled(
                format!("{}", game.heavenly_chips),
                Style::default()
                    .fg(WHITE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" (+{}%)", game.heavenly_chips),
                Style::default().fg(MAGENTA_DIM),
            ),
        ]));
    }
    if game.ascension_count > 0 {
        lines.push(Line::from(Span::styled(
            format!(" Ascensions: {}", game.ascension_count),
            Style::default().fg(MAGENTA_DIM),
        )));
    }
    if game.ascend_available {
        let blink = if game.animation_tick / 2 % 2 == 0 {
            "🌟 [A]SCEND READY! 🌟"
        } else {
            "   [A]SCEND READY!   "
        };
        lines.push(Line::from(Span::styled(
            format!(" {}", blink),
            Style::default()
                .fg(GOLD)
                .add_modifier(Modifier::BOLD),
        )));
    }

    // Frenzy indicator
    if game.frenzy_ticks > 0 {
        let secs = (game.frenzy_ticks as f32 * 0.25).ceil() as u32;
        let phase = game.animation_tick % 6;
        let frenzy_color = match phase {
            0 => RED_BRIGHT,
            1 => ORANGE,
            2 => GOLD,
            3 => GREEN_BRIGHT,
            4 => CYAN_BRIGHT,
            _ => MAGENTA_BRIGHT,
        };
        lines.push(Line::from(Span::styled(
            format!(
                " \u{1f525} FRENZY x{:.0} \u{1f525} {}s",
                game.frenzy_multiplier, secs
            ),
            Style::default()
                .fg(frenzy_color)
                .add_modifier(Modifier::BOLD),
        )));
    }

    // Golden cookie
    if game.golden_collect_window > 0 {
        let secs_left = (game.golden_collect_window as f32 * 0.25).ceil() as u32;
        let sparkle_phase = game.animation_tick % 4;
        let sparkle = match sparkle_phase {
            0 => "✨ ★ GOLDEN COOKIE! ★ ✨",
            1 => "★ ✨ GOLDEN COOKIE! ✨ ★",
            2 => "✨ ✦ GOLDEN COOKIE! ✦ ✨",
            _ => "✦ ✨ GOLDEN COOKIE! ✨ ✦",
        };
        let bonus_text = if game.golden_cookie_bonus < 0.0 {
            "FRENZY".to_string()
        } else {
            format!("+{:.0}", game.golden_cookie_bonus)
        };
        lines.push(Line::from(Span::styled(
            format!(" {} [C] {} {}s", sparkle, bonus_text, secs_left),
            Style::default()
                .fg(GOLD)
                .add_modifier(Modifier::BOLD),
        )));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CYAN_DIM))
        .title(Span::styled(
            "╣ Stats ╠",
            Style::default()
                .fg(CYAN_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ));
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

// ─── Buildings panel ─────────────────────────────────────────────────────────

fn render_buildings(frame: &mut Frame, area: Rect, game: &GameState) {
    let items: Vec<ListItem> = game
        .buildings
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let cost = b.next_cost();
            let affordable = game.cookies >= cost;
            let close_to_afford = !affordable && game.cookies >= cost * CLOSE_TO_AFFORD_RATIO;
            let cost_str = format_number(cost);
            let mult = game.building_cps_multiplier(i);
            let effective_cps = b.base_cps * mult;
            let icon = BUILDING_ICONS.get(i).copied().unwrap_or("?");

            // Progress bar toward next purchase
            let progress = if affordable {
                1.0
            } else {
                (game.cookies / cost).min(1.0)
            };
            let bar_width = 8;
            let filled = (progress * bar_width as f64) as usize;
            let bar: String = "\u{2593}".repeat(filled)
                + &"\u{2591}".repeat(bar_width - filled);

            let mult_str = if mult > 1.0 {
                format!(" ({}x)", mult as u32)
            } else {
                String::new()
            };

            let line = vec![
                Span::styled(
                    format!(" [{}] ", i + 1),
                    Style::default().fg(CYAN_BRIGHT).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} ", icon),
                    Style::default().fg(if affordable { GREEN_BRIGHT } else { GRAY_MID }),
                ),
                Span::styled(
                    format!("{:<12}", b.name),
                    Style::default().fg(if affordable {
                        GREEN_BRIGHT
                    } else if close_to_afford {
                        ORANGE
                    } else {
                        GRAY_MID
                    }),
                ),
                Span::styled(
                    format!("x{:<3} ", b.owned),
                    Style::default()
                        .fg(WHITE)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:<9}", cost_str),
                    Style::default().fg(if affordable {
                        GREEN_MID
                    } else if close_to_afford {
                        ORANGE_DIM
                    } else {
                        GRAY_DIM
                    }),
                ),
                Span::styled(
                    format!("+{:.1}/s", effective_cps),
                    Style::default().fg(CYAN_MID),
                ),
                Span::styled(
                    mult_str,
                    Style::default().fg(MAGENTA_DIM),
                ),
                Span::styled(
                    format!(" {}", bar),
                    Style::default().fg(if affordable { GREEN_BRIGHT } else { GRAY_DIM }),
                ),
            ];

            ListItem::new(Line::from(line))
        })
        .collect();

    let tab_title = Line::from(vec![
        Span::styled(" ▶ ", Style::default().fg(CYAN_BRIGHT)),
        Span::styled(
            "Buildings",
            Style::default()
                .fg(CYAN_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ◀ ", Style::default().fg(CYAN_BRIGHT)),
        Span::styled("  Upgrades  ", Style::default().fg(GRAY_MID)),
        Span::styled("[T]", Style::default().fg(GRAY_LIGHT)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CYAN_DIM))
        .title(tab_title);
    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

// ─── Upgrades panel ──────────────────────────────────────────────────────────

fn render_upgrades(frame: &mut Frame, area: Rect, game: &GameState) {
    let page = game.upgrade_page as usize;
    let start = page * UPGRADES_PER_PAGE;
    let end = (start + UPGRADES_PER_PAGE).min(game.upgrades.len());
    let total_pages = (game.upgrades.len() + UPGRADES_PER_PAGE - 1) / UPGRADES_PER_PAGE;

    let items: Vec<ListItem> = game.upgrades[start..end]
        .iter()
        .enumerate()
        .map(|(i, u)| {
            let display_key = i + 1;
            let cost_str = format_number(u.cost);
            let bname = game
                .buildings
                .get(u.building_index)
                .map(|b| b.name.as_str())
                .unwrap_or("?");

            let (status_tag, status_color) = if u.purchased {
                ("\u{2714} OWNED", GRAY_MID)
            } else if game.cookies >= u.cost {
                ("\u{1f4b0} AFFORD", GREEN_BRIGHT)
            } else {
                ("\u{1f512} locked", GRAY_DIM)
            };

            let name_color = if u.purchased {
                GRAY_DIM
            } else if game.cookies >= u.cost {
                GREEN_BRIGHT
            } else {
                GRAY_LIGHT
            };

            let header = Line::from(vec![
                Span::styled(
                    format!(" [{}] ", display_key),
                    Style::default().fg(CYAN_BRIGHT).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:<22}", u.name),
                    Style::default().fg(name_color),
                ),
                Span::styled(
                    format!("{:>10}", cost_str),
                    Style::default().fg(ORANGE),
                ),
                Span::styled(
                    format!("  {}", status_tag),
                    Style::default().fg(status_color),
                ),
            ]);
            let detail = Line::from(vec![
                Span::styled("      ", Style::default()),
                Span::styled(
                    &u.description,
                    Style::default().fg(GRAY_LIGHT),
                ),
                Span::styled(
                    format!(" │ {} 2\u{00d7}", bname),
                    Style::default().fg(GRAY_MID),
                ),
            ]);

            ListItem::new(Text::from(vec![header, detail]))
        })
        .collect();

    let tab_title = Line::from(vec![
        Span::styled("  Buildings  ", Style::default().fg(GRAY_MID)),
        Span::styled(" ▶ ", Style::default().fg(CYAN_BRIGHT)),
        Span::styled(
            "Upgrades",
            Style::default()
                .fg(CYAN_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ◀ ", Style::default().fg(CYAN_BRIGHT)),
        Span::styled("[T]", Style::default().fg(GRAY_LIGHT)),
        Span::styled(
            format!("  Pg {}/{} ", page + 1, total_pages),
            Style::default().fg(GRAY_MID),
        ),
        Span::styled("[N]", Style::default().fg(CYAN_DIM)),
        Span::styled("ext ", Style::default().fg(GRAY_DIM)),
        Span::styled("[P]", Style::default().fg(CYAN_DIM)),
        Span::styled("rev", Style::default().fg(GRAY_DIM)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CYAN_DIM))
        .title(tab_title);
    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

// ─── Activity log ────────────────────────────────────────────────────────────

fn render_log(frame: &mut Frame, area: Rect, game: &GameState) {
    let items: Vec<ListItem> = game
        .log
        .iter()
        .map(|msg| {
            let color = log_color(msg);
            ListItem::new(Line::from(vec![
                Span::styled(" \u{25b8} ", Style::default().fg(GRAY_DIM)),
                Span::styled(msg.as_str(), Style::default().fg(color)),
            ]))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(GRAY_DIM))
        .title(Span::styled(
            "╣ Activity Log ╠",
            Style::default().fg(GRAY_LIGHT),
        ));
    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn log_color(msg: &str) -> Color {
    let m = msg.to_lowercase();
    if m.contains("purchased") || m.contains("you won") || m.contains("win!")
        || m.contains("heads!") || m.contains("correct!")
    {
        GREEN_MID
    } else if m.contains("lost ") || m.contains("vanished") || m.contains("wrong!")
        || m.contains("tails!")
    {
        RED_BRIGHT
    } else if m.contains("golden") || m.contains("\u{2728}") {
        GOLD
    } else if m.contains("ascen") || m.contains("heavenly") {
        MAGENTA_BRIGHT
    } else if m.contains("frenzy") || m.contains("\u{1f525}") {
        ORANGE
    } else {
        CYAN_DIM
    }
}

// ─── Hotkey bar ──────────────────────────────────────────────────────────────

fn render_hotkeys(frame: &mut Frame, area: Rect, game: &GameState) {
    let mut spans: Vec<Span> = vec![
        Span::styled(" [", Style::default().fg(GRAY_DIM)),
        Span::styled("Space", Style::default().fg(CYAN_BRIGHT).add_modifier(Modifier::BOLD)),
        Span::styled("] ", Style::default().fg(GRAY_DIM)),
        Span::styled("Mine", Style::default().fg(GRAY_LIGHT)),
        Span::styled(" \u{2502} ", Style::default().fg(GRAY_DIM)),
        Span::styled("[", Style::default().fg(GRAY_DIM)),
        Span::styled("1-8", Style::default().fg(CYAN_BRIGHT).add_modifier(Modifier::BOLD)),
        Span::styled("] ", Style::default().fg(GRAY_DIM)),
        Span::styled("Buy", Style::default().fg(GRAY_LIGHT)),
        Span::styled(" \u{2502} ", Style::default().fg(GRAY_DIM)),
        Span::styled("[", Style::default().fg(GRAY_DIM)),
        Span::styled("T", Style::default().fg(CYAN_BRIGHT).add_modifier(Modifier::BOLD)),
        Span::styled("] ", Style::default().fg(GRAY_DIM)),
        Span::styled("Tab", Style::default().fg(GRAY_LIGHT)),
        Span::styled(" \u{2502} ", Style::default().fg(GRAY_DIM)),
        Span::styled("[", Style::default().fg(GRAY_DIM)),
        Span::styled("G", Style::default().fg(NEON_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled("] ", Style::default().fg(GRAY_DIM)),
        Span::styled("Casino", Style::default().fg(GRAY_LIGHT)),
        Span::styled(" \u{2502} ", Style::default().fg(GRAY_DIM)),
        Span::styled("[", Style::default().fg(GRAY_DIM)),
        Span::styled("S", Style::default().fg(CYAN_BRIGHT).add_modifier(Modifier::BOLD)),
        Span::styled("] ", Style::default().fg(GRAY_DIM)),
        Span::styled("Save", Style::default().fg(GRAY_LIGHT)),
        Span::styled(" \u{2502} ", Style::default().fg(GRAY_DIM)),
        Span::styled("[", Style::default().fg(GRAY_DIM)),
        Span::styled("Q", Style::default().fg(RED_BRIGHT).add_modifier(Modifier::BOLD)),
        Span::styled("] ", Style::default().fg(GRAY_DIM)),
        Span::styled("Quit", Style::default().fg(GRAY_LIGHT)),
        Span::styled(" \u{2502} ", Style::default().fg(GRAY_DIM)),
        Span::styled("[", Style::default().fg(GRAY_DIM)),
        Span::styled("/", Style::default().fg(NEON_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled("] ", Style::default().fg(GRAY_DIM)),
        Span::styled("Chat", Style::default().fg(GRAY_LIGHT)),
    ];

    if game.golden_collect_window > 0 {
        spans.push(Span::styled(" \u{2502} ", Style::default().fg(GRAY_DIM)));
        spans.push(Span::styled("[", Style::default().fg(GRAY_DIM)));
        spans.push(Span::styled(
            "C",
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled("] ", Style::default().fg(GRAY_DIM)));
        spans.push(Span::styled(
            "\u{1f36a} COLLECT",
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ));
    }

    if game.ascend_available {
        spans.push(Span::styled(" \u{2502} ", Style::default().fg(GRAY_DIM)));
        spans.push(Span::styled("[", Style::default().fg(GRAY_DIM)));
        spans.push(Span::styled(
            "A",
            Style::default()
                .fg(MAGENTA_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled("] ", Style::default().fg(GRAY_DIM)));
        spans.push(Span::styled(
            "Ascend",
            Style::default()
                .fg(MAGENTA_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(GRAY_DIM))
        .title(Span::styled(
            "╣ Hotkeys ╠",
            Style::default().fg(GRAY_LIGHT),
        ));
    let paragraph = Paragraph::new(Line::from(spans)).block(block);
    frame.render_widget(paragraph, area);
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Casino views
// ═══════════════════════════════════════════════════════════════════════════════

fn render_casino(frame: &mut Frame, area: Rect, game: &GameState, casino: &CasinoState) {
    match &casino.active_game {
        None => render_casino_menu(frame, area, game, casino),
        Some(CasinoGame::SlotMachine) => render_slots(frame, area, game, casino),
        Some(CasinoGame::CoinFlip) => render_coinflip(frame, area, game, casino),
        Some(CasinoGame::DiceWager) => render_dice(frame, area, game, casino),
        Some(CasinoGame::Roulette) => render_roulette(frame, area, game, casino),
        Some(CasinoGame::Blackjack) => render_blackjack(frame, area, game, casino),
    }
}

fn casino_balance_line(game: &GameState) -> Line<'static> {
    let bal = format_number(game.cookies);
    Line::from(vec![
        Span::styled(" Balance: ", Style::default().fg(CYAN_MID)),
        Span::styled(
            format!("{} cookies", bal),
            Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
        ),
    ])
}

// ─── Casino menu ─────────────────────────────────────────────────────────────

fn render_casino_menu(frame: &mut Frame, area: Rect, game: &GameState, casino: &CasinoState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(area);

    let net = casino.net_profit();
    let net_str = if net >= 0.0 {
        format!("+{}", format_number(net))
    } else {
        format!("-{}", format_number(-net))
    };
    let net_color = if net >= 0.0 { GREEN_BRIGHT } else { RED_BRIGHT };

    // Animated neon banner
    let phase = game.animation_tick as usize % 3;
    let neon = [NEON_PINK, NEON_GREEN, NEON_PURPLE];
    let banner_color = neon[phase];

    let mut lines = vec![
        Line::from(Span::styled(
            " \u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2557}",
            Style::default().fg(banner_color),
        )),
        Line::from(Span::styled(
            " \u{2551}  \u{1f3b0}  WELCOME TO THE COOKIE CASINO  \u{2551}",
            Style::default()
                .fg(banner_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            " \u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}",
            Style::default().fg(banner_color),
        )),
        Line::from(""),
        casino_balance_line(game),
        Line::from(""),
        Line::from(Span::styled(
            " Choose your poison:",
            Style::default().fg(GRAY_LIGHT),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [S] ",
                Style::default().fg(NEON_GREEN).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "\u{1f3b0} Slot Machine ",
                Style::default().fg(GREEN_BRIGHT),
            ),
            Span::styled(
                format!("  Bet: {} cookies", crate::casino::SLOT_BET as u64),
                Style::default().fg(GRAY_MID),
            ),
        ]),
        Line::from(Span::styled(
            "       3\u{00d7}\u{1f352}=5x \u{2502} 3\u{00d7}\u{1f34b}=3x \u{2502} 3\u{00d7}\u{1f514}=7x \u{2502} 3\u{00d7}\u{1f48e}=25x \u{2502} 3\u{00d7}7\u{fe0f}\u{20e3}=100x",
            Style::default().fg(GRAY_DIM),
        )),
        Line::from(vec![
            Span::styled(
                "  [F] ",
                Style::default().fg(CYAN_BRIGHT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "\u{1fa99} Coin Flip    ",
                Style::default().fg(CYAN_MID),
            ),
            Span::styled("  Custom bet, 2x or bust", Style::default().fg(GRAY_MID)),
        ]),
        Line::from(vec![
            Span::styled(
                "  [D] ",
                Style::default()
                    .fg(MAGENTA_BRIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "\u{1f3b2} Dice Wager   ",
                Style::default().fg(MAGENTA_DIM),
            ),
            Span::styled("  Guess 1-6, win 5x", Style::default().fg(GRAY_MID)),
        ]),
        Line::from(vec![
            Span::styled(
                "  [R] ",
                Style::default()
                    .fg(NEON_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "🎡 Roulette     ",
                Style::default().fg(GREEN_MID),
            ),
            Span::styled("  Red/Black/Green/Odd/Even/Hi/Lo", Style::default().fg(GRAY_MID)),
        ]),
        Line::from(vec![
            Span::styled(
                "  [B] ",
                Style::default()
                    .fg(GOLD)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "🃏 Blackjack    ",
                Style::default().fg(GOLD_DIM),
            ),
            Span::styled("  Hit 21 to win, beat the dealer", Style::default().fg(GRAY_MID)),
        ]),
        Line::from(""),
    ];

    // Lifetime stats
    if casino.total_spins > 0 || casino.total_wagered > 0.0 {
        lines.push(Line::from(Span::styled(
            " \u{2500}\u{2500}\u{2500} Lifetime Stats \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
            Style::default().fg(GRAY_DIM),
        )));
        lines.push(Line::from(vec![
            Span::styled("  Spins: ", Style::default().fg(GRAY_LIGHT)),
            Span::styled(
                casino.total_spins.to_string(),
                Style::default().fg(WHITE),
            ),
            Span::styled("   Wagered: ", Style::default().fg(GRAY_LIGHT)),
            Span::styled(
                format_number(casino.total_wagered),
                Style::default().fg(WHITE),
            ),
            Span::styled("   Net: ", Style::default().fg(GRAY_LIGHT)),
            Span::styled(net_str, Style::default().fg(net_color)),
        ]));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        " \u{26a0}  Gambling is for cookies, not real life!",
        Style::default().fg(RED_DIM),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(NEON_PURPLE))
        .title(Span::styled(
            "\u{2563} \u{1f3b0} Casino \u{2560}",
            Style::default()
                .fg(NEON_PINK)
                .add_modifier(Modifier::BOLD),
        ));
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, rows[0]);

    render_casino_hotkeys(
        frame,
        rows[1],
        &[
            ("S", "Slots", NEON_GREEN),
            ("F", "Flip", CYAN_BRIGHT),
            ("D", "Dice", MAGENTA_BRIGHT),
            ("R", "Roulette", GREEN_MID),
            ("B", "Blackjack", GOLD),
            ("G", "Exit", ORANGE),
            ("Q", "Quit", RED_BRIGHT),
        ],
    );
}

// ─── Slot machine ────────────────────────────────────────────────────────────

fn render_slots(frame: &mut Frame, area: Rect, game: &GameState, casino: &CasinoState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(area);

    let mut lines = vec![
        Line::from(Span::styled(
            " \u{1f3b0}  S L O T   M A C H I N E",
            Style::default()
                .fg(GOLD)
                .add_modifier(Modifier::BOLD),
        )),
        casino_balance_line(game),
        Line::from(vec![
            Span::styled(" Bet: ", Style::default().fg(CYAN_MID)),
            Span::styled(
                format!("{} cookies", crate::casino::SLOT_BET as u64),
                Style::default().fg(ORANGE),
            ),
        ]),
        Line::from(""),
    ];

    if let Some(ref result) = casino.last_slot {
        // Reel display box
        let r0 = result.reels[0].label();
        let r1 = result.reels[1].label();
        let r2 = result.reels[2].label();
        lines.push(Line::from(Span::styled(
            " \u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2566}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2566}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2557}",
            Style::default().fg(GOLD),
        )));
        lines.push(Line::from(vec![
            Span::styled(" \u{2551} ", Style::default().fg(GOLD)),
            Span::styled(format!(" {} ", r0), Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
            Span::styled(" \u{2551} ", Style::default().fg(GOLD)),
            Span::styled(format!(" {} ", r1), Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
            Span::styled(" \u{2551} ", Style::default().fg(GOLD)),
            Span::styled(format!(" {} ", r2), Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
            Span::styled(" \u{2551}", Style::default().fg(GOLD)),
        ]));
        lines.push(Line::from(Span::styled(
            " \u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2569}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2569}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}",
            Style::default().fg(GOLD),
        )));
        lines.push(Line::from(""));

        let (outcome_str, color) = if result.payout > 0.0 {
            (
                format!(
                    " \u{1f389} JACKPOT! +{} cookies!",
                    format_number(result.payout)
                ),
                GREEN_BRIGHT,
            )
        } else if result.payout == 0.0 {
            (" \u{1f610} No payout. Try again!".to_string(), GRAY_MID)
        } else {
            (
                format!(" \u{1f4b8} Lost {} cookies.", format_number(-result.payout)),
                RED_BRIGHT,
            )
        };
        lines.push(Line::from(Span::styled(
            outcome_str,
            Style::default().fg(color).add_modifier(if result.payout > 0.0 {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }),
        )));
    } else {
        // Animated spinning symbols when no result
        let symbols = ["\u{1f352}", "\u{1f34b}", "\u{1f514}", "\u{1f48e}", "7\u{fe0f}\u{20e3}"];
        let t = game.animation_tick as usize;
        let s0 = symbols[t % 5];
        let s1 = symbols[(t + 2) % 5];
        let s2 = symbols[(t + 4) % 5];
        lines.push(Line::from(Span::styled(
            " \u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2566}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2566}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2557}",
            Style::default().fg(GRAY_MID),
        )));
        lines.push(Line::from(vec![
            Span::styled(" \u{2551} ", Style::default().fg(GRAY_MID)),
            Span::styled(format!(" {} ", s0), Style::default().fg(GRAY_LIGHT)),
            Span::styled(" \u{2551} ", Style::default().fg(GRAY_MID)),
            Span::styled(format!(" {} ", s1), Style::default().fg(GRAY_LIGHT)),
            Span::styled(" \u{2551} ", Style::default().fg(GRAY_MID)),
            Span::styled(format!(" {} ", s2), Style::default().fg(GRAY_LIGHT)),
            Span::styled(" \u{2551}", Style::default().fg(GRAY_MID)),
        ]));
        lines.push(Line::from(Span::styled(
            " \u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2569}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2569}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}",
            Style::default().fg(GRAY_MID),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Press [S] to spin!",
            Style::default().fg(CYAN_MID),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Payouts: 3\u{00d7}\u{1f352}=5x \u{2502} 3\u{00d7}\u{1f34b}=3x \u{2502} 3\u{00d7}\u{1f514}=7x \u{2502} 3\u{00d7}\u{1f48e}=25x \u{2502} 3\u{00d7}7\u{fe0f}\u{20e3}=100x",
        Style::default().fg(GRAY_MID),
    )));
    if casino.total_spins > 0 {
        lines.push(Line::from(Span::styled(
            format!(" Total spins: {}", casino.total_spins),
            Style::default().fg(GRAY_DIM),
        )));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(GOLD_DIM))
        .title(Span::styled(
            "\u{2563} \u{1f3b0} Slot Machine \u{2560}",
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ));
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, rows[0]);

    render_casino_hotkeys(
        frame,
        rows[1],
        &[
            ("S", "Spin", NEON_GREEN),
            ("G", "Menu", ORANGE),
            ("Q", "Quit", RED_BRIGHT),
        ],
    );
}

// ─── Coin flip ───────────────────────────────────────────────────────────────

fn render_coinflip(frame: &mut Frame, area: Rect, game: &GameState, casino: &CasinoState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(area);

    let wager_display = if casino.wager_input.is_empty() {
        "enter wager...".to_string()
    } else {
        casino.wager_input.clone()
    };

    let mut lines = vec![
        Line::from(Span::styled(
            " \u{1fa99}  C O I N   F L I P",
            Style::default()
                .fg(CYAN_BRIGHT)
                .add_modifier(Modifier::BOLD),
        )),
        casino_balance_line(game),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Wager: ", Style::default().fg(CYAN_MID)),
            Span::styled(
                &wager_display,
                Style::default()
                    .fg(GOLD)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  (min {})", crate::casino::COINFLIP_MIN_BET as u64),
                Style::default().fg(GRAY_DIM),
            ),
        ]),
        Line::from(""),
    ];

    if let Some(ref result) = casino.last_coin {
        // Coin art
        if result.won {
            lines.push(Line::from(Span::styled(
                "      ╭────────╮",
                Style::default().fg(GREEN_BRIGHT),
            )));
            lines.push(Line::from(Span::styled(
                "      │ HEADS  │",
                Style::default()
                    .fg(GREEN_BRIGHT)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                "      │  ◉  ◉  │",
                Style::default().fg(GREEN_MID),
            )));
            lines.push(Line::from(Span::styled(
                "      ╰────────╯",
                Style::default().fg(GREEN_BRIGHT),
            )));
            lines.push(Line::from(Span::styled(
                format!(
                    " \u{1f389} You won {} cookies!",
                    format_number(result.net)
                ),
                Style::default()
                    .fg(GREEN_BRIGHT)
                    .add_modifier(Modifier::BOLD),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "      ╭────────╮",
                Style::default().fg(RED_BRIGHT),
            )));
            lines.push(Line::from(Span::styled(
                "      │ TAILS  │",
                Style::default()
                    .fg(RED_BRIGHT)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                "      │  ╳  ╳  │",
                Style::default().fg(RED_DIM),
            )));
            lines.push(Line::from(Span::styled(
                "      ╰────────╯",
                Style::default().fg(RED_BRIGHT),
            )));
            lines.push(Line::from(Span::styled(
                format!(
                    " \u{1f4b8} Lost {} cookies.",
                    format_number(-result.net)
                ),
                Style::default().fg(RED_BRIGHT),
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            " 50/50 chance \u{2502} Win = 2\u{00d7} your wager",
            Style::default().fg(GRAY_MID),
        )));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CYAN_DIM))
        .title(Span::styled(
            "\u{2563} \u{1fa99} Coin Flip \u{2560}",
            Style::default()
                .fg(CYAN_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ));
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, rows[0]);

    render_casino_hotkeys(
        frame,
        rows[1],
        &[
            ("0-9", "Wager", CYAN_BRIGHT),
            ("F", "Flip", NEON_GREEN),
            ("\u{232b}", "Erase", GRAY_LIGHT),
            ("G", "Menu", ORANGE),
        ],
    );
}

// ─── Dice wager ──────────────────────────────────────────────────────────────

fn render_dice(frame: &mut Frame, area: Rect, game: &GameState, casino: &CasinoState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(area);

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
            " \u{1f3b2}  D I C E   W A G E R",
            Style::default()
                .fg(MAGENTA_BRIGHT)
                .add_modifier(Modifier::BOLD),
        )),
        casino_balance_line(game),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Wager:     ", Style::default().fg(CYAN_MID)),
            Span::styled(
                &wager_display,
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  (min {})", crate::casino::DICE_MIN_BET as u64),
                Style::default().fg(GRAY_DIM),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Guess 1-6: ", Style::default().fg(CYAN_MID)),
            Span::styled(
                &guess_display,
                Style::default()
                    .fg(GOLD)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    if let Some(ref result) = casino.last_dice {
        // 3x3 die face
        let face = die_face_art(result.rolled);
        let face_color = if result.won { GREEN_BRIGHT } else { RED_BRIGHT };
        for row in &face {
            lines.push(Line::from(Span::styled(
                format!("      {}", row),
                Style::default().fg(face_color),
            )));
        }
        lines.push(Line::from(Span::styled(
            format!(
                " Rolled: {} (you guessed {})",
                die_ascii(result.rolled),
                result.guessed
            ),
            Style::default().fg(WHITE),
        )));

        let (outcome_str, color) = if result.won {
            (
                format!(
                    " \u{1f389} Correct! +{} cookies!",
                    format_number(result.net)
                ),
                GREEN_BRIGHT,
            )
        } else {
            (
                format!(
                    " \u{1f4b8} Wrong! Lost {} cookies.",
                    format_number(-result.net)
                ),
                RED_BRIGHT,
            )
        };
        lines.push(Line::from(Span::styled(
            outcome_str,
            Style::default().fg(color).add_modifier(if result.won {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Win: 1/6 \u{2502} Payout: 5x bet (net +4x)",
        Style::default().fg(GRAY_MID),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MAGENTA_DIM))
        .title(Span::styled(
            "\u{2563} \u{1f3b2} Dice Wager \u{2560}",
            Style::default()
                .fg(MAGENTA_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ));
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, rows[0]);

    let step_keys: &[(&str, &str, Color)] = if casino.entering_wager {
        &[
            ("0-9", "Wager", CYAN_BRIGHT),
            ("\u{21b5}", "Confirm", NEON_GREEN),
            ("\u{232b}", "Erase", GRAY_LIGHT),
        ]
    } else if casino.entering_dice_guess {
        &[
            ("1-6", "Guess", MAGENTA_BRIGHT),
            ("D", "Roll", NEON_GREEN),
            ("\u{232b}", "Erase", GRAY_LIGHT),
        ]
    } else {
        &[
            ("0-9", "Wager", CYAN_BRIGHT),
            ("D", "Roll", NEON_GREEN),
            ("G", "Menu", ORANGE),
            ("Q", "Quit", RED_BRIGHT),
        ]
    };
    render_casino_hotkeys(frame, rows[1], step_keys);
}

// ─── Roulette ────────────────────────────────────────────────────────────────

fn render_roulette(frame: &mut Frame, area: Rect, game: &GameState, casino: &CasinoState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(area);

    let wager_display = if casino.wager_input.is_empty() {
        "enter wager...".to_string()
    } else {
        casino.wager_input.clone()
    };

    let mut lines = vec![
        Line::from(Span::styled(
            " 🎡  R O U L E T T E",
            Style::default()
                .fg(GREEN_BRIGHT)
                .add_modifier(Modifier::BOLD),
        )),
        casino_balance_line(game),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Wager: ", Style::default().fg(CYAN_MID)),
            Span::styled(
                &wager_display,
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  (min {})", crate::casino::ROULETTE_MIN_BET as u64),
                Style::default().fg(GRAY_DIM),
            ),
        ]),
        Line::from(""),
    ];

    if casino.entering_wager {
        lines.push(Line::from(Span::styled(
            " Enter wager, then press Enter to confirm.",
            Style::default().fg(CYAN_MID),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            " Choose your bet:",
            Style::default().fg(GRAY_LIGHT),
        )));
        lines.push(Line::from(vec![
            Span::styled("   [R] ", Style::default().fg(RED_BRIGHT).add_modifier(Modifier::BOLD)),
            Span::styled("Red (2x)   ", Style::default().fg(RED_BRIGHT)),
            Span::styled("   [B] ", Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
            Span::styled("Black (2x)", Style::default().fg(WHITE)),
            Span::styled("   [Z] ", Style::default().fg(GREEN_BRIGHT).add_modifier(Modifier::BOLD)),
            Span::styled("Green/0 (14x)", Style::default().fg(GREEN_BRIGHT)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("   [O] ", Style::default().fg(CYAN_MID).add_modifier(Modifier::BOLD)),
            Span::styled("Odd (2x)   ", Style::default().fg(CYAN_MID)),
            Span::styled("   [E] ", Style::default().fg(CYAN_MID).add_modifier(Modifier::BOLD)),
            Span::styled("Even (2x) ", Style::default().fg(CYAN_MID)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("   [L] ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::styled("Low 1-18 (2x) ", Style::default().fg(ORANGE)),
            Span::styled("[H] ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::styled("High 19-36 (2x)", Style::default().fg(ORANGE)),
        ]));
    }
    lines.push(Line::from(""));

    if let Some(ref result) = casino.last_roulette {
        let color_label = match result.color {
            crate::casino::RouletteColor::Red => ("🔴", RED_BRIGHT),
            crate::casino::RouletteColor::Black => ("⚫", WHITE),
            crate::casino::RouletteColor::Green => ("🟢", GREEN_BRIGHT),
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!(" Landed on: {} {} ", color_label.0, result.number),
                Style::default().fg(color_label.1).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("(Bet: {})", result.bet_type.label()),
                Style::default().fg(GRAY_MID),
            ),
        ]));

        if result.won {
            lines.push(Line::from(Span::styled(
                format!(" 🎉 You won! +{} cookies!", format_number(result.net)),
                Style::default().fg(GREEN_BRIGHT).add_modifier(Modifier::BOLD),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                format!(" 💸 Lost {} cookies.", format_number(result.wager)),
                Style::default().fg(RED_BRIGHT),
            )));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(GREEN_MID))
        .title(Span::styled(
            "╣ 🎡 Roulette ╠",
            Style::default()
                .fg(GREEN_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ));
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, rows[0]);

    let hotkeys: &[(&str, &str, Color)] = if casino.entering_wager {
        &[
            ("0-9", "Wager", CYAN_BRIGHT),
            ("\u{21b5}", "Confirm", NEON_GREEN),
            ("\u{232b}", "Erase", GRAY_LIGHT),
            ("G", "Menu", ORANGE),
        ]
    } else {
        &[
            ("R", "Red", RED_BRIGHT),
            ("B", "Black", WHITE),
            ("Z", "Zero", GREEN_BRIGHT),
            ("O", "Odd", CYAN_MID),
            ("E", "Even", CYAN_MID),
            ("G", "Menu", ORANGE),
        ]
    };
    render_casino_hotkeys(frame, rows[1], hotkeys);
}

// ─── Blackjack ───────────────────────────────────────────────────────────────

fn render_blackjack(frame: &mut Frame, area: Rect, game: &GameState, casino: &CasinoState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(area);

    let mut lines = vec![
        Line::from(Span::styled(
            " 🃏  B L A C K J A C K",
            Style::default()
                .fg(GOLD)
                .add_modifier(Modifier::BOLD),
        )),
        casino_balance_line(game),
        Line::from(""),
    ];

    if let Some(ref hand) = casino.blackjack_hand {
        let wager_str = format_number(hand.wager);
        lines.push(Line::from(vec![
            Span::styled(" Wager: ", Style::default().fg(CYAN_MID)),
            Span::styled(
                format!("{} cookies", wager_str),
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));

        // Dealer's hand
        let dealer_display: String = if hand.phase == crate::casino::BlackjackPhase::PlayerTurn {
            if hand.dealer_cards.is_empty() {
                "?".to_string()
            } else {
                format!("{} [?]", hand.dealer_cards[0].display())
            }
        } else {
            hand.dealer_cards
                .iter()
                .map(|c| c.display())
                .collect::<Vec<_>>()
                .join(" ")
        };

        let dealer_score = if hand.phase == crate::casino::BlackjackPhase::PlayerTurn {
            "?".to_string()
        } else {
            crate::casino::hand_value(&hand.dealer_cards).to_string()
        };

        lines.push(Line::from(vec![
            Span::styled(" Dealer: ", Style::default().fg(RED_BRIGHT)),
            Span::styled(
                dealer_display,
                Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  ({})", dealer_score), Style::default().fg(GRAY_MID)),
        ]));

        // Player's hand
        let player_display: String = hand
            .player_cards
            .iter()
            .map(|c| c.display())
            .collect::<Vec<_>>()
            .join(" ");
        let player_val = crate::casino::hand_value(&hand.player_cards);

        let val_color = if player_val > 21 {
            RED_BRIGHT
        } else if player_val == 21 {
            GREEN_BRIGHT
        } else {
            WHITE
        };

        lines.push(Line::from(vec![
            Span::styled(" You:    ", Style::default().fg(CYAN_BRIGHT)),
            Span::styled(
                player_display,
                Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  ({})", player_val),
                Style::default().fg(val_color),
            ),
        ]));
        lines.push(Line::from(""));

        // Show outcome if resolved
        if hand.phase == crate::casino::BlackjackPhase::Resolved {
            if let Some(ref result) = casino.last_blackjack {
                let (msg, color) = match result.outcome {
                    crate::casino::BlackjackOutcome::PlayerBlackjack => {
                        (format!("🎉 BLACKJACK! +{} cookies!", format_number(result.net)), GREEN_BRIGHT)
                    }
                    crate::casino::BlackjackOutcome::PlayerWin => {
                        (format!("🎉 You win! +{} cookies!", format_number(result.net)), GREEN_BRIGHT)
                    }
                    crate::casino::BlackjackOutcome::DealerBust => {
                        (format!("🎉 Dealer busts! +{} cookies!", format_number(result.net)), GREEN_BRIGHT)
                    }
                    crate::casino::BlackjackOutcome::Push => {
                        ("🤝 Push! Bet returned.".to_string(), GOLD)
                    }
                    crate::casino::BlackjackOutcome::PlayerBust => {
                        (format!("💥 BUST! Lost {} cookies.", format_number(result.wager)), RED_BRIGHT)
                    }
                    crate::casino::BlackjackOutcome::DealerWin => {
                        (format!("😞 Dealer wins. Lost {} cookies.", format_number(result.wager)), RED_BRIGHT)
                    }
                };
                lines.push(Line::from(Span::styled(
                    format!(" {}", msg),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    " Press [N] for a new hand.",
                    Style::default().fg(GRAY_LIGHT),
                )));
            }
        } else if hand.phase == crate::casino::BlackjackPhase::PlayerTurn {
            lines.push(Line::from(Span::styled(
                " [H]it or [S]tand?",
                Style::default().fg(CYAN_BRIGHT).add_modifier(Modifier::BOLD),
            )));
        }
    } else {
        // No hand: show betting UI
        let wager_display = if casino.wager_input.is_empty() {
            "enter wager...".to_string()
        } else {
            casino.wager_input.clone()
        };
        lines.push(Line::from(vec![
            Span::styled(" Wager: ", Style::default().fg(CYAN_MID)),
            Span::styled(
                wager_display,
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  (min {})", crate::casino::BLACKJACK_MIN_BET as u64),
                Style::default().fg(GRAY_DIM),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Enter wager and press Enter to deal.",
            Style::default().fg(GRAY_LIGHT),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Rules: Beat dealer without going over 21.",
            Style::default().fg(GRAY_MID),
        )));
        lines.push(Line::from(Span::styled(
            " Blackjack (21 with 2 cards) pays 2.5x. Win pays 2x.",
            Style::default().fg(GRAY_MID),
        )));
        lines.push(Line::from(Span::styled(
            " Dealer stands on 17+. Ace = 1 or 11.",
            Style::default().fg(GRAY_MID),
        )));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(GOLD_DIM))
        .title(Span::styled(
            "╣ 🃏 Blackjack ╠",
            Style::default()
                .fg(GOLD)
                .add_modifier(Modifier::BOLD),
        ));
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, rows[0]);

    let phase = casino
        .blackjack_hand
        .as_ref()
        .map(|h| h.phase)
        .unwrap_or(crate::casino::BlackjackPhase::Betting);

    let hotkeys: &[(&str, &str, Color)] = match phase {
        crate::casino::BlackjackPhase::PlayerTurn => &[
            ("H", "Hit", NEON_GREEN),
            ("S", "Stand", ORANGE),
        ],
        crate::casino::BlackjackPhase::Resolved => &[
            ("N", "New Hand", CYAN_BRIGHT),
            ("G", "Menu", ORANGE),
            ("Q", "Quit", RED_BRIGHT),
        ],
        _ => &[
            ("0-9", "Wager", CYAN_BRIGHT),
            ("\u{21b5}", "Deal", NEON_GREEN),
            ("\u{232b}", "Erase", GRAY_LIGHT),
            ("G", "Menu", ORANGE),
        ],
    };
    render_casino_hotkeys(frame, rows[1], hotkeys);
}

// ─── Chat view ───────────────────────────────────────────────────────────────

fn render_chat(frame: &mut Frame, area: Rect, chat: &ChatState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),   // Messages
            Constraint::Length(3), // Input
            Constraint::Length(3), // Hotkeys
        ])
        .split(area);

    // Header
    let status = if chat.is_connected() {
        Span::styled(" 🟢 Connected", Style::default().fg(GREEN_BRIGHT))
    } else {
        Span::styled(" 🔴 Offline", Style::default().fg(RED_BRIGHT))
    };
    let header_lines = vec![Line::from(vec![
        Span::styled(
            " 💬 Global Chat  ",
            Style::default().fg(CYAN_BRIGHT).add_modifier(Modifier::BOLD),
        ),
        status,
        Span::styled(
            format!("  │  {} ", chat.identity),
            Style::default().fg(GRAY_MID),
        ),
        Span::styled(
            format!("  │  {} users known", chat.known_users.len()),
            Style::default().fg(GRAY_DIM),
        ),
    ])];
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CYAN_DIM))
        .title(Span::styled(
            "╣ Chat ╠",
            Style::default().fg(CYAN_BRIGHT).add_modifier(Modifier::BOLD),
        ));
    let header = Paragraph::new(header_lines)
        .block(header_block)
        .wrap(Wrap { trim: false });
    frame.render_widget(header, rows[0]);

    // Messages
    let msg_items: Vec<ListItem> = chat
        .messages
        .iter()
        .map(|msg| {
            let is_self = msg.sender == chat.identity;
            let is_pinged = ChatState::is_user_pinged(&msg.content, &chat.identity);

            let sender_color = if is_self { CYAN_BRIGHT } else { NEON_GREEN };
            let content_color = if is_pinged {
                GOLD
            } else if is_self {
                WHITE
            } else {
                GRAY_LIGHT
            };

            let ping_marker = if is_pinged { " 🔔" } else { "" };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {}: ", msg.sender),
                    Style::default().fg(sender_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &msg.content,
                    Style::default().fg(content_color),
                ),
                Span::styled(
                    ping_marker.to_string(),
                    Style::default().fg(GOLD),
                ),
            ]))
        })
        .collect();

    let msg_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(GRAY_DIM))
        .title(Span::styled(
            "╣ Messages ╠",
            Style::default().fg(GRAY_LIGHT),
        ));
    let msg_list = List::new(msg_items).block(msg_block);
    frame.render_widget(msg_list, rows[1]);

    // Input box
    let input_display = if chat.input_buffer.is_empty() {
        "Type a message... ($ to ping users, Tab to autocomplete)"
    } else {
        &chat.input_buffer
    };
    let input_color = if chat.input_buffer.is_empty() {
        GRAY_DIM
    } else {
        WHITE
    };
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CYAN_MID))
        .title(Span::styled(
            "╣ Input ╠",
            Style::default().fg(CYAN_BRIGHT),
        ));
    let input = Paragraph::new(Line::from(vec![
        Span::styled(" > ", Style::default().fg(CYAN_BRIGHT)),
        Span::styled(input_display, Style::default().fg(input_color)),
        Span::styled("█", Style::default().fg(CYAN_BRIGHT)), // Cursor
    ]))
    .block(input_block);
    frame.render_widget(input, rows[2]);

    // Hotkeys
    render_casino_hotkeys(
        frame,
        rows[3],
        &[
            ("Enter", "Send", NEON_GREEN),
            ("Tab", "Autocomplete", CYAN_BRIGHT),
            ("$user", "Ping", GOLD),
            ("Esc", "Close", RED_BRIGHT),
        ],
    );
}

// ─── Shared casino hotkey bar ────────────────────────────────────────────────

fn render_casino_hotkeys(frame: &mut Frame, area: Rect, keys: &[(&str, &str, Color)]) {
    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, label, color)) in keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" \u{2502} ", Style::default().fg(GRAY_DIM)));
        }
        spans.push(Span::styled("[", Style::default().fg(GRAY_DIM)));
        spans.push(Span::styled(
            *key,
            Style::default().fg(*color).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled("] ", Style::default().fg(GRAY_DIM)));
        spans.push(Span::styled(*label, Style::default().fg(GRAY_LIGHT)));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(GRAY_DIM))
        .title(Span::styled(
            "\u{2563} Hotkeys \u{2560}",
            Style::default().fg(GRAY_LIGHT),
        ));
    let paragraph = Paragraph::new(Line::from(spans)).block(block);
    frame.render_widget(paragraph, area);
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Helpers
// ═══════════════════════════════════════════════════════════════════════════════

fn format_number(n: f64) -> String {
    if n >= 1e21 {
        format!("{:.2}Sx", n / 1e21)
    } else if n >= 1e18 {
        format!("{:.2}Qi", n / 1e18)
    } else if n >= 1e15 {
        format!("{:.2}Qa", n / 1e15)
    } else if n >= 1e12 {
        format!("{:.2}T", n / 1e12)
    } else if n >= 1e9 {
        format!("{:.2}B", n / 1e9)
    } else if n >= 1e6 {
        format!("{:.2}M", n / 1e6)
    } else if n >= 1_000.0 {
        // Comma-separated
        let i = n as u64;
        if i >= 1_000_000 {
            let m = i / 1_000_000;
            let k = (i % 1_000_000) / 1_000;
            let r = i % 1_000;
            format!("{},{:03},{:03}", m, k, r)
        } else {
            let k = i / 1_000;
            let r = i % 1_000;
            format!("{},{:03}", k, r)
        }
    } else {
        format!("{:.0}", n)
    }
}

fn die_ascii(n: u8) -> &'static str {
    match n {
        1 => "\u{2680}",
        2 => "\u{2681}",
        3 => "\u{2682}",
        4 => "\u{2683}",
        5 => "\u{2684}",
        6 => "\u{2685}",
        _ => "?",
    }
}

fn die_face_art(n: u8) -> [&'static str; 5] {
    match n {
        1 => [
            "\u{250c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}",
            "\u{2502}     \u{2502}",
            "\u{2502}  \u{25cf}  \u{2502}",
            "\u{2502}     \u{2502}",
            "\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}",
        ],
        2 => [
            "\u{250c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}",
            "\u{2502} \u{25cf}   \u{2502}",
            "\u{2502}     \u{2502}",
            "\u{2502}   \u{25cf} \u{2502}",
            "\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}",
        ],
        3 => [
            "\u{250c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}",
            "\u{2502} \u{25cf}   \u{2502}",
            "\u{2502}  \u{25cf}  \u{2502}",
            "\u{2502}   \u{25cf} \u{2502}",
            "\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}",
        ],
        4 => [
            "\u{250c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}",
            "\u{2502} \u{25cf} \u{25cf} \u{2502}",
            "\u{2502}     \u{2502}",
            "\u{2502} \u{25cf} \u{25cf} \u{2502}",
            "\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}",
        ],
        5 => [
            "\u{250c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}",
            "\u{2502} \u{25cf} \u{25cf} \u{2502}",
            "\u{2502}  \u{25cf}  \u{2502}",
            "\u{2502} \u{25cf} \u{25cf} \u{2502}",
            "\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}",
        ],
        6 => [
            "\u{250c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}",
            "\u{2502} \u{25cf} \u{25cf} \u{2502}",
            "\u{2502} \u{25cf} \u{25cf} \u{2502}",
            "\u{2502} \u{25cf} \u{25cf} \u{2502}",
            "\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}",
        ],
        _ => [
            "\u{250c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}",
            "\u{2502}     \u{2502}",
            "\u{2502}  ?  \u{2502}",
            "\u{2502}     \u{2502}",
            "\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}",
        ],
    }
}
