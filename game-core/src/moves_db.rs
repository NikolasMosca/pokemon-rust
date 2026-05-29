use crate::moves::{Move, MoveCategory, MoveEffect};
use crate::types::Type;
use crate::battle::rng::Rng;

/// Entry nel database statico delle mosse.
/// Separata da `Move` per essere usabile in `const` context.
pub struct MoveEntry {
    pub name:      &'static str,
    pub move_type: Type,
    pub category:  MoveCategory,
    pub power:     u8,
    pub accuracy:  u8,
    pub pp:        u8,
    pub effect:    MoveEffect,
}

impl MoveEntry {
    const fn phys(name: &'static str, t: Type, power: u8, accuracy: u8, pp: u8) -> Self {
        Self { name, move_type: t, category: MoveCategory::Physical, power, accuracy, pp, effect: MoveEffect::None }
    }
    const fn spec(name: &'static str, t: Type, power: u8, accuracy: u8, pp: u8) -> Self {
        Self { name, move_type: t, category: MoveCategory::Special, power, accuracy, pp, effect: MoveEffect::None }
    }
    const fn heal(name: &'static str, t: Type) -> Self {
        Self { name, move_type: t, category: MoveCategory::Status, power: 0, accuracy: 100, pp: 10, effect: MoveEffect::Heal { percent: 50 } }
    }
    const fn drain_phys(name: &'static str, t: Type, power: u8, accuracy: u8, pp: u8) -> Self {
        Self { name, move_type: t, category: MoveCategory::Physical, power, accuracy, pp, effect: MoveEffect::Drain { percent: 50 } }
    }
    const fn drain_spec(name: &'static str, t: Type, power: u8, accuracy: u8, pp: u8) -> Self {
        Self { name, move_type: t, category: MoveCategory::Special, power, accuracy, pp, effect: MoveEffect::Drain { percent: 50 } }
    }

    pub fn to_move(&self) -> Move {
        Move::new(self.name, self.move_type, self.category.clone(), self.power, self.accuracy, self.pp)
            .with_effect(self.effect.clone())
    }

    pub fn is_supported(&self) -> bool {
        match &self.effect {
            MoveEffect::Heal { .. } | MoveEffect::Drain { .. } => true,
            MoveEffect::None => self.power > 0 && self.category != MoveCategory::Status,
        }
    }

    pub fn deals_damage(&self) -> bool {
        self.power > 0
    }
}

pub static ALL_MOVES: &[MoveEntry] = &[
    // ── Normal ───────────────────────────────────────────────────────────────
    MoveEntry::phys("tackle",        Type::Normal,   40,  100, 35),
    MoveEntry::phys("scratch",       Type::Normal,   40,  100, 35),
    MoveEntry::phys("pound",         Type::Normal,   40,  100, 35),
    MoveEntry::phys("quick-attack",  Type::Normal,   40,  100, 30),
    MoveEntry::phys("cut",           Type::Normal,   50,   95, 30),
    MoveEntry::phys("headbutt",      Type::Normal,   70,  100, 15),
    MoveEntry::phys("body-slam",     Type::Normal,   85,  100, 15),
    MoveEntry::phys("slash",         Type::Normal,   70,  100, 20),
    MoveEntry::phys("strength",      Type::Normal,   80,  100, 15),
    MoveEntry::phys("take-down",     Type::Normal,   90,   85, 20),
    MoveEntry::phys("double-edge",   Type::Normal,  120,  100, 15),
    MoveEntry::phys("facade",        Type::Normal,   70,  100, 20),
    MoveEntry::phys("extreme-speed", Type::Normal,   80,  100,  5),
    MoveEntry::phys("hyper-fang",    Type::Normal,   80,   90, 15),
    MoveEntry::phys("fury-swipes",   Type::Normal,   18,   80, 15),
    MoveEntry::spec("swift",         Type::Normal,   60,  100, 20),
    MoveEntry::spec("hyper-voice",   Type::Normal,   90,  100, 10),
    MoveEntry::spec("tri-attack",    Type::Normal,   80,  100, 10),
    MoveEntry::heal("recover",       Type::Normal),
    MoveEntry::heal("soft-boiled",   Type::Normal),
    MoveEntry::heal("milk-drink",    Type::Normal),
    MoveEntry::heal("slack-off",     Type::Normal),

    // ── Fire ─────────────────────────────────────────────────────────────────
    MoveEntry::spec("ember",         Type::Fire,     40,  100, 25),
    MoveEntry::spec("fire-spin",     Type::Fire,     35,   85, 15),
    MoveEntry::spec("flamethrower",  Type::Fire,     90,  100, 15),
    MoveEntry::spec("heat-wave",     Type::Fire,     95,   90, 10),
    MoveEntry::spec("fire-blast",    Type::Fire,    110,   85,  5),
    MoveEntry::spec("overheat",      Type::Fire,    130,   90,  5),
    MoveEntry::phys("flame-wheel",   Type::Fire,     60,  100, 25),
    MoveEntry::phys("fire-punch",    Type::Fire,     75,  100, 15),
    MoveEntry::phys("blaze-kick",    Type::Fire,     85,   90, 10),
    MoveEntry::phys("flare-blitz",   Type::Fire,    120,  100, 15),
    MoveEntry::phys("sacred-fire",   Type::Fire,    100,   95,  5),

    // ── Water ────────────────────────────────────────────────────────────────
    MoveEntry::spec("water-gun",     Type::Water,    40,  100, 25),
    MoveEntry::spec("bubble-beam",   Type::Water,    65,  100, 20),
    MoveEntry::spec("water-pulse",   Type::Water,    60,  100, 20),
    MoveEntry::spec("brine",         Type::Water,    65,  100, 10),
    MoveEntry::spec("surf",          Type::Water,    90,  100, 15),
    MoveEntry::spec("muddy-water",   Type::Water,    90,   85, 10),
    MoveEntry::spec("hydro-pump",    Type::Water,   110,   80,  5),
    MoveEntry::phys("aqua-jet",      Type::Water,    40,  100, 20),
    MoveEntry::phys("waterfall",     Type::Water,    80,  100, 15),
    MoveEntry::phys("dive",          Type::Water,    80,  100, 10),
    MoveEntry::phys("aqua-tail",     Type::Water,    90,   90, 10),
    MoveEntry::phys("crabhammer",    Type::Water,   100,   90, 10),

    // ── Electric ─────────────────────────────────────────────────────────────
    MoveEntry::spec("thunder-shock", Type::Electric, 40,  100, 30),
    MoveEntry::spec("charge-beam",   Type::Electric, 50,   90, 10),
    MoveEntry::spec("discharge",     Type::Electric, 80,  100, 15),
    MoveEntry::spec("thunderbolt",   Type::Electric, 90,  100, 15),
    MoveEntry::spec("thunder",       Type::Electric,110,   70, 10),
    MoveEntry::spec("zap-cannon",    Type::Electric,120,   50,  5),
    MoveEntry::phys("spark",         Type::Electric, 65,  100, 20),
    MoveEntry::phys("thunder-punch", Type::Electric, 75,  100, 15),
    MoveEntry::phys("wild-charge",   Type::Electric, 90,  100, 15),
    MoveEntry::phys("volt-tackle",   Type::Electric,120,  100, 15),

    // ── Grass ────────────────────────────────────────────────────────────────
    MoveEntry::phys("vine-whip",     Type::Grass,    45,  100, 25),
    MoveEntry::phys("razor-leaf",    Type::Grass,    55,   95, 25),
    MoveEntry::phys("bullet-seed",   Type::Grass,    25,  100, 30),
    MoveEntry::phys("seed-bomb",     Type::Grass,    80,  100, 15),
    MoveEntry::phys("leaf-blade",    Type::Grass,    90,  100, 15),
    MoveEntry::phys("power-whip",    Type::Grass,   120,   85, 10),
    MoveEntry::phys("horn-leech",    Type::Grass,    75,  100, 10), // drain
    MoveEntry::spec("magical-leaf",  Type::Grass,    60,  100, 20),
    MoveEntry::spec("energy-ball",   Type::Grass,    90,  100, 10),
    MoveEntry::spec("solar-beam",    Type::Grass,   120,  100, 10),
    MoveEntry::spec("petal-dance",   Type::Grass,   120,  100, 10),
    MoveEntry::drain_spec("absorb",      Type::Grass, 20, 100, 25),
    MoveEntry::drain_spec("mega-drain",  Type::Grass, 40, 100, 15),
    MoveEntry::drain_spec("giga-drain",  Type::Grass, 75, 100, 10),
    MoveEntry::heal("synthesis",     Type::Grass),

    // ── Ice ──────────────────────────────────────────────────────────────────
    MoveEntry::spec("powder-snow",   Type::Ice,      40,  100, 25),
    MoveEntry::spec("aurora-beam",   Type::Ice,      65,  100, 20),
    MoveEntry::spec("ice-beam",      Type::Ice,      90,  100, 10),
    MoveEntry::spec("blizzard",      Type::Ice,     110,   70,  5),
    MoveEntry::spec("freeze-dry",    Type::Ice,      70,  100, 20),
    MoveEntry::spec("glaciate",      Type::Ice,      65,   95, 10),
    MoveEntry::phys("ice-shard",     Type::Ice,      40,  100, 30),
    MoveEntry::phys("ice-punch",     Type::Ice,      75,  100, 15),
    MoveEntry::phys("icicle-spear",  Type::Ice,      25,  100, 30),
    MoveEntry::phys("avalanche",     Type::Ice,      60,  100, 10),

    // ── Fighting ─────────────────────────────────────────────────────────────
    MoveEntry::phys("karate-chop",   Type::Fighting, 50,  100, 25),
    MoveEntry::phys("low-kick",      Type::Fighting, 60,  100, 20),
    MoveEntry::phys("mach-punch",    Type::Fighting, 40,  100, 30),
    MoveEntry::phys("brick-break",   Type::Fighting, 75,  100, 15),
    MoveEntry::phys("sky-uppercut",  Type::Fighting, 85,   90, 15),
    MoveEntry::phys("jump-kick",     Type::Fighting,100,   95, 10),
    MoveEntry::phys("cross-chop",    Type::Fighting,100,   80,  5),
    MoveEntry::phys("high-jump-kick",Type::Fighting,130,   90, 10),
    MoveEntry::phys("superpower",    Type::Fighting,120,  100,  5),
    MoveEntry::phys("close-combat",  Type::Fighting,120,  100,  5),
    MoveEntry::phys("submission",    Type::Fighting, 80,   80, 20),
    MoveEntry::spec("vacuum-wave",   Type::Fighting, 40,  100, 30),
    MoveEntry::spec("focus-blast",   Type::Fighting,120,   70,  5),
    MoveEntry::spec("aura-sphere",   Type::Fighting, 80,  100, 20),
    MoveEntry::drain_phys("drain-punch", Type::Fighting, 75, 100, 10),

    // ── Poison ───────────────────────────────────────────────────────────────
    MoveEntry::phys("poison-sting",  Type::Poison,   15,  100, 35),
    MoveEntry::phys("poison-fang",   Type::Poison,   50,  100, 15),
    MoveEntry::phys("cross-poison",  Type::Poison,   70,  100, 20),
    MoveEntry::phys("poison-jab",    Type::Poison,   80,  100, 20),
    MoveEntry::phys("gunk-shot",     Type::Poison,  120,   80,  5),
    MoveEntry::spec("acid",          Type::Poison,   40,  100, 30),
    MoveEntry::spec("sludge",        Type::Poison,   65,  100, 20),
    MoveEntry::spec("venoshock",     Type::Poison,   65,  100, 10),
    MoveEntry::spec("sludge-bomb",   Type::Poison,   90,  100, 10),
    MoveEntry::spec("sludge-wave",   Type::Poison,   95,  100, 10),

    // ── Ground ───────────────────────────────────────────────────────────────
    MoveEntry::spec("mud-slap",      Type::Ground,   20,  100, 10),
    MoveEntry::spec("earth-power",   Type::Ground,   90,  100, 10),
    MoveEntry::spec("mud-bomb",      Type::Ground,   65,   85, 10),
    MoveEntry::phys("sand-tomb",     Type::Ground,   35,   85, 15),
    MoveEntry::phys("bone-club",     Type::Ground,   65,   85, 20),
    MoveEntry::phys("magnitude",     Type::Ground,   70,  100, 30),
    MoveEntry::phys("dig",           Type::Ground,   80,  100, 10),
    MoveEntry::phys("drill-run",     Type::Ground,   80,   95, 10),
    MoveEntry::phys("bonemerang",    Type::Ground,   50,   90, 10),
    MoveEntry::phys("earthquake",    Type::Ground,  100,  100, 10),

    // ── Flying ───────────────────────────────────────────────────────────────
    MoveEntry::spec("gust",          Type::Flying,   40,  100, 35),
    MoveEntry::spec("air-slash",     Type::Flying,   75,   95, 15),
    MoveEntry::phys("peck",          Type::Flying,   35,  100, 35),
    MoveEntry::phys("wing-attack",   Type::Flying,   60,  100, 35),
    MoveEntry::phys("aerial-ace",    Type::Flying,   60,  100, 20),
    MoveEntry::phys("pluck",         Type::Flying,   60,  100, 20),
    MoveEntry::phys("drill-peck",    Type::Flying,   80,  100, 20),
    MoveEntry::phys("bounce",        Type::Flying,   85,   85,  5),
    MoveEntry::phys("fly",           Type::Flying,   90,   95, 15),
    MoveEntry::phys("brave-bird",    Type::Flying,  120,  100, 15),
    MoveEntry::phys("sky-attack",    Type::Flying,  140,   90,  5),
    MoveEntry::heal("roost",         Type::Flying),

    // ── Psychic ──────────────────────────────────────────────────────────────
    MoveEntry::spec("confusion",     Type::Psychic,  50,  100, 25),
    MoveEntry::spec("psybeam",       Type::Psychic,  65,  100, 20),
    MoveEntry::spec("extrasensory",  Type::Psychic,  80,  100, 20),
    MoveEntry::spec("psyshock",      Type::Psychic,  80,  100, 10),
    MoveEntry::spec("psychic",       Type::Psychic,  90,  100, 10),
    MoveEntry::spec("stored-power",  Type::Psychic,  20,  100, 10),
    MoveEntry::spec("future-sight",  Type::Psychic, 120,  100, 10),
    MoveEntry::spec("dream-eater",   Type::Psychic, 100,  100, 15),
    MoveEntry::phys("psycho-cut",    Type::Psychic,  70,  100, 20),
    MoveEntry::phys("zen-headbutt",  Type::Psychic,  80,   90, 15),
    MoveEntry::heal("morning-sun",   Type::Psychic),

    // ── Bug ──────────────────────────────────────────────────────────────────
    MoveEntry::phys("fury-cutter",   Type::Bug,      40,   95, 20),
    MoveEntry::phys("twineedle",     Type::Bug,      25,  100, 20),
    MoveEntry::phys("bug-bite",      Type::Bug,      60,  100, 20),
    MoveEntry::phys("u-turn",        Type::Bug,      70,  100, 20),
    MoveEntry::phys("x-scissor",     Type::Bug,      80,  100, 15),
    MoveEntry::phys("megahorn",      Type::Bug,     120,   85, 10),
    MoveEntry::phys("pin-missile",   Type::Bug,      25,   95, 20),
    MoveEntry::spec("silver-wind",   Type::Bug,      60,  100,  5),
    MoveEntry::spec("signal-beam",   Type::Bug,      75,  100, 15),
    MoveEntry::spec("bug-buzz",      Type::Bug,      90,  100, 10),
    MoveEntry::drain_phys("leech-life", Type::Bug,   80,  100, 10),

    // ── Rock ─────────────────────────────────────────────────────────────────
    MoveEntry::phys("rollout",       Type::Rock,     30,   90, 20),
    MoveEntry::phys("rock-throw",    Type::Rock,     50,   90, 15),
    MoveEntry::phys("smack-down",    Type::Rock,     50,  100, 15),
    MoveEntry::phys("rock-blast",    Type::Rock,     25,   90, 10),
    MoveEntry::phys("rock-slide",    Type::Rock,     75,   90, 10),
    MoveEntry::phys("stone-edge",    Type::Rock,    100,   80,  5),
    MoveEntry::phys("head-smash",    Type::Rock,    150,   80,  5),
    MoveEntry::phys("rock-wrecker",  Type::Rock,    150,   90,  5),
    MoveEntry::spec("ancient-power", Type::Rock,     60,  100,  5),
    MoveEntry::spec("power-gem",     Type::Rock,     80,  100, 20),

    // ── Ghost ────────────────────────────────────────────────────────────────
    MoveEntry::phys("lick",          Type::Ghost,    30,  100, 30),
    MoveEntry::phys("shadow-sneak",  Type::Ghost,    40,  100, 30),
    MoveEntry::phys("shadow-punch",  Type::Ghost,    60,  100, 20),
    MoveEntry::phys("shadow-claw",   Type::Ghost,    70,  100, 15),
    MoveEntry::phys("phantom-force", Type::Ghost,    90,  100, 10),
    MoveEntry::spec("ominous-wind",  Type::Ghost,    60,  100,  5),
    MoveEntry::spec("hex",           Type::Ghost,    65,  100, 10),
    MoveEntry::spec("shadow-ball",   Type::Ghost,    80,  100, 15),
    MoveEntry::spec("night-shade",   Type::Ghost,    60,  100, 15),

    // ── Dragon ───────────────────────────────────────────────────────────────
    MoveEntry::spec("twister",       Type::Dragon,   40,  100, 20),
    MoveEntry::spec("dragon-breath", Type::Dragon,   60,  100, 20),
    MoveEntry::spec("dragon-pulse",  Type::Dragon,   85,  100, 10),
    MoveEntry::spec("draco-meteor",  Type::Dragon,  130,   90,  5),
    MoveEntry::spec("spacial-rend",  Type::Dragon,  100,   95,  5),
    MoveEntry::phys("dragon-tail",   Type::Dragon,   60,   90, 10),
    MoveEntry::phys("dragon-claw",   Type::Dragon,   80,  100, 15),
    MoveEntry::phys("dragon-rush",   Type::Dragon,  100,   75, 10),
    MoveEntry::phys("outrage",       Type::Dragon,  120,  100, 10),

    // ── Dark ─────────────────────────────────────────────────────────────────
    MoveEntry::phys("pursuit",       Type::Dark,     40,  100, 20),
    MoveEntry::phys("bite",          Type::Dark,     60,  100, 25),
    MoveEntry::phys("assurance",     Type::Dark,     60,  100, 10),
    MoveEntry::phys("payback",       Type::Dark,     50,  100, 10),
    MoveEntry::phys("knock-off",     Type::Dark,     65,  100, 20),
    MoveEntry::phys("sucker-punch",  Type::Dark,     70,  100,  5),
    MoveEntry::phys("night-slash",   Type::Dark,     70,  100, 15),
    MoveEntry::phys("crunch",        Type::Dark,     80,  100, 15),
    MoveEntry::phys("foul-play",     Type::Dark,     95,  100, 15),
    MoveEntry::spec("dark-pulse",    Type::Dark,     80,  100, 15),

    // ── Steel ────────────────────────────────────────────────────────────────
    MoveEntry::phys("metal-claw",    Type::Steel,    50,   95, 35),
    MoveEntry::phys("bullet-punch",  Type::Steel,    40,  100, 30),
    MoveEntry::phys("steel-wing",    Type::Steel,    70,   90, 25),
    MoveEntry::phys("magnet-bomb",   Type::Steel,    60,  100, 20),
    MoveEntry::phys("iron-head",     Type::Steel,    80,  100, 15),
    MoveEntry::phys("meteor-mash",   Type::Steel,    90,   90, 10),
    MoveEntry::phys("iron-tail",     Type::Steel,   100,   75, 15),
    MoveEntry::phys("gyro-ball",     Type::Steel,   150,  100,  5),
    MoveEntry::spec("mirror-shot",   Type::Steel,    65,   85, 10),
    MoveEntry::spec("flash-cannon",  Type::Steel,    80,  100, 10),
    MoveEntry::spec("doom-desire",   Type::Steel,   140,  100,  5),

    // ── Fairy ────────────────────────────────────────────────────────────────
    MoveEntry::spec("fairy-wind",    Type::Fairy,    40,  100, 30),
    MoveEntry::spec("disarming-voice",Type::Fairy,   40,  100, 15),
    MoveEntry::spec("dazzling-gleam",Type::Fairy,    80,  100, 10),
    MoveEntry::spec("moonblast",     Type::Fairy,    95,  100, 15),
    MoveEntry::phys("play-rough",    Type::Fairy,    90,   90, 10),
    MoveEntry::drain_spec("draining-kiss", Type::Fairy, 50, 100, 10),
    MoveEntry::heal("moonlight",     Type::Fairy),
];

/// Livello soglia oltre il quale una mossa "potente" diventa disponibile.
/// Mosse con power > 80 sono riservate a livelli ≥ HIGH_POWER_MIN_LEVEL.
const HIGH_POWER_MIN_LEVEL: u8 = 25;
/// Mosse con power > 110 sono riservate a livelli ≥ VERY_HIGH_POWER_MIN_LEVEL.
const VERY_HIGH_POWER_MIN_LEVEL: u8 = 50;

/// Seleziona 4 mosse per un Pokémon dato il suo tipo primario e livello.
///
/// Garanzie:
/// - Restituisce sempre esattamente 4 mosse
/// - Prima mossa: STAB (stesso tipo del Pokémon)
/// - Nessun duplicato
/// - Almeno una mossa con danno diretto
/// - Le mosse più potenti appaiono solo ad alti livelli
pub fn pick_moves(pokemon_type: Type, level: u8, rng: &mut Rng) -> Vec<Move> {
    // Pool STAB: mosse dello stesso tipo, filtrate per livello
    let stab_pool: Vec<&MoveEntry> = ALL_MOVES.iter()
        .filter(|m| m.move_type == pokemon_type && level_ok(m, level))
        .collect();

    // Pool Normal: mosse Normal (copertura universale), filtrate per livello
    let normal_pool: Vec<&MoveEntry> = ALL_MOVES.iter()
        .filter(|m| m.move_type == Type::Normal && level_ok(m, level))
        .collect();

    // Pool generale: tutte le mosse disponibili al livello, escluse STAB e Normal
    let other_pool: Vec<&MoveEntry> = ALL_MOVES.iter()
        .filter(|m| m.move_type != pokemon_type && m.move_type != Type::Normal && level_ok(m, level))
        .collect();

    let mut result: Vec<Move> = Vec::with_capacity(4);
    let mut used_names: Vec<&str> = Vec::with_capacity(4);

    // Slot 1: STAB con danno (obbligatorio)
    let stab_dmg: Vec<&&MoveEntry> = stab_pool.iter()
        .filter(|m| m.deals_damage())
        .collect();
    if !stab_dmg.is_empty() {
        let idx = rng.next(stab_dmg.len() as u64) as usize;
        let entry = stab_dmg[idx];
        used_names.push(entry.name);
        result.push(entry.to_move());
    }

    // Slot 2: seconda mossa STAB (preferibilmente diversa categoria dal slot 1)
    if result.len() < 4 {
        let first_cat = result.first().map(|m| &m.category);
        let candidates: Vec<&&MoveEntry> = stab_pool.iter()
            .filter(|m| !used_names.contains(&m.name))
            .filter(|m| Some(&m.category) != first_cat || stab_pool.len() <= used_names.len() + 1)
            .collect();
        if !candidates.is_empty() {
            let idx = rng.next(candidates.len() as u64) as usize;
            let entry = candidates[idx];
            used_names.push(entry.name);
            result.push(entry.to_move());
        }
    }

    // Slot 3: mossa Normal (copertura)
    if result.len() < 4 {
        let candidates: Vec<&&MoveEntry> = normal_pool.iter()
            .filter(|m| !used_names.contains(&m.name) && m.deals_damage())
            .collect();
        if !candidates.is_empty() {
            let idx = rng.next(candidates.len() as u64) as usize;
            let entry = candidates[idx];
            used_names.push(entry.name);
            result.push(entry.to_move());
        }
    }

    // Slot 4: mossa da pool generale o riempitura
    if result.len() < 4 {
        let candidates: Vec<&&MoveEntry> = other_pool.iter()
            .filter(|m| !used_names.contains(&m.name))
            .collect();
        if !candidates.is_empty() {
            let idx = rng.next(candidates.len() as u64) as usize;
            let entry = candidates[idx];
            used_names.push(entry.name);
            result.push(entry.to_move());
        }
    }

    // Riempimento di emergenza: se mancano ancora slot, pesca da tutto il DB
    while result.len() < 4 {
        let all_available: Vec<&MoveEntry> = ALL_MOVES.iter()
            .filter(|m| !used_names.contains(&m.name) && level_ok(m, level))
            .collect();
        if all_available.is_empty() {
            // Ultimo resort: ripeti mosse (non dovrebbe mai accadere con il DB attuale)
            let entry = &ALL_MOVES[rng.next(ALL_MOVES.len() as u64) as usize];
            result.push(entry.to_move());
            break;
        }
        let idx = rng.next(all_available.len() as u64) as usize;
        let entry = all_available[idx];
        used_names.push(entry.name);
        result.push(entry.to_move());
    }

    result
}

/// Controlla se una mossa è disponibile al livello dato in base al suo power.
fn level_ok(entry: &MoveEntry, level: u8) -> bool {
    if entry.power >= 110 {
        level >= VERY_HIGH_POWER_MIN_LEVEL
    } else if entry.power >= 85 {
        level >= HIGH_POWER_MIN_LEVEL
    } else {
        true
    }
}
