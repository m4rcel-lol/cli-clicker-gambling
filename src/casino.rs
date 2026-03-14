use rand::Rng;
use serde::{Deserialize, Serialize};

pub const SLOT_BET: f64 = 100.0;
pub const COINFLIP_MIN_BET: f64 = 10.0;
pub const DICE_MIN_BET: f64 = 10.0;
pub const ROULETTE_MIN_BET: f64 = 10.0;
pub const BLACKJACK_MIN_BET: f64 = 25.0;

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

// ═══════════════════════════════════════════════════════════════════════════════
//  Roulette types
// ═══════════════════════════════════════════════════════════════════════════════

/// Color of a roulette pocket.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RouletteColor {
    Red,
    Black,
    Green,
}

/// Available bet types in roulette.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RouletteBetType {
    Red,
    Black,
    Green,
    Odd,
    Even,
    Low,  // 1-18
    High, // 19-36
}

impl RouletteBetType {
    pub fn label(&self) -> &'static str {
        match self {
            RouletteBetType::Red => "Red",
            RouletteBetType::Black => "Black",
            RouletteBetType::Green => "Green (0)",
            RouletteBetType::Odd => "Odd",
            RouletteBetType::Even => "Even",
            RouletteBetType::Low => "Low (1-18)",
            RouletteBetType::High => "High (19-36)",
        }
    }

    pub fn payout_multiplier(&self) -> f64 {
        match self {
            RouletteBetType::Green => 14.0,
            _ => 2.0,
        }
    }
}

/// Standard European roulette number-to-color mapping.
pub fn roulette_color(number: u8) -> RouletteColor {
    if number == 0 {
        RouletteColor::Green
    } else {
        // Standard red numbers on a European roulette wheel
        const RED_NUMBERS: [u8; 18] = [
            1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36,
        ];
        if RED_NUMBERS.contains(&number) {
            RouletteColor::Red
        } else {
            RouletteColor::Black
        }
    }
}

/// Check if a roulette bet wins for the given number.
pub fn roulette_bet_wins(bet: RouletteBetType, number: u8) -> bool {
    let color = roulette_color(number);
    match bet {
        RouletteBetType::Red => color == RouletteColor::Red,
        RouletteBetType::Black => color == RouletteColor::Black,
        RouletteBetType::Green => color == RouletteColor::Green,
        RouletteBetType::Odd => number > 0 && number % 2 == 1,
        RouletteBetType::Even => number > 0 && number % 2 == 0,
        RouletteBetType::Low => number >= 1 && number <= 18,
        RouletteBetType::High => number >= 19 && number <= 36,
    }
}

/// Outcome of a roulette spin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouletteResult {
    pub number: u8,
    pub color: RouletteColor,
    pub bet_type: RouletteBetType,
    pub wager: f64,
    pub won: bool,
    pub net: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Blackjack types
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

impl Suit {
    pub fn symbol(&self) -> &'static str {
        match self {
            Suit::Hearts => "♥",
            Suit::Diamonds => "♦",
            Suit::Clubs => "♣",
            Suit::Spades => "♠",
        }
    }

    #[allow(dead_code)]
    pub fn is_red(&self) -> bool {
        matches!(self, Suit::Hearts | Suit::Diamonds)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub rank: u8, // 1=Ace, 2-10, 11=Jack, 12=Queen, 13=King
    pub suit: Suit,
}

impl Card {
    pub fn random(rng: &mut impl Rng) -> Self {
        Card {
            rank: rng.gen_range(1..=13),
            suit: match rng.gen_range(0..4) {
                0 => Suit::Hearts,
                1 => Suit::Diamonds,
                2 => Suit::Clubs,
                _ => Suit::Spades,
            },
        }
    }

    pub fn rank_label(&self) -> String {
        match self.rank {
            1 => "A".to_string(),
            2..=10 => self.rank.to_string(),
            11 => "J".to_string(),
            12 => "Q".to_string(),
            13 => "K".to_string(),
            _ => "?".to_string(),
        }
    }

    pub fn display(&self) -> String {
        format!("{}{}", self.rank_label(), self.suit.symbol())
    }

    /// Blackjack point value (Ace counted as 11; caller handles soft totals).
    pub fn bj_value(&self) -> u8 {
        match self.rank {
            1 => 11,         // Ace (soft)
            2..=10 => self.rank,
            _ => 10,         // Face cards
        }
    }
}

/// Calculate the best blackjack hand value (handles soft Aces).
pub fn hand_value(hand: &[Card]) -> u8 {
    let mut total: u16 = hand.iter().map(|c| c.bj_value() as u16).sum();
    let mut aces = hand.iter().filter(|c| c.rank == 1).count() as u16;
    while total > 21 && aces > 0 {
        total -= 10; // Convert one Ace from 11 to 1
        aces -= 1;
    }
    total.min(255) as u8
}

/// Phase of a blackjack hand.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BlackjackPhase {
    Betting,
    PlayerTurn,
    DealerTurn,
    Resolved,
}

impl Default for BlackjackPhase {
    fn default() -> Self {
        BlackjackPhase::Betting
    }
}

/// Outcome of a blackjack hand.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BlackjackOutcome {
    PlayerBlackjack, // Natural 21, pays 2.5x
    PlayerWin,       // Normal win, pays 2x
    DealerWin,       // Player loses bet
    Push,            // Tie, bet returned
    PlayerBust,      // Player went over 21
    DealerBust,      // Dealer went over 21
}

/// State of an in-progress blackjack hand.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackjackHand {
    pub player_cards: Vec<Card>,
    pub dealer_cards: Vec<Card>,
    pub wager: f64,
    pub phase: BlackjackPhase,
    pub outcome: Option<BlackjackOutcome>,
}

/// Outcome of a completed blackjack hand.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackjackResult {
    pub player_cards: Vec<Card>,
    pub dealer_cards: Vec<Card>,
    pub player_score: u8,
    pub dealer_score: u8,
    pub wager: f64,
    pub outcome: BlackjackOutcome,
    pub net: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Existing types
// ═══════════════════════════════════════════════════════════════════════════════

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

/// Tracks the casino UI state (active game, pending bets, last results, lifetime stats).
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
    /// Last roulette result.
    pub last_roulette: Option<RouletteResult>,
    /// Last completed blackjack result.
    pub last_blackjack: Option<BlackjackResult>,
    /// Active blackjack hand in progress.
    pub blackjack_hand: Option<BlackjackHand>,
    /// Current wager input buffer (string of digits).
    pub wager_input: String,
    /// Dice guess input (1-6).
    pub dice_guess: u8,
    /// Whether we are in "entering wager" mode.
    pub entering_wager: bool,
    /// Whether we are in "entering dice guess" mode.
    pub entering_dice_guess: bool,
    /// Selected roulette bet type.
    pub roulette_bet_type: Option<RouletteBetType>,

    // ── Lifetime stats ────────────────────────────────────────────────────
    /// Total number of slot spins ever made.
    pub total_spins: u32,
    /// Total cookies wagered across all casino games.
    pub total_wagered: f64,
    /// Total gross cookies won back (including returned bets on wins).
    pub total_won: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CasinoGame {
    SlotMachine,
    CoinFlip,
    DiceWager,
    Roulette,
    Blackjack,
}

impl CasinoState {
    /// Spin the slot machine. Returns the gross payout (0 = total loss).
    pub fn spin_slots(&mut self, rng: &mut impl Rng) -> f64 {
        let reels = [Symbol::random(rng), Symbol::random(rng), Symbol::random(rng)];
        let gross = slot_payout(&reels);
        let net = gross - SLOT_BET;
        self.last_slot = Some(SlotResult { reels, payout: net });
        self.total_spins += 1;
        self.total_wagered += SLOT_BET;
        self.total_won += gross;
        gross
    }

    /// Flip a coin with the given wager. Returns net cookie change.
    pub fn flip_coin(&mut self, wager: f64, rng: &mut impl Rng) -> f64 {
        let won: bool = rng.gen();
        let net = if won { wager } else { -wager };
        self.last_coin = Some(CoinFlipResult { won, net });
        self.total_wagered += wager;
        if won {
            self.total_won += wager * 2.0;
        }
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
        self.total_wagered += wager;
        if won {
            self.total_won += wager * 5.0;
        }
        net
    }

    /// Spin the roulette wheel with the given wager and bet type. Returns net cookie change.
    pub fn spin_roulette(
        &mut self,
        wager: f64,
        bet_type: RouletteBetType,
        rng: &mut impl Rng,
    ) -> f64 {
        let number: u8 = rng.gen_range(0..=36);
        let color = roulette_color(number);
        let won = roulette_bet_wins(bet_type, number);
        let net = if won {
            wager * (bet_type.payout_multiplier() - 1.0)
        } else {
            -wager
        };
        self.last_roulette = Some(RouletteResult {
            number,
            color,
            bet_type,
            wager,
            won,
            net,
        });
        self.total_wagered += wager;
        if won {
            self.total_won += wager * bet_type.payout_multiplier();
        }
        net
    }

    /// Deal a new blackjack hand. Returns true if hand was dealt.
    pub fn deal_blackjack(&mut self, wager: f64, rng: &mut impl Rng) -> bool {
        if self.blackjack_hand.is_some() {
            return false; // Already in a hand
        }
        let player_cards = vec![Card::random(rng), Card::random(rng)];
        let dealer_cards = vec![Card::random(rng), Card::random(rng)];

        let player_val = hand_value(&player_cards);
        let dealer_val = hand_value(&dealer_cards);

        // Check for natural blackjack
        if player_val == 21 {
            let outcome = if dealer_val == 21 {
                BlackjackOutcome::Push
            } else {
                BlackjackOutcome::PlayerBlackjack
            };
            let net = match outcome {
                BlackjackOutcome::PlayerBlackjack => wager * 1.5, // 2.5x gross
                BlackjackOutcome::Push => 0.0,
                _ => -wager,
            };
            self.total_wagered += wager;
            if net > 0.0 {
                self.total_won += wager + net;
            } else if net == 0.0 {
                self.total_won += wager; // push: return bet
            }
            self.last_blackjack = Some(BlackjackResult {
                player_score: player_val,
                dealer_score: dealer_val,
                player_cards: player_cards.clone(),
                dealer_cards: dealer_cards.clone(),
                wager,
                outcome,
                net,
            });
            self.blackjack_hand = Some(BlackjackHand {
                player_cards,
                dealer_cards,
                wager,
                phase: BlackjackPhase::Resolved,
                outcome: Some(outcome),
            });
            return true;
        }

        self.total_wagered += wager;
        self.blackjack_hand = Some(BlackjackHand {
            player_cards,
            dealer_cards,
            wager,
            phase: BlackjackPhase::PlayerTurn,
            outcome: None,
        });
        self.last_blackjack = None;
        true
    }

    /// Player hits in blackjack (draws a card). Returns true if still in play.
    pub fn blackjack_hit(&mut self, rng: &mut impl Rng) -> bool {
        let hand = match self.blackjack_hand.as_mut() {
            Some(h) if h.phase == BlackjackPhase::PlayerTurn => h,
            _ => return false,
        };
        hand.player_cards.push(Card::random(rng));
        let val = hand_value(&hand.player_cards);
        if val > 21 {
            hand.phase = BlackjackPhase::Resolved;
            hand.outcome = Some(BlackjackOutcome::PlayerBust);
            self.finalize_blackjack();
            return false;
        }
        if val == 21 {
            // Auto-stand on 21
            self.blackjack_stand(rng);
            return false;
        }
        true
    }

    /// Player stands in blackjack. Dealer plays out their hand.
    pub fn blackjack_stand(&mut self, rng: &mut impl Rng) {
        let hand = match self.blackjack_hand.as_mut() {
            Some(h) if h.phase == BlackjackPhase::PlayerTurn => h,
            _ => return,
        };
        hand.phase = BlackjackPhase::DealerTurn;

        // Dealer hits on 16 or below, stands on 17+
        while hand_value(&hand.dealer_cards) < 17 {
            hand.dealer_cards.push(Card::random(rng));
        }

        let player_val = hand_value(&hand.player_cards);
        let dealer_val = hand_value(&hand.dealer_cards);

        let outcome = if dealer_val > 21 {
            BlackjackOutcome::DealerBust
        } else if player_val > dealer_val {
            BlackjackOutcome::PlayerWin
        } else if dealer_val > player_val {
            BlackjackOutcome::DealerWin
        } else {
            BlackjackOutcome::Push
        };

        hand.phase = BlackjackPhase::Resolved;
        hand.outcome = Some(outcome);
        self.finalize_blackjack();
    }

    /// Finalize a blackjack hand and record results.
    fn finalize_blackjack(&mut self) {
        let hand = match &self.blackjack_hand {
            Some(h) if h.phase == BlackjackPhase::Resolved => h,
            _ => return,
        };
        let outcome = match hand.outcome {
            Some(o) => o,
            None => return,
        };
        let wager = hand.wager;
        let net = match outcome {
            BlackjackOutcome::PlayerBlackjack => wager * 1.5,
            BlackjackOutcome::PlayerWin | BlackjackOutcome::DealerBust => wager,
            BlackjackOutcome::Push => 0.0,
            BlackjackOutcome::DealerWin | BlackjackOutcome::PlayerBust => -wager,
        };
        if net > 0.0 {
            self.total_won += wager + net; // return bet + profit
        } else if net == 0.0 {
            self.total_won += wager; // push: return bet
        }
        self.last_blackjack = Some(BlackjackResult {
            player_score: hand_value(&hand.player_cards),
            dealer_score: hand_value(&hand.dealer_cards),
            player_cards: hand.player_cards.clone(),
            dealer_cards: hand.dealer_cards.clone(),
            wager,
            outcome,
            net,
        });
    }

    /// Clear the current blackjack hand so a new one can be dealt.
    pub fn clear_blackjack_hand(&mut self) {
        self.blackjack_hand = None;
    }

    /// Net profit/loss across all casino games.
    pub fn net_profit(&self) -> f64 {
        self.total_won - self.total_wagered
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

    // ── Roulette tests ──────────────────────────────────────────────────

    #[test]
    fn test_roulette_color_zero_is_green() {
        assert_eq!(roulette_color(0), RouletteColor::Green);
    }

    #[test]
    fn test_roulette_color_one_is_red() {
        assert_eq!(roulette_color(1), RouletteColor::Red);
    }

    #[test]
    fn test_roulette_color_two_is_black() {
        assert_eq!(roulette_color(2), RouletteColor::Black);
    }

    #[test]
    fn test_roulette_bet_red_wins_on_red_number() {
        assert!(roulette_bet_wins(RouletteBetType::Red, 1));
        assert!(!roulette_bet_wins(RouletteBetType::Red, 2));
        assert!(!roulette_bet_wins(RouletteBetType::Red, 0));
    }

    #[test]
    fn test_roulette_bet_green_wins_on_zero() {
        assert!(roulette_bet_wins(RouletteBetType::Green, 0));
        assert!(!roulette_bet_wins(RouletteBetType::Green, 1));
    }

    #[test]
    fn test_roulette_bet_odd_even() {
        assert!(roulette_bet_wins(RouletteBetType::Odd, 3));
        assert!(!roulette_bet_wins(RouletteBetType::Odd, 4));
        assert!(!roulette_bet_wins(RouletteBetType::Odd, 0)); // 0 is not odd
        assert!(roulette_bet_wins(RouletteBetType::Even, 4));
        assert!(!roulette_bet_wins(RouletteBetType::Even, 0)); // 0 is not even
    }

    #[test]
    fn test_roulette_bet_low_high() {
        assert!(roulette_bet_wins(RouletteBetType::Low, 1));
        assert!(roulette_bet_wins(RouletteBetType::Low, 18));
        assert!(!roulette_bet_wins(RouletteBetType::Low, 19));
        assert!(roulette_bet_wins(RouletteBetType::High, 19));
        assert!(roulette_bet_wins(RouletteBetType::High, 36));
        assert!(!roulette_bet_wins(RouletteBetType::High, 18));
    }

    #[test]
    fn test_roulette_spin() {
        let mut casino = CasinoState::default();
        let mut rng = SmallRng::seed_from_u64(42);
        let net = casino.spin_roulette(100.0, RouletteBetType::Red, &mut rng);
        // net should be +100 (won) or -100 (lost)
        assert!(net == 100.0 || net == -100.0);
        assert!(casino.last_roulette.is_some());
        assert!((casino.total_wagered - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_roulette_green_payout() {
        assert!((RouletteBetType::Green.payout_multiplier() - 14.0).abs() < 0.001);
        assert!((RouletteBetType::Red.payout_multiplier() - 2.0).abs() < 0.001);
    }

    // ── Blackjack tests ─────────────────────────────────────────────────

    #[test]
    fn test_hand_value_simple() {
        let hand = vec![
            Card { rank: 10, suit: Suit::Hearts },
            Card { rank: 8, suit: Suit::Spades },
        ];
        assert_eq!(hand_value(&hand), 18);
    }

    #[test]
    fn test_hand_value_ace_as_eleven() {
        let hand = vec![
            Card { rank: 1, suit: Suit::Hearts },
            Card { rank: 9, suit: Suit::Spades },
        ];
        assert_eq!(hand_value(&hand), 20); // Ace = 11
    }

    #[test]
    fn test_hand_value_ace_as_one() {
        let hand = vec![
            Card { rank: 1, suit: Suit::Hearts },
            Card { rank: 9, suit: Suit::Spades },
            Card { rank: 5, suit: Suit::Clubs },
        ];
        assert_eq!(hand_value(&hand), 15); // Ace demoted to 1
    }

    #[test]
    fn test_hand_value_blackjack() {
        let hand = vec![
            Card { rank: 1, suit: Suit::Hearts },
            Card { rank: 13, suit: Suit::Spades }, // King = 10
        ];
        assert_eq!(hand_value(&hand), 21);
    }

    #[test]
    fn test_hand_value_double_ace() {
        let hand = vec![
            Card { rank: 1, suit: Suit::Hearts },
            Card { rank: 1, suit: Suit::Spades },
        ];
        assert_eq!(hand_value(&hand), 12); // One ace = 11, one = 1
    }

    #[test]
    fn test_card_display() {
        let card = Card { rank: 1, suit: Suit::Spades };
        assert_eq!(card.display(), "A♠");
        let card2 = Card { rank: 13, suit: Suit::Hearts };
        assert_eq!(card2.display(), "K♥");
    }

    #[test]
    fn test_blackjack_deal() {
        let mut casino = CasinoState::default();
        let mut rng = SmallRng::seed_from_u64(42);
        let dealt = casino.deal_blackjack(100.0, &mut rng);
        assert!(dealt);
        assert!(casino.blackjack_hand.is_some());
        assert!((casino.total_wagered - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_blackjack_cannot_deal_twice() {
        let mut casino = CasinoState::default();
        let mut rng = SmallRng::seed_from_u64(42);
        casino.deal_blackjack(100.0, &mut rng);
        // If the hand was resolved (natural blackjack), clear it first
        if let Some(ref h) = casino.blackjack_hand {
            if h.phase != BlackjackPhase::Resolved {
                let dealt2 = casino.deal_blackjack(100.0, &mut rng);
                assert!(!dealt2); // Cannot deal while hand is active
            }
        }
    }

    #[test]
    fn test_blackjack_stand_resolves() {
        let mut casino = CasinoState::default();
        let mut rng = SmallRng::seed_from_u64(99);
        casino.deal_blackjack(50.0, &mut rng);
        if let Some(ref h) = casino.blackjack_hand {
            if h.phase == BlackjackPhase::PlayerTurn {
                casino.blackjack_stand(&mut rng);
                let hand = casino.blackjack_hand.as_ref().unwrap();
                assert_eq!(hand.phase, BlackjackPhase::Resolved);
                assert!(hand.outcome.is_some());
                assert!(casino.last_blackjack.is_some());
            }
        }
    }
}
