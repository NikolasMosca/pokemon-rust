# Frontend Game Loop — 2026-05-22

## Architettura

### Rotta
Rotta unica `/game` pilotata da `RunPhase`. Ogni fase corrisponde a un componente
dedicato montato/smontato dalla pagina principale. Permette switch futuro a rotte
separate senza riscrivere i componenti.

### Struttura componenti

```
frontend/src/
├── pages/
│   └── game.rs              — GamePage: provider di RunState, switch su RunPhase
└── components/
    ├── game/
    │   ├── mod.rs
    │   ├── starter_select.rs — scelta fra 3 starter random
    │   ├── battle_screen.rs  — battaglia attiva (usa BattleLayout esistente)
    │   ├── post_battle.rs    — pannello vittoria + opzioni [Pokécenter|Shop|Avanza]
    │   ├── shop_screen.rs    — negozio con 6 item hardcoded
    │   ├── pokecenter_screen.rs — heal + conferma
    │   ├── catch_screen.rs   — cattura + eventuale sostituzione team
    │   ├── game_over.rs      — schermata game over
    │   ├── run_complete.rs   — schermata fine run (8 badge)
    │   └── progress_popup.rs — popup avanzamento palestre
    └── (esistenti: battle_layout, pokemon_card, ecc.)
```

### Stato globale

`RunState` (da game-core) vive in un `RwSignal<Option<RunState>>` fornito via
`provide_context` in `GamePage`. Ogni componente figlio lo legge con `use_context`.

La fase corrente è `run_state.phase` — `GamePage` fa match su di essa e monta
il componente corretto.

---

## Fasi e componenti

| RunPhase | Componente montato |
|---|---|
| None (prima della run) | `StarterSelect` |
| `InBattle { Wild }` | `BattleScreen` |
| `InBattle { Trainer }` | `BattleScreen` |
| `InBattle { GymLeader }` | `BattleScreen` |
| `PostBattle` | `PostBattle` |
| `Pokecenter` | `PokecenterScreen` |
| `Shop` | `ShopScreen` |
| `GameOver` | `GameOver` |
| `RunComplete` | `RunComplete` |

`CatchScreen` appare come overlay sopra `PostBattle` quando il giocatore
sceglie di catturare il Pokémon selvatico appena sconfitto.

`ProgressPopup` appare come overlay su qualsiasi fase (bottone fisso in alto a dx).

---

## Starter Select

- All'apertura della pagina `/game` si fetchano 3 Pokémon random dal pool starter
  (lista hardcoded di specie base: Bulbasaur, Charmander, Squirtle, Pikachu,
  Eevee, Chikorita, Cyndaquil, Totodile, Treecko, Torchic, Mudkip)
- Ogni card mostra: sprite front, nome, tipo, stat HP
- Il giocatore clicca uno → viene creato il `RunState` con quel Pokémon a livello 5
- Le mosse del starter vengono fetchate da PokéAPI (mosse apprese a livello ≤5)

---

## Pool Pokémon selvatici

20 specie con peso di apparizione:

| Rarità | Peso | Specie |
|---|---|---|
| Comune | 30 | Rattata, Pidgey, Caterpie, Weedle, Geodude |
| Non comune | 15 | Ekans, Sandshrew, Jigglypuff, Oddish, Psyduck |
| Raro | 7 | Growlithe, Abra, Machop, Gastly, Onix |
| Molto raro | 3 | Scyther, Lapras, Dratini, Eevee, Porygon |

Livello del Pokémon selvatico = `team_avg_level ± 2` (min 3).
Se il livello raggiunge o supera il `min_level` di evoluzione dalla chain → specie evoluta.

---

## Mosse da PokéAPI

Fetch di `pokemon/{name}` → campo `moves[]` filtrato per:
- `version_group_details[].move_learn_method.name == "level-up"`
- `version_group_details[].level_learned_at <= pokemon.level`

Si prendono le ultime 4 mosse (ordinate per livello decrescente).
Il risultato viene salvato in `api_cache` IDB.

---

## Battle Screen

Usa `BattleLayout` esistente. La logica di turno:
1. Giocatore sceglie azione (Lotta / Borsa / Pokémon)
2. `execute_turn()` da game-core con la mossa scelta
3. UI aggiorna HP, mostra `DamageNumber`
4. Se il nemico è KO → transizione a `PostBattle` (o `CatchScreen` se selvatico)
5. Se il giocatore è KO → `check_game_over()` → `GameOver` o cambio Pokémon attivo obbligato

**Pokémon attivo**: indice del primo Pokémon non KO nel team. Se tutti KO → game over.

### Sequenza UI del turno

Il risultato del turno viene calcolato immediatamente da `execute_turn()` (sincrono),
ma la UI lo mostra in sequenza con delay fisso configurabile:

```
1. Blocca input (pannello azioni disabilitato)
2. Mostra danno giocatore → DamageNumber nemico + HP bar nemico scende  [delay]
3. Mostra messaggio "X ha usato Y!"                                      [delay]
4. Se nemico non KO:
   4a. Mostra danno nemico → DamageNumber giocatore + HP bar giocatore scende [delay]
   4b. Mostra messaggio "Nemico ha usato Z!"                              [delay]
5. Sblocca input (o transizione di fase)
```

Se il Pokémon giocatore attacca per secondo (speed inferiore), l'ordine è invertito
(4a→4b prima di 2→3).

Il delay è definito in una costante `TURN_DELAY_MS: u32 = 800` in `battle_screen.rs`,
modificabile senza toccare la logica.

### Azioni disponibili

Il pannello azioni (in basso a destra) mostra 3 bottoni principali:
`[LOTTA]` `[BORSA]` `[POKÉMON]`

Non esiste il tasto Fuga — il giocatore non può scappare dalle battaglie.

#### LOTTA
Sostituisce i 4 bottoni generici con le mosse del Pokémon attivo.
- Ogni bottone mostra: nome mossa, tipo, PP rimanenti (`X/Y`)
- Mosse con PP = 0: bottone disabilitato (grigio, non cliccabile)
- Se tutte le mosse hanno PP = 0: i bottoni sono tutti disabilitati,
  il giocatore è costretto a usare Borsa o cambiare Pokémon
- Click su mossa → esegue il turno

#### BORSA
Apre overlay popup con lista item in possesso (raggruppati per tipo).
- Ogni riga: nome item, descrizione breve, quantità
- Item non applicabili al contesto sono disabilitati
  (es. Revive disabilitato se nessun Pokémon è KO)
- Click su item curativo → apre secondo popup di selezione Pokémon del team
  (si può usare su qualsiasi Pokémon, non solo quello attivo)
- Click su item PP → apre selezione mossa del Pokémon target
- Dopo uso item: chiude popup, il nemico esegue il suo attacco (turno consumato)

#### POKÉMON
Apre overlay popup con la lista del team (max 6).
- Ogni riga: sprite, nome, HP bar, livello
- Pokémon KO: mostrati ma non selezionabili (testo rosso)
- Pokémon attivo: evidenziato, non selezionabile
- **Cambio volontario** (Pokémon attivo ancora vivo): consuma il turno,
  il nemico attacca dopo il cambio
- **Cambio obbligato** (Pokémon attivo KO): non consuma il turno,
  il nemico non attacca; il popup si apre automaticamente

---

## Post Battle

Pannello overlay sopra la scena con:
- Riepilogo: EXP guadagnata, soldi guadagnati, eventuali level up
- Se selvatico e HP nemico < 50%: bottone "Cattura"
- Bottoni: [Pokécenter (N rimasti) | Shop | Prossima battaglia]
- Pokécenter disabilitato se `pokecenter_uses >= POKECENTER_MAX`

---

## Progress Popup

Bottone fisso in alto a destra su tutte le schermate di gioco.
Mostra un percorso verticale con 8 nodi (uno per palestra):

```
● Palestra 1 — COMPLETATA (badge ottenuto)
● Palestra 2 — IN CORSO
  ├─ Allenatore 1 ✓
  ├─ Allenatore 2 ✓
  ├─ Allenatore 3 ✗
  └─ Capopalestra ✗
○ Palestra 3 — BLOCCATA
...
○ Palestra 8 — BLOCCATA
```

---

## Generazione nemici (frontend)

La generazione dei nemici selvatici e degli allenatori avviene nel frontend
(richiede fetch API) ma usa `team_average_level()` da game-core.

```
frontend/src/core/
└── generator.rs   — genera Pokemon nemici: specie, livello, mosse, evoluzione
```

`generator.rs` fa:
1. Sceglie specie dal pool pesato usando `Rng` da game-core
2. Determina livello (`avg ± 2`)
3. Fetcha evolution chain → determina forma corretta per il livello
4. Fetcha mosse per specie+livello
5. Costruisce `Pokemon` da game-core

---

## Persistenza

Ad ogni cambio di fase `RunState` viene serializzato e salvato su IDB
(`store.save_run()`). All'apertura di `/game` si controlla se esiste una run
salvata → se sì, riprende; se no, mostra `StarterSelect`.

---

## Ordine di implementazione

1. `core/generator.rs` — generazione nemici (base della logica di gioco)
2. `pages/game.rs` — scheletro con context e switch su RunPhase
3. `components/game/starter_select.rs`
4. `components/game/battle_screen.rs`
5. `components/game/post_battle.rs`
6. `components/game/catch_screen.rs`
7. `components/game/shop_screen.rs`
8. `components/game/pokecenter_screen.rs`
9. `components/game/progress_popup.rs`
10. `components/game/game_over.rs` + `run_complete.rs`
11. Persistenza IDB integrata nei punti di transizione
