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

- **🍪 Cookie Mining** – press `Space` or `Enter` to manually bake cookies; click power scales with Cursors owned
- **🏗 Auto-Generation** – buy Cursors, Grandmas, Farms, and Mines that generate cookies per second
- **📈 Exponential Cost Scaling** – `cost = base × 1.15ⁿ`
- **🔬 Upgrades System** – 8 purchasable upgrades (2 per building) that multiply a building's CPS output; press `[T]` to toggle the Upgrades tab
- **✨ Golden Cookies** – random golden cookies appear every ~2 minutes; press `[C]` quickly to collect a bonus!
- **🌟 Prestige System (Ascension)** – reach 1,000,000 total cookies to unlock Heavenly Chips (+1% global CPS per chip, permanent)
- **🎰 Casino Minigames** – Slot Machine, Coin Flip, Dice Wager with lifetime stats tracking
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
| `Space` / `Enter` | Mine a cookie (power scales with Cursors owned) |
| `1` | Buy Cursor (or buy upgrade 1 in Upgrades tab) |
| `2` | Buy Grandma (or buy upgrade 2 in Upgrades tab) |
| `3` | Buy Farm (or buy upgrade 3 in Upgrades tab) |
| `4` | Buy Mine (or buy upgrade 4 in Upgrades tab) |
| `5`–`8` | Buy upgrades 5–8 (Upgrades tab only) |
| `T` | Toggle between Buildings and Upgrades tab |
| `C` | Collect a Golden Cookie (when one appears!) |
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

### Click Power

Manually clicking gives **`1 + 0.5 × cursors_owned`** cookies per click.
Owning 10 Cursors → 6 cookies per click!

### Upgrades

Toggle to the Upgrades tab with `[T]` and press the number key shown to purchase.
Each upgrade **doubles** that building's CPS output; two upgrades of the same type stack
multiplicatively (×2 then ×2 = ×4 total).

| # | Name | Building | Cost |
|---|------|----------|------|
| 1 | Nimble Fingers | Cursor ×2 | 100 |
| 2 | Cursor Overdrive | Cursor ×2 (stack) | 2,000 |
| 3 | Loving Grandmas | Grandma ×2 | 1,000 |
| 4 | Senior Discount | Grandma ×2 (stack) | 20,000 |
| 5 | Irrigation System | Farm ×2 | 11,000 |
| 6 | Hydroponics | Farm ×2 (stack) | 110,000 |
| 7 | Deep Excavation | Mine ×2 | 130,000 |
| 8 | Quantum Drilling | Mine ×2 (stack) | 1,300,000 |

### Golden Cookies

Roughly every 2 minutes a **Golden Cookie** flashes in the stats panel.
Press **`[C]`** before the 15-second window closes to collect
**500 + 5% of your current cookies** as a bonus!

### Prestige (Ascension)

Once you've baked **1,000,000 total cookies**, the `[A]scend` key unlocks.
Ascending resets your cookies, buildings, and upgrades, but grants **Heavenly Chips** equal to
`floor(total_baked / 1,000,000)`. Each chip gives a **permanent +1% global CPS multiplier**.

### Casino

| Game | Bet | Odds | Payout |
|------|-----|------|--------|
| Slot Machine | 100 cookies fixed | Weighted RNG | 3×🍒=5× · 3×🍋=3× · 3×🔔=7× · 3×💎=25× · 3×7=100× |
| Coin Flip | Custom (min 10) | 50/50 | 2× bet |
| Dice Wager | Custom (min 10) | 1/6 | 5× bet (net +4×) |

The casino menu displays your **lifetime wagered / won / net** stats.
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

