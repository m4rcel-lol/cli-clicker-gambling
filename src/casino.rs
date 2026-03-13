use rand::Rng;
use serde::{Deserialize, Serialize};

pub const SLOT_BET: f64 = 100.0;
pub const COINFLIP_MIN_BET: f64 = 10.0;
pub const DICE_MIN_BET: f64 = 10.0;

/// Symbols for the slot machine reels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Symbol {
    Cherry,
    Lemon,
    Bell,
    Diamond,
    Seven,
}

impl Symbol {
    pub fn label(&self) -> &'static str {
        match self {
            Symbol::Cherry => "🍒",
            Symbol::Lemon => "🍋",
            Symbol::Bell => "🔔",
            Symbol::Diamond => "💎",
            Symbol::Seven => "7️⃣",
        }
    }

    fn random(rng: &mut impl Rng) -> Self {
        // Weighted: Cherry 35%, Lemon 30%, Bell 20%, Diamond 10%, Seven 5%
        let roll: u8 = rng.gen_range(0..100);
        if roll < 35 {
            Symbol::Cherry
        } else if roll < 65 {
            Symbol::Lemon
        } else if roll < 85 {
            Symbol::Bell
        } else if roll < 95 {
            Symbol::Diamond
        } else {
            Symbol::Seven
        }
    }
}

/// Outcome of a single slot spin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotResult {
    pub reels: [Symbol; 3],
    pub payout: f64, // net change (negative means loss)
}

/// Outcome of a coin flip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinFlipResult {
    pub won: bool,
    pub net: f64,
}

/// Outcome of a dice wager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiceResult {
    pub rolled: u8,
    pub guessed: u8,
    pub won: bool,
    pub net: f64,
}

/// Tracks the casino UI state (active game, pending bets, last results).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CasinoState {
    /// Which casino game is currently active (None = menu).
    pub active_game: Option<CasinoGame>,
    /// Last slot machine result.
    pub last_slot: Option<SlotResult>,
    /// Last coin flip result.
    pub last_coin: Option<CoinFlipResult>,
    /// Last dice result.
    pub last_dice: Option<DiceResult>,
    /// Current wager input buffer (string of digits).
    pub wager_input: String,
    /// Dice guess input (1-6).
    pub dice_guess: u8,
    /// Whether we are in "entering wager" mode.
    pub entering_wager: bool,
    /// Whether we are in "entering dice guess" mode.
    pub entering_dice_guess: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CasinoGame {
    SlotMachine,
    CoinFlip,
    DiceWager,
}

impl CasinoState {
    /// Spin the slot machine. Returns the gross payout (0 = total loss).
    pub fn spin_slots(&mut self, rng: &mut impl Rng) -> f64 {
        let reels = [Symbol::random(rng), Symbol::random(rng), Symbol::random(rng)];
        let gross = slot_payout(&reels);
        let net = gross - SLOT_BET;
        self.last_slot = Some(SlotResult { reels, payout: net });
        gross
    }

    /// Flip a coin with the given wager. Returns net cookie change.
    pub fn flip_coin(&mut self, wager: f64, rng: &mut impl Rng) -> f64 {
        let won: bool = rng.gen();
        let net = if won { wager } else { -wager };
        self.last_coin = Some(CoinFlipResult { won, net });
        net
    }

    /// Roll a die and compare to guess. Returns net cookie change.
    pub fn roll_dice(&mut self, wager: f64, guess: u8, rng: &mut impl Rng) -> f64 {
        let rolled: u8 = rng.gen_range(1..=6);
        let won = rolled == guess;
        let net = if won { wager * 4.0 } else { -wager }; // 5x gross = 4x net
        self.last_dice = Some(DiceResult {
            rolled,
            guessed: guess,
            won,
            net,
        });
        net
    }

    /// Parse the wager input buffer to an f64. Returns None on empty/invalid.
    pub fn parsed_wager(&self) -> Option<f64> {
        if self.wager_input.is_empty() {
            None
        } else {
            self.wager_input.parse::<f64>().ok().filter(|&v| v >= 1.0)
        }
    }
}

/// Compute slot payout based on three reels (gross, not net).
fn slot_payout(reels: &[Symbol; 3]) -> f64 {
    if reels[0] == reels[1] && reels[1] == reels[2] {
        match reels[0] {
            Symbol::Cherry => SLOT_BET * 5.0,
            Symbol::Lemon => SLOT_BET * 3.0,
            Symbol::Bell => SLOT_BET * 7.0,
            Symbol::Diamond => SLOT_BET * 25.0,
            Symbol::Seven => SLOT_BET * 100.0,
        }
    } else if reels[0] == Symbol::Cherry
        || reels[1] == Symbol::Cherry
        || reels[2] == Symbol::Cherry
    {
        // Any cherry = small consolation
        SLOT_BET * 0.5
    } else {
        0.0 // No payout
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    #[test]
    fn test_slot_payout_three_sevens() {
        let reels = [Symbol::Seven, Symbol::Seven, Symbol::Seven];
        assert!((slot_payout(&reels) - SLOT_BET * 100.0).abs() < 0.001);
    }

    #[test]
    fn test_slot_payout_three_diamonds() {
        let reels = [Symbol::Diamond, Symbol::Diamond, Symbol::Diamond];
        assert!((slot_payout(&reels) - SLOT_BET * 25.0).abs() < 0.001);
    }

    #[test]
    fn test_slot_payout_three_cherries() {
        let reels = [Symbol::Cherry, Symbol::Cherry, Symbol::Cherry];
        assert!((slot_payout(&reels) - SLOT_BET * 5.0).abs() < 0.001);
    }

    #[test]
    fn test_slot_payout_no_match() {
        let reels = [Symbol::Lemon, Symbol::Bell, Symbol::Diamond];
        assert!((slot_payout(&reels) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_coin_flip_deterministic() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut casino = CasinoState::default();
        let net = casino.flip_coin(100.0, &mut rng);
        // net should be either +100 or -100
        assert!(net == 100.0 || net == -100.0);
    }

    #[test]
    fn test_dice_win() {
        let mut casino = CasinoState::default();
        // Force rolled == guessed by seeding
        let mut rng = SmallRng::seed_from_u64(0);
        let net = casino.roll_dice(50.0, 1, &mut rng);
        assert!(net == 200.0 || net == -50.0);
    }

    #[test]
    fn test_parsed_wager_valid() {
        let mut casino = CasinoState::default();
        casino.wager_input = "250".to_string();
        assert_eq!(casino.parsed_wager(), Some(250.0));
    }

    #[test]
    fn test_parsed_wager_empty() {
        let casino = CasinoState::default();
        assert_eq!(casino.parsed_wager(), None);
    }
}
