use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const ASCENSION_THRESHOLD: f64 = 1_000_000.0;
pub const MAX_LOG_ENTRIES: usize = 8;
/// Number of upgrades shown per page in the UI.
pub const UPGRADES_PER_PAGE: usize = 8;
/// Probability that a golden cookie grants Frenzy mode instead of bonus cookies.
pub const GOLDEN_FRENZY_PROBABILITY: f64 = 0.5;
/// Duration of frenzy mode in ticks (120 ticks × 0.25s/tick = 30 seconds).
pub const FRENZY_DURATION_TICKS: u32 = 120;

// ─── Upgrades ────────────────────────────────────────────────────────────────

/// A one-time purchasable upgrade that multiplies a building's CPS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upgrade {
    pub name: String,
    pub description: String,
    pub cost: f64,
    pub purchased: bool,
    /// Index into `buildings` that this upgrade boosts.
    pub building_index: usize,
    /// Multiplicative CPS factor applied to the target building (stacks with other upgrades).
    pub multiplier: f64,
}

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
    /// The eight buildings.
    pub buildings: Vec<Building>,
    /// All purchasable upgrades.
    pub upgrades: Vec<Upgrade>,
    /// Total manual clicks ever made.
    pub total_clicks: u64,
    /// Recent log messages shown in the bottom panel.
    pub log: VecDeque<String>,
    /// Whether the casino overlay is open.
    pub casino_open: bool,
    /// Ticks since last autosave.
    #[serde(skip)]
    pub ticks_since_save: u32,
    /// Fractional cookie accumulator for sub-second CPS.
    #[serde(skip)]
    pub cps_remainder: f64,
    /// Whether the ascend action is available.
    pub ascend_available: bool,
    /// Number of ascensions completed.
    pub ascension_count: u32,

    // ── Golden Cookie ──────────────────────────────────────────────────────
    /// Ticks remaining in the golden cookie collection window (0 = none active).
    #[serde(skip)]
    pub golden_collect_window: u32,
    /// Pre-computed bonus cookies for the active golden cookie.
    #[serde(skip)]
    pub golden_cookie_bonus: f64,
    /// Ticks to wait before the next golden cookie can spawn.
    #[serde(skip)]
    pub golden_spawn_cooldown: u32,

    // ── Frenzy ──────────────────────────────────────────────────────────────
    /// Ticks remaining in frenzy mode (0 = not active).
    #[serde(skip)]
    pub frenzy_ticks: u32,
    /// CPS multiplier during frenzy (default 1.0).
    #[serde(skip, default = "default_frenzy_multiplier")]
    pub frenzy_multiplier: f64,

    // ── Animation ────────────────────────────────────────────────────────
    /// Monotonically increasing tick counter for UI animations.
    #[serde(skip)]
    pub animation_tick: u32,
    /// Counts down after each click for visual feedback.
    #[serde(skip)]
    pub click_animation: u32,

    // ── UI state ───────────────────────────────────────────────────────────
    /// Active tab in the main view: 1 = Buildings, 2 = Upgrades.
    #[serde(skip, default = "default_active_tab")]
    pub active_tab: u8,
    /// Current page of upgrades (0-based, 8 upgrades per page).
    #[serde(skip)]
    pub upgrade_page: u8,
}

fn default_active_tab() -> u8 {
    1
}

fn default_frenzy_multiplier() -> f64 {
    1.0
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            cookies: 0.0,
            total_baked: 0.0,
            heavenly_chips: 0,
            buildings: default_buildings(),
            upgrades: default_upgrades(),
            total_clicks: 0,
            log: VecDeque::new(),
            casino_open: false,
            ticks_since_save: 0,
            cps_remainder: 0.0,
            ascend_available: false,
            ascension_count: 0,
            golden_collect_window: 0,
            golden_cookie_bonus: 0.0,
            golden_spawn_cooldown: 100, // first golden cookie after ~25 s
            frenzy_ticks: 0,
            frenzy_multiplier: 1.0,
            animation_tick: 0,
            click_animation: 0,
            active_tab: 1,
            upgrade_page: 0,
        }
    }
}

impl GameState {
    /// Global CPS multiplier from heavenly chips: 1 + 0.01 * chips.
    pub fn heavenly_multiplier(&self) -> f64 {
        1.0 + 0.01 * self.heavenly_chips as f64
    }

    /// Multiplicative CPS bonus for a specific building from purchased upgrades.
    pub fn building_cps_multiplier(&self, index: usize) -> f64 {
        // product() on an empty iterator returns 1.0 (multiplicative identity for f64)
        self.upgrades
            .iter()
            .filter(|u| u.purchased && u.building_index == index)
            .map(|u| u.multiplier)
            .product::<f64>()
    }

    /// Total cookies generated per second (respects upgrade multipliers).
    pub fn total_cps(&self) -> f64 {
        let raw: f64 = self
            .buildings
            .iter()
            .enumerate()
            .map(|(i, b)| b.base_cps * b.owned as f64 * self.building_cps_multiplier(i))
            .sum();
        raw * self.heavenly_multiplier() * self.frenzy_multiplier
    }

    /// Cookies earned per manual click: 1 + 0.5 per owned Cursor.
    pub fn click_power(&self) -> f64 {
        let cursors = self.buildings.first().map(|b| b.owned).unwrap_or(0);
        1.0 + cursors as f64 * 0.5
    }

    /// Advance the game by one tick (called every ~250 ms).
    pub fn tick(&mut self, tick_rate_seconds: f64, rng: &mut impl Rng) {
        let gained = self.total_cps() * tick_rate_seconds;
        self.cookies += gained;
        self.total_baked += gained;

        self.ascend_available = self.total_baked >= ASCENSION_THRESHOLD;

        self.ticks_since_save += 1;
        self.animation_tick = self.animation_tick.wrapping_add(1);
        if self.click_animation > 0 {
            self.click_animation -= 1;
        }

        // ── Frenzy countdown ─────────────────────────────────────────────
        if self.frenzy_ticks > 0 {
            self.frenzy_ticks -= 1;
            if self.frenzy_ticks == 0 {
                self.frenzy_multiplier = 1.0;
                self.push_log("🔥 Frenzy ended!".to_string());
            }
        }

        // ── Golden Cookie logic ──────────────────────────────────────────
        if self.golden_collect_window > 0 {
            self.golden_collect_window -= 1;
            if self.golden_collect_window == 0 {
                self.push_log("✨ Golden Cookie vanished... too slow!".to_string());
                // Random cooldown: 300–800 ticks (75–200 s at 250 ms/tick)
                self.golden_spawn_cooldown = rng.gen_range(300..=800);
            }
        } else if self.golden_spawn_cooldown > 0 {
            self.golden_spawn_cooldown -= 1;
        } else {
            // Spawn a new golden cookie – randomly choose effect
            if rng.gen_bool(GOLDEN_FRENZY_PROBABILITY) {
                // Bonus cookies
                self.golden_cookie_bonus = (500.0 + self.cookies * 0.05).max(500.0);
            } else {
                // Frenzy mode (signalled by negative bonus)
                self.golden_cookie_bonus = -1.0;
            }
            self.golden_collect_window = 60; // 15-second window
            self.push_log("✨ A Golden Cookie appeared! Press [C] to collect!".to_string());
            // The next cooldown will be set when this one expires or is collected
        }
    }

    /// Player manually clicks to mine cookies.
    pub fn mine_cookie(&mut self) {
        let power = self.click_power();
        self.cookies += power;
        self.total_baked += power;
        self.total_clicks += 1;
        self.click_animation = 3;
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

    /// Attempt to purchase the upgrade at `index` (0-based).
    /// Returns true if successful.
    pub fn buy_upgrade(&mut self, index: usize) -> bool {
        if index >= self.upgrades.len() {
            return false;
        }
        if self.upgrades[index].purchased {
            self.push_log(format!(
                "{} is already purchased!",
                self.upgrades[index].name.clone()
            ));
            return false;
        }
        let cost = self.upgrades[index].cost;
        if self.cookies >= cost {
            self.cookies -= cost;
            self.upgrades[index].purchased = true;
            let name = self.upgrades[index].name.clone();
            self.push_log(format!("Upgrade '{}' purchased for {:.0} cookies!", name, cost));
            true
        } else {
            false
        }
    }

    /// Collect the active golden cookie, if any.  Returns true if collected.
    pub fn collect_golden_cookie(&mut self) -> bool {
        if self.golden_collect_window > 0 {
            self.golden_collect_window = 0;
            self.golden_spawn_cooldown = 400; // 100 s until next

            if self.golden_cookie_bonus > 0.0 {
                // Bonus cookies
                let bonus = self.golden_cookie_bonus;
                self.add_cookies(bonus);
                self.push_log(format!("✨ Golden Cookie collected! +{:.0} cookies!", bonus));
            } else {
                // Frenzy mode: 7x CPS for 120 ticks (30 seconds at 250ms/tick)
                self.frenzy_multiplier = 7.0;
                self.frenzy_ticks = FRENZY_DURATION_TICKS;
                self.push_log("🔥 Frenzy activated! 7x CPS for 30 seconds!".to_string());
            }
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
        // Reset progress (keep heavenly chips & ascension_count)
        self.cookies = 0.0;
        self.total_baked = 0.0;
        self.buildings = default_buildings();
        self.upgrades = default_upgrades();
        self.total_clicks = 0;
        self.ascend_available = false;
        self.cps_remainder = 0.0;
        self.golden_collect_window = 0;
        self.golden_spawn_cooldown = 100;
        self.frenzy_ticks = 0;
        self.frenzy_multiplier = 1.0;
        self.push_log(msg);
    }

    /// Add a message to the rolling log (capped at MAX_LOG_ENTRIES).
    pub fn push_log(&mut self, msg: String) {
        if self.log.len() >= MAX_LOG_ENTRIES {
            self.log.pop_front();
        }
        self.log.push_back(msg);
    }

    /// Add cookies directly (e.g., from casino wins or golden cookies).
    pub fn add_cookies(&mut self, amount: f64) {
        self.cookies += amount;
        self.total_baked += amount;
        self.ascend_available = self.total_baked >= ASCENSION_THRESHOLD;
    }

    /// Remove cookies (e.g., casino bets). Returns false if insufficient.
    pub fn spend_cookies(&mut self, amount: f64) -> bool {
        if self.cookies >= amount {
            self.cookies -= amount;
            // Guard against tiny floating-point negative due to imprecision
            if self.cookies < 0.0 {
                self.cookies = 0.0;
            }
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
        Building {
            name: "Factory".to_string(),
            base_cost: 130_000.0,
            base_cps: 260.0,
            owned: 0,
        },
        Building {
            name: "Bank".to_string(),
            base_cost: 1_400_000.0,
            base_cps: 1_400.0,
            owned: 0,
        },
        Building {
            name: "Temple".to_string(),
            base_cost: 20_000_000.0,
            base_cps: 7_800.0,
            owned: 0,
        },
        Building {
            name: "Wizard Tower".to_string(),
            base_cost: 330_000_000.0,
            base_cps: 44_000.0,
            owned: 0,
        },
    ]
}

fn default_upgrades() -> Vec<Upgrade> {
    vec![
        // ── Cursor upgrades (building_index = 0) ──────────────────────────
        Upgrade {
            name: "Nimble Fingers".to_string(),
            description: "Cursors are 2x more effective.".to_string(),
            cost: 100.0,
            purchased: false,
            building_index: 0,
            multiplier: 2.0,
        },
        Upgrade {
            name: "Cursor Overdrive".to_string(),
            description: "Cursors are 2x more effective (stacks).".to_string(),
            cost: 2_000.0,
            purchased: false,
            building_index: 0,
            multiplier: 2.0,
        },
        // ── Grandma upgrades (building_index = 1) ─────────────────────────
        Upgrade {
            name: "Loving Grandmas".to_string(),
            description: "Grandmas bake 2x more cookies.".to_string(),
            cost: 1_000.0,
            purchased: false,
            building_index: 1,
            multiplier: 2.0,
        },
        Upgrade {
            name: "Senior Discount".to_string(),
            description: "Grandmas bake 2x more cookies (stacks).".to_string(),
            cost: 20_000.0,
            purchased: false,
            building_index: 1,
            multiplier: 2.0,
        },
        // ── Farm upgrades (building_index = 2) ────────────────────────────
        Upgrade {
            name: "Irrigation System".to_string(),
            description: "Farms produce 2x more cookies.".to_string(),
            cost: 11_000.0,
            purchased: false,
            building_index: 2,
            multiplier: 2.0,
        },
        Upgrade {
            name: "Hydroponics".to_string(),
            description: "Farms produce 2x more cookies (stacks).".to_string(),
            cost: 110_000.0,
            purchased: false,
            building_index: 2,
            multiplier: 2.0,
        },
        // ── Mine upgrades (building_index = 3) ────────────────────────────
        Upgrade {
            name: "Deep Excavation".to_string(),
            description: "Mines yield 2x more cookies.".to_string(),
            cost: 130_000.0,
            purchased: false,
            building_index: 3,
            multiplier: 2.0,
        },
        Upgrade {
            name: "Quantum Drilling".to_string(),
            description: "Mines yield 2x more cookies (stacks).".to_string(),
            cost: 1_300_000.0,
            purchased: false,
            building_index: 3,
            multiplier: 2.0,
        },
        // ── Factory upgrades (building_index = 4) ─────────────────────────
        Upgrade {
            name: "Assembly Line".to_string(),
            description: "Factories produce 2x more cookies.".to_string(),
            cost: 1_300_000.0,
            purchased: false,
            building_index: 4,
            multiplier: 2.0,
        },
        Upgrade {
            name: "Mass Production".to_string(),
            description: "Factories produce 2x more cookies (stacks).".to_string(),
            cost: 13_000_000.0,
            purchased: false,
            building_index: 4,
            multiplier: 2.0,
        },
        // ── Bank upgrades (building_index = 5) ────────────────────────────
        Upgrade {
            name: "Investment Portfolio".to_string(),
            description: "Banks generate 2x more cookies.".to_string(),
            cost: 14_000_000.0,
            purchased: false,
            building_index: 5,
            multiplier: 2.0,
        },
        Upgrade {
            name: "Compound Interest".to_string(),
            description: "Banks generate 2x more cookies (stacks).".to_string(),
            cost: 140_000_000.0,
            purchased: false,
            building_index: 5,
            multiplier: 2.0,
        },
        // ── Temple upgrades (building_index = 6) ──────────────────────────
        Upgrade {
            name: "Divine Blessing".to_string(),
            description: "Temples produce 2x more cookies.".to_string(),
            cost: 200_000_000.0,
            purchased: false,
            building_index: 6,
            multiplier: 2.0,
        },
        Upgrade {
            name: "Sacred Ritual".to_string(),
            description: "Temples produce 2x more cookies (stacks).".to_string(),
            cost: 2_000_000_000.0,
            purchased: false,
            building_index: 6,
            multiplier: 2.0,
        },
        // ── Wizard Tower upgrades (building_index = 7) ────────────────────
        Upgrade {
            name: "Arcane Enchant".to_string(),
            description: "Wizard Towers produce 2x more cookies.".to_string(),
            cost: 3_300_000_000.0,
            purchased: false,
            building_index: 7,
            multiplier: 2.0,
        },
        Upgrade {
            name: "Conjure Cookies".to_string(),
            description: "Wizard Towers produce 2x more cookies (stacks).".to_string(),
            cost: 33_000_000_000.0,
            purchased: false,
            building_index: 7,
            multiplier: 2.0,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    fn test_rng() -> SmallRng {
        SmallRng::seed_from_u64(42)
    }

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
        assert_eq!(gs.total_clicks, 1);
    }

    #[test]
    fn test_click_power_no_cursors() {
        let gs = GameState::default();
        assert!((gs.click_power() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_click_power_with_cursors() {
        let mut gs = GameState::default();
        gs.buildings[0].owned = 4; // 1 + 4 * 0.5 = 3.0
        assert!((gs.click_power() - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_mine_cookie_uses_click_power() {
        let mut gs = GameState::default();
        gs.buildings[0].owned = 2; // click power = 1 + 2*0.5 = 2.0
        gs.mine_cookie();
        assert!((gs.cookies - 2.0).abs() < 0.001);
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
    fn test_building_cps_multiplier_no_upgrades() {
        let gs = GameState::default();
        assert!((gs.building_cps_multiplier(0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_building_cps_multiplier_with_one_upgrade() {
        let mut gs = GameState::default();
        gs.upgrades[0].purchased = true; // Nimble Fingers: 2x cursor
        assert!((gs.building_cps_multiplier(0) - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_building_cps_multiplier_stacks() {
        let mut gs = GameState::default();
        gs.upgrades[0].purchased = true; // 2x
        gs.upgrades[1].purchased = true; // 2x again → 4x total
        assert!((gs.building_cps_multiplier(0) - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_total_cps_with_upgrade() {
        let mut gs = GameState::default();
        gs.buildings[0].owned = 10; // 10 cursors * 0.1 = 1.0 raw CPS
        gs.upgrades[0].purchased = true; // 2x cursor → 2.0
        // heavenly_mult = 1.0 (no chips)
        assert!((gs.total_cps() - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_buy_upgrade_success() {
        let mut gs = GameState::default();
        gs.cookies = 200.0;
        let result = gs.buy_upgrade(0); // Nimble Fingers costs 100
        assert!(result);
        assert!(gs.upgrades[0].purchased);
        assert!((gs.cookies - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_buy_upgrade_insufficient_cookies() {
        let mut gs = GameState::default();
        gs.cookies = 50.0;
        let result = gs.buy_upgrade(0); // costs 100
        assert!(!result);
        assert!(!gs.upgrades[0].purchased);
    }

    #[test]
    fn test_buy_upgrade_already_purchased() {
        let mut gs = GameState::default();
        gs.cookies = 10_000.0;
        gs.buy_upgrade(0);
        let result = gs.buy_upgrade(0); // try again
        assert!(!result);
        assert!((gs.cookies - 9_900.0).abs() < 0.001); // only paid once
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
        assert_eq!(gs.total_clicks, 0); // reset on ascend
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
        let mut rng = test_rng();
        gs.buildings[1].owned = 1; // 1 Grandma = 1.0 CPS
        gs.golden_spawn_cooldown = 999; // suppress golden cookie during test
        gs.tick(1.0, &mut rng); // 1 second tick
        assert!((gs.cookies - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_collect_golden_cookie() {
        let mut gs = GameState::default();
        gs.cookies = 1_000.0;
        gs.golden_collect_window = 30;
        gs.golden_cookie_bonus = 500.0;
        let result = gs.collect_golden_cookie();
        assert!(result);
        assert!((gs.cookies - 1_500.0).abs() < 0.001);
        assert_eq!(gs.golden_collect_window, 0);
        assert_eq!(gs.golden_spawn_cooldown, 400);
    }

    #[test]
    fn test_collect_golden_cookie_when_none() {
        let mut gs = GameState::default();
        gs.golden_collect_window = 0;
        let result = gs.collect_golden_cookie();
        assert!(!result);
    }

    #[test]
    fn test_new_buildings_exist() {
        let gs = GameState::default();
        assert_eq!(gs.buildings.len(), 8);
        assert_eq!(gs.buildings[4].name, "Factory");
        assert_eq!(gs.buildings[5].name, "Bank");
        assert_eq!(gs.buildings[6].name, "Temple");
        assert_eq!(gs.buildings[7].name, "Wizard Tower");
    }

    #[test]
    fn test_new_upgrades_exist() {
        let gs = GameState::default();
        assert_eq!(gs.upgrades.len(), 16);
        assert_eq!(gs.upgrades[8].name, "Assembly Line");
        assert_eq!(gs.upgrades[9].name, "Mass Production");
        assert_eq!(gs.upgrades[10].name, "Investment Portfolio");
        assert_eq!(gs.upgrades[11].name, "Compound Interest");
        assert_eq!(gs.upgrades[12].name, "Divine Blessing");
        assert_eq!(gs.upgrades[13].name, "Sacred Ritual");
        assert_eq!(gs.upgrades[14].name, "Arcane Enchant");
        assert_eq!(gs.upgrades[15].name, "Conjure Cookies");
    }

    #[test]
    fn test_frenzy_multiplier() {
        let mut gs = GameState::default();
        gs.buildings[1].owned = 10; // 10 Grandmas = 10.0 raw CPS
        let normal_cps = gs.total_cps();
        gs.frenzy_multiplier = 7.0;
        gs.frenzy_ticks = FRENZY_DURATION_TICKS;
        let frenzy_cps = gs.total_cps();
        assert!((frenzy_cps - normal_cps * 7.0).abs() < 0.001);
    }

    #[test]
    fn test_factory_building_cost() {
        let gs = GameState::default();
        assert!((gs.buildings[4].next_cost() - 130_000.0).abs() < 0.001);
    }

    #[test]
    fn test_spend_cookies_clamps_to_zero() {
        let mut gs = GameState::default();
        gs.cookies = 100.0;
        assert!(gs.spend_cookies(100.0));
        assert!(gs.cookies >= 0.0, "Cookies should never go negative");
    }

    #[test]
    fn test_spend_cookies_insufficient() {
        let mut gs = GameState::default();
        gs.cookies = 50.0;
        assert!(!gs.spend_cookies(100.0));
        assert!((gs.cookies - 50.0).abs() < 0.001, "Cookies unchanged on failed spend");
    }
}
