use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const ASCENSION_THRESHOLD: f64 = 1_000_000.0;
pub const MAX_LOG_ENTRIES: usize = 5;

/// A building that generates cookies per second.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub name: String,
    pub base_cost: f64,
    pub base_cps: f64,
    pub owned: u32,
}

impl Building {
    /// Cost to buy the next unit: base_cost * 1.15^owned
    pub fn next_cost(&self) -> f64 {
        self.base_cost * 1.15_f64.powi(self.owned as i32)
    }

    /// Total CPS contribution of this building.
    pub fn cps(&self) -> f64 {
        self.base_cps * self.owned as f64
    }
}

/// Full game state – serialised to/from save.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Cookies currently in the jar.
    pub cookies: f64,
    /// All-time cookies baked (never decreases, used for prestige).
    pub total_baked: f64,
    /// Heavenly chips from previous ascensions.
    pub heavenly_chips: u32,
    /// The four buildings.
    pub buildings: Vec<Building>,
    /// Recent log messages shown in the bottom panel.
    pub log: VecDeque<String>,
    /// Whether the casino overlay is open.
    pub casino_open: bool,
    /// Seconds since last autosave (tracked outside of serde-state, but kept here for convenience).
    #[serde(skip)]
    pub ticks_since_save: u32,
    /// Fractional cookie accumulator for sub-second CPS.
    #[serde(skip)]
    pub cps_remainder: f64,
    /// Whether the ascend action is available.
    pub ascend_available: bool,
    /// Number of ascensions completed.
    pub ascension_count: u32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            cookies: 0.0,
            total_baked: 0.0,
            heavenly_chips: 0,
            buildings: default_buildings(),
            log: VecDeque::new(),
            casino_open: false,
            ticks_since_save: 0,
            cps_remainder: 0.0,
            ascend_available: false,
            ascension_count: 0,
        }
    }
}

impl GameState {
    /// Global CPS multiplier from heavenly chips: 1 + 0.01 * chips.
    pub fn heavenly_multiplier(&self) -> f64 {
        1.0 + 0.01 * self.heavenly_chips as f64
    }

    /// Total cookies generated per second.
    pub fn total_cps(&self) -> f64 {
        let raw: f64 = self.buildings.iter().map(|b| b.cps()).sum();
        raw * self.heavenly_multiplier()
    }

    /// Advance the game by one tick (called every ~250 ms).
    pub fn tick(&mut self, tick_rate_seconds: f64) {
        let gained = self.total_cps() * tick_rate_seconds;
        self.cookies += gained;
        self.total_baked += gained;

        self.ascend_available = self.total_baked >= ASCENSION_THRESHOLD;

        self.ticks_since_save += 1;
    }

    /// Player manually clicks to mine one cookie.
    pub fn mine_cookie(&mut self) {
        self.cookies += 1.0;
        self.total_baked += 1.0;
        self.ascend_available = self.total_baked >= ASCENSION_THRESHOLD;
    }

    /// Attempt to purchase the building at `index` (0-based).
    /// Returns true if successful.
    pub fn buy_building(&mut self, index: usize) -> bool {
        if index >= self.buildings.len() {
            return false;
        }
        let cost = self.buildings[index].next_cost();
        if self.cookies >= cost {
            self.cookies -= cost;
            self.buildings[index].owned += 1;
            let name = self.buildings[index].name.clone();
            self.push_log(format!(
                "{} purchased for {:.0} cookies.",
                name,
                cost
            ));
            true
        } else {
            false
        }
    }

    /// Perform ascension: reset progress, award heavenly chips.
    pub fn ascend(&mut self) {
        if !self.ascend_available {
            return;
        }
        let chips_earned = (self.total_baked / ASCENSION_THRESHOLD).floor() as u32;
        self.heavenly_chips += chips_earned;
        self.ascension_count += 1;
        let msg = format!(
            "Ascended! Earned {} Heavenly Chip(s). Total: {}",
            chips_earned, self.heavenly_chips
        );
        // Reset progress
        self.cookies = 0.0;
        self.total_baked = 0.0;
        self.buildings = default_buildings();
        self.ascend_available = false;
        self.cps_remainder = 0.0;
        self.push_log(msg);
    }

    /// Add a message to the rolling log (capped at MAX_LOG_ENTRIES).
    pub fn push_log(&mut self, msg: String) {
        if self.log.len() >= MAX_LOG_ENTRIES {
            self.log.pop_front();
        }
        self.log.push_back(msg);
    }

    /// Add cookies directly (e.g., from casino wins).
    pub fn add_cookies(&mut self, amount: f64) {
        self.cookies += amount;
        self.total_baked += amount;
        self.ascend_available = self.total_baked >= ASCENSION_THRESHOLD;
    }

    /// Remove cookies (e.g., casino bets). Returns false if insufficient.
    pub fn spend_cookies(&mut self, amount: f64) -> bool {
        if self.cookies >= amount {
            self.cookies -= amount;
            true
        } else {
            false
        }
    }
}

fn default_buildings() -> Vec<Building> {
    vec![
        Building {
            name: "Cursor".to_string(),
            base_cost: 15.0,
            base_cps: 0.1,
            owned: 0,
        },
        Building {
            name: "Grandma".to_string(),
            base_cost: 100.0,
            base_cps: 1.0,
            owned: 0,
        },
        Building {
            name: "Farm".to_string(),
            base_cost: 1_100.0,
            base_cps: 8.0,
            owned: 0,
        },
        Building {
            name: "Mine".to_string(),
            base_cost: 12_000.0,
            base_cps: 47.0,
            owned: 0,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_next_cost_zero_owned() {
        let b = Building {
            name: "Cursor".to_string(),
            base_cost: 15.0,
            base_cps: 0.1,
            owned: 0,
        };
        assert!((b.next_cost() - 15.0).abs() < 0.001);
    }

    #[test]
    fn test_building_cost_scaling() {
        let mut b = Building {
            name: "Cursor".to_string(),
            base_cost: 15.0,
            base_cps: 0.1,
            owned: 1,
        };
        // 15 * 1.15^1 = 17.25
        assert!((b.next_cost() - 17.25).abs() < 0.001);
        b.owned = 2;
        // 15 * 1.15^2 = 19.8375
        assert!((b.next_cost() - 19.8375).abs() < 0.001);
    }

    #[test]
    fn test_mine_cookie() {
        let mut gs = GameState::default();
        gs.mine_cookie();
        assert!((gs.cookies - 1.0).abs() < 0.001);
        assert!((gs.total_baked - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_buy_building_success() {
        let mut gs = GameState::default();
        gs.cookies = 100.0;
        let result = gs.buy_building(0); // Cursor costs 15
        assert!(result);
        assert_eq!(gs.buildings[0].owned, 1);
        assert!((gs.cookies - 85.0).abs() < 0.001);
    }

    #[test]
    fn test_buy_building_insufficient_cookies() {
        let mut gs = GameState::default();
        gs.cookies = 10.0;
        let result = gs.buy_building(0); // Cursor costs 15
        assert!(!result);
        assert_eq!(gs.buildings[0].owned, 0);
    }

    #[test]
    fn test_ascend() {
        let mut gs = GameState::default();
        gs.total_baked = 2_000_000.0;
        gs.cookies = 2_000_000.0;
        gs.ascend_available = true;
        gs.ascend();
        assert_eq!(gs.heavenly_chips, 2); // 2_000_000 / 1_000_000 = 2
        assert!((gs.cookies).abs() < 0.001);
        assert!((gs.total_baked).abs() < 0.001);
    }

    #[test]
    fn test_total_cps_with_heavenly_chips() {
        let mut gs = GameState::default();
        gs.heavenly_chips = 100; // +100% = 2x multiplier
        gs.buildings[0].owned = 10; // 10 cursors * 0.1 cps = 1.0 raw CPS
        // total = 1.0 * 2.0 = 2.0
        assert!((gs.total_cps() - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_tick_adds_cookies() {
        let mut gs = GameState::default();
        gs.buildings[1].owned = 1; // 1 Grandma = 1.0 CPS
        gs.tick(1.0); // 1 second tick
        assert!((gs.cookies - 1.0).abs() < 0.001);
    }
}
