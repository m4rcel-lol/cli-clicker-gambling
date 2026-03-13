# 🍪 Cookie Clicker CLI

> A Cookie Clicker-style idle game with integrated casino minigames, built entirely in the terminal.
> Targeted at **Arch Linux** users. Safe, addictive, and 100% free. Written in Rust.

```
┌───────────────────────────────┬───────────────────────────────────────────────┐
│ 🍪  THE COOKIE CLI            │ [1] Buildings  [2] Upgrades  [3] Casino       │
│                               ├───────────────────────────────────────────────┤
│ Cookies: 1,402                │ [1] Cursor   (Owned:  12) - Cost: 80          │
│ CPS:     13.2                 │ [2] Grandma  (Owned:   2) - Cost: 132         │
│ Baked:   15,000               │ [3] Farm     (Owned:   0) - Cost: 1,100       │
│ Prestige: 1.5%                │ [4] Mine     (Owned:   0) - Cost: 12,000      │
│       ______                  │                                               │
│      /  🍪  \                 │                                               │
│     /  Clck  \                │                                               │
│    / CLI Game \               │                                               │
│   \____________/              │                                               │
├───────────────────────────────┴───────────────────────────────────────────────┤
│ [Log]: Grandma purchased for 115 cookies.                                     │
│ [Log]: Game saved!                                                            │
├───────────────────────────────────────────────────────────────────────────────┤
│ Hotkeys: [Space/Enter] Mine | [1-4] Buy Building | [G] Casino | [S] Save | [Q] Quit │
└───────────────────────────────────────────────────────────────────────────────┘
```

---

## Elevator Pitch

You are a cookie tycoon. Click to bake. Buy buildings to automate production. Watch your cookie empire grow in real time – all without leaving your terminal. When you're feeling lucky, head to the **Casino** and risk your hard-earned cookies on slots, coin flips, and dice.

---

## Features

- **🍪 Cookie Mining** – press `Space` or `Enter` to manually bake cookies
- **🏗 Auto-Generation** – buy Cursors, Grandmas, Farms, and Mines that generate cookies per second
- **📈 Exponential Cost Scaling** – `cost = base × 1.15ⁿ`
- **✨ Prestige System (Ascension)** – reach 1,000,000 total cookies to unlock Heavenly Chips (+1% global CPS per chip, permanent)
- **🎰 Casino Minigames** – Slot Machine, Coin Flip, Dice Wager
- **💾 Auto-Save / Manual Save** – saves every 60 s to `$XDG_CONFIG_HOME/cookie_clicker/save.json`
- **⚡ Multi-threaded** – input and tick threads feed a single event channel, keeping CPS ticking even while you're idle

---

## Installation (Arch Linux)

### Via `makepkg` (recommended)

```bash
git clone https://github.com/m4rcel-lol/cli-clicker-gambling.git
cd cli-clicker-gambling
makepkg -si
```

The binary will be installed at `/usr/bin/cookie-clicker`.

### Manual (any Linux)

```bash
git clone https://github.com/m4rcel-lol/cli-clicker-gambling.git
cd cli-clicker-gambling
cargo build --release
./target/release/cookie_clicker
```

### Requirements

- Rust stable ≥ 1.70
- A terminal emulator that supports Unicode and at least 80×24 cells

---

## Keybindings

| Key | Action |
|-----|--------|
| `Space` / `Enter` | Mine a cookie |
| `1` | Buy Cursor |
| `2` | Buy Grandma |
| `3` | Buy Farm |
| `4` | Buy Mine |
| `G` | Toggle Casino view |
| `A` | Ascend (Prestige) – only when unlocked |
| `S` | Manual save |
| `Q` / `Ctrl-C` | Quit (auto-saves) |

### Casino Keybindings

| Key | Action |
|-----|--------|
| `S` | Slot Machine |
| `F` | Coin Flip |
| `D` | Dice Wager |
| `G` | Return to Casino menu / main game |
| `0-9` | Enter wager / guess |
| `Enter` | Confirm / flip / roll |
| `Backspace` | Erase digit |

---

## Game Mechanics

### Buildings

| Building | Base Cost | CPS |
|----------|-----------|-----|
| Cursor  | 15        | 0.1 |
| Grandma | 100       | 1.0 |
| Farm    | 1,100     | 8.0 |
| Mine    | 12,000    | 47.0 |

Cost scaling: **`base × 1.15ⁿ`** where `n` = number already owned.

### Prestige (Ascension)

Once you've baked **1,000,000 total cookies**, the `[A]scend` key unlocks.
Ascending resets your cookies and buildings, but grants **Heavenly Chips** equal to
`floor(total_baked / 1,000,000)`. Each chip gives a **permanent +1% global CPS multiplier**.

### Casino

| Game | Bet | Odds | Payout |
|------|-----|------|--------|
| Slot Machine | 100 cookies fixed | Weighted RNG | 3×🍒=5× · 3×💎=25× · 3×7=100× |
| Coin Flip | Custom (min 10) | 50/50 | 2× bet |
| Dice Wager | Custom (min 10) | 1/6 | 5× bet (net +4×) |

The house maintains an edge: slot symbols are weighted so 7️⃣ appears ~5% of the time.

---

## Save Location

```
$XDG_CONFIG_HOME/cookie_clicker/save.json
# Falls back to:
~/.config/cookie_clicker/save.json
```

---

## ⚠️ Health Warning

> **Remember to take breaks! Gambling is for cookies, not real life.**

This game contains simulated gambling mechanics using fictional in-game currency only.
No real money is involved. If you or someone you know has a gambling problem, please seek help.

---

## License

MIT – see [LICENSE](LICENSE).

