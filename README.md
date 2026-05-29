# Pokémon Rust

A roguelite Pokémon-inspired battle game built with **Rust**, **Leptos** and **WebAssembly**.

![Pokémon Rust](frontend/assets/images/logo.png)

---

## How to Play

### Objective
Defeat the Gym Leaders of all **8 gyms** to complete the game. Battles follow a fixed sequence: first the gym trainers, then the Gym Leader.

### Starting Out
Choose your **starter Pokémon** at the beginning. From that point, build your team by catching wild Pokémon you encounter after battles.

### Battles
Each turn you can:
- **Fight** — use one of your Pokémon's moves
- **Bag** — use an item from your inventory
- **Pokémon** — switch your active Pokémon
- **Run** — flee from wild Pokémon battles

Moves have types, power and limited PP. The battle ends when one side's Pokémon reaches 0 HP.

### Catching Pokémon
After defeating a **wild Pokémon** you can choose to catch it. If your team already has 6 members, you must choose which one to replace.
> You cannot catch Trainer or Gym Leader Pokémon.

### Team
Your team can hold up to **6 Pokémon**. All non-fainted members share EXP after each battle (divided equally). Levelling up restores HP and PP fully.

### Pokécenter
After each battle you have access to the Pokécenter to fully heal your entire team.
- **3 uses per gym** — resets when you advance to the next gym.

### Shop
After each battle you can visit the shop to spend money earned from victories.

| Item | Effect | Price |
|------|--------|-------|
| Potion | Restores 20 HP | ₽ 300 |
| Super Potion | Restores 50 HP | ₽ 700 |
| Full Restore | Fully restores HP | ₽ 3000 |
| Revive | Revives a fainted Pokémon at half HP | ₽ 1500 |
| Ether | Restores 10 PP to all moves | ₽ 1200 |
| Max Ether | Fully restores PP to all moves | ₽ 2000 |

### Rewards
- **EXP** multiplier: `20×` (Wild `1×`, Trainer `2×`, Gym Leader `4×`)
- **Money** multiplier: `10×` (same type scaling)

### Game Over
If all your Pokémon faint it's **game over** — start from scratch.

---

## Notes on This Version

- Status moves (e.g. Growl, Sleep) and multi-turn moves are **not implemented**. Pokémon will never be assigned moves of these types.
- Pokémon data (stats, moves, sprites, cries) is fetched from the [PokéAPI](https://pokeapi.co) and cached locally in IndexedDB.

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | [Rust](https://www.rust-lang.org/) |
| Frontend framework | [Leptos 0.7](https://leptos.dev/) (CSR) |
| Compile target | [WebAssembly](https://webassembly.org/) via `wasm32-unknown-unknown` |
| Build tool | [Trunk](https://trunkrs.dev/) |
| Pokémon data | [PokéAPI](https://pokeapi.co) |
| Local cache | IndexedDB via [Rexie](https://github.com/devashishdxt/rexie) |
| Audio | Web Audio API |
| Font | [Press Start 2P](https://fonts.google.com/specimen/Press+Start+2P) |
| Styling | Vanilla CSS |

### Project Structure

```
pokemon-rust/
├── game-core/          # Pure Rust game logic (no WASM deps)
│   ├── src/
│   │   ├── battle/     # Turn engine, damage calc, AI
│   │   ├── run/        # Run state, gym progression, rewards
│   │   ├── pokemon.rs  # Pokémon model, EXP, level up
│   │   ├── moves.rs    # Move model and PP
│   │   ├── inventory.rs
│   │   └── types.rs
│   └── tests/          # Integration tests
└── frontend/           # Leptos CSR app
    ├── src/
    │   ├── audio/      # Web Audio API wrapper (music, SFX, cries)
    │   ├── components/ # UI components (battle layout, cards, animations)
    │   ├── core/       # API client, IndexedDB cache, Pokémon generator
    │   └── pages/      # Home, Game
    └── assets/
        ├── audio/      # Music and SFX
        ├── fonts/
        ├── images/
        └── styles/     # Per-component CSS files
```

### Running Locally

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Trunk
cargo install trunk

# Run dev server
cd frontend
trunk serve
```

---

## License

MIT
