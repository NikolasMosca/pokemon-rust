use leptos::prelude::*;
use game_core::battle::ai::choose_foe_move;
use game_core::battle::damage::calculate_damage;
use game_core::battle::rng::Rng;
use game_core::battle::turn::{execute_turn, TurnAction, TurnResult};
use game_core::inventory::ItemKind;
use game_core::run::rewards::BattleKind;
use crate::components::attack_anim::{AttackAnim, orb_color};
use crate::components::battle_layout::BattleLayout;
use crate::components::damage_number::{DamageKind, DamageNumber};
use game_core::moves::MoveCategory;
use crate::components::pokemon_card::PokemonCard;
use crate::components::pokemon_select_card::PokemonSelectCard;
use crate::components::type_badge::TypeBadge;
use crate::core::cache::use_pokemon_cache;
use crate::core::generator::{generate_gym_leader_team, generate_trainer_team, generate_wild_pokemon, TURN_DELAY_MS};
use crate::pages::game::use_game_context;
use crate::components::game::post_battle::{PostBattleResult, use_last_battle_result};
use crate::components::game::catch_screen::use_catch_candidate;
use crate::audio::{use_audio, music::MusicTrack, sfx::Sfx};

#[derive(Clone, PartialEq)]
enum ActionPanel {
    Main,
    // Modal overlay
    FightModal,                 // modal selezione mossa
    MoveInfo(usize),            // modal dettaglio singola mossa
    BagModal,                   // modal borsa
    BagTargetSelect(ItemKind),  // modal selezione target oggetto
    PokemonModal { forced: bool }, // modal selezione Pokémon
}

#[component]
pub fn BattleScreen(kind: BattleKind) -> impl IntoView {
    let ctx = use_game_context();
    let audio = use_audio();
    let cache = use_pokemon_cache();
    let last_result = use_last_battle_result();
    let catch_candidate = use_catch_candidate();

    // Signal dal GameContext — sopravvivono alla transizione di fase
    let bs = ctx.battle.clone();
    let enemy        = bs.enemy;
    let enemy_team   = bs.enemy_team;
    let enemy_sprite = bs.enemy_sprite;
    let player_sprite = bs.player_sprite;
    let team_sprites = bs.team_sprites;
    let active_idx   = bs.active_idx;
    let dialog       = bs.dialog;
    let damage_event = bs.damage_event;
    let foe_damage_event = bs.foe_damage_event;
    let input_locked = bs.input_locked;
    let player_fainted      = bs.player_fainted;
    let player_attack_anim  = bs.player_attack_anim;
    let enemy_attack_anim   = bs.enemy_attack_anim;
    let hit_flash           = bs.hit_flash;

    // Panel è locale — non serve sopravvivere fuori dalla battaglia
    let panel = RwSignal::new(ActionPanel::Main);

    // Reset signal e genera nemico all'avvio
    {
        let cache = cache.clone();
        let kind = kind.clone();
        let audio = audio.clone();
        enemy.set(None);
        enemy_team.set(Vec::new());
        input_locked.set(true);
        player_fainted.set(false);
        player_attack_anim.set(None);
        enemy_attack_anim.set(None);
        hit_flash.set(false);
        panel.set(ActionPanel::Main);
        damage_event.set(None);
        foe_damage_event.set(None);

        leptos::task::spawn_local(async move {
            let avg = ctx.run.with(|r| r.as_ref().map(|r| r.team_avg_level()).unwrap_or(5));
            let team_size = ctx.run.with(|r| r.as_ref().map(|r| r.team.len()).unwrap_or(1));
            let mut rng = Rng::new(js_sys::Date::now() as u64);

            let result = match &kind {
                BattleKind::Wild =>
                    generate_wild_pokemon(avg, &mut rng, &cache).await.map(|p| vec![p]),
                BattleKind::Trainer =>
                    generate_trainer_team(avg, team_size, &mut rng, &cache).await,
                BattleKind::GymLeader =>
                    generate_gym_leader_team(avg, team_size, &mut rng, &cache).await,
            };

            match result {
                Ok(mut team) if !team.is_empty() => {
                    let first = team.remove(0);
                    if !team.is_empty() {
                        web_sys::console::log_1(&format!(
                            "[BATTLE] team avversario: {} Pokémon rimanenti dopo il primo",
                            team.len()
                        ).into());
                        enemy_team.set(team);
                    }
                    web_sys::console::log_1(&format!(
                        "[BATTLE] nemico generato: {} lv{} hp={} mosse=[{}]",
                        first.name, first.level, first.max_hp(),
                        first.moves.iter().map(|m| format!("{}(pp={}/{})", m.name, m.current_pp, m.max_pp)).collect::<Vec<_>>().join(", ")
                    ).into());
                    if let Ok(data) = cache.fetch(&first.name).await {
                        enemy_sprite.set(data.sprites.front_default);
                    }
                    enemy.set(Some(first));

                    // Mantieni il Pokémon che era in campo se è ancora vivo,
                    // altrimenti prendi il primo disponibile.
                    let idx = ctx.run.with(|r| {
                        let current = active_idx.get_untracked();
                        r.as_ref().and_then(|r| {
                            if r.team.get(current).map(|p| !p.is_fainted()).unwrap_or(false) {
                                Some(current)
                            } else {
                                r.team.iter().position(|p| !p.is_fainted())
                            }
                        }).unwrap_or(0)
                    });
                    active_idx.set(idx);

                    // Fetcha gli sprite di tutto il team per il modal selezione
                    let player_names = ctx.run.with(|r| {
                        r.as_ref().map(|r| r.team.iter().map(|p| p.name.clone()).collect::<Vec<_>>())
                            .unwrap_or_default()
                    });
                    let mut sprites = vec![None; player_names.len()];
                    for (i, name) in player_names.iter().enumerate() {
                        if let Ok(data) = cache.fetch(name).await {
                            sprites[i] = data.sprites.back_default;
                        }
                    }
                    // Sprite del player attivo in campo = back sprite
                    let active_sprite = sprites.get(idx).and_then(|s| s.clone());
                    player_sprite.set(active_sprite);
                    team_sprites.set(sprites);

                    web_sys::console::log_1(&format!(
                        "[BATTLE] player attivo: idx={idx}"
                    ).into());

                    // ── Audio: cry nemico → cry player → musica ──────────
                    audio.init();
                    let enemy_id = enemy.with(|e| e.as_ref().and_then(|e| e.pokedex_id));
                    let player_id = ctx.run.with(|r| r.as_ref()
                        .and_then(|r| r.team.get(idx))
                        .and_then(|p| p.pokedex_id));
                    let music_track = match &kind {
                        BattleKind::Wild      => MusicTrack::BattleWild,
                        BattleKind::Trainer   => MusicTrack::BattleTrainer,
                        BattleKind::GymLeader => MusicTrack::BattleGym,
                    };
                    audio.play_music(music_track);
                    if let Some(id) = enemy_id { audio.play_cry(id); }
                    gloo_timers::future::TimeoutFuture::new(1200).await;
                    if let Some(id) = player_id { audio.play_cry(id); }

                    dialog.set("Cosa farà il tuo Pokémon?".to_string());
                    input_locked.set(false);
                }
                _ => {
                    web_sys::console::log_1(&"[BATTLE] ERRORE: generazione nemico fallita".into());
                    dialog.set("Errore nella generazione del nemico.".to_string());
                }
            }
        });
    }

    let kind_for_label = kind.clone();
    let cache_for_end_battle = cache.clone();
    let audio_for_execute = audio.clone();

    let execute_move = StoredValue::new(move |move_idx: usize| {
        let audio = audio_for_execute.clone();
        // 1. Leggi tutto lo stato necessario PRIMA di modificare qualsiasi signal.
        let (mut player_snap, foe_snap) = {
            let run_r = ctx.run.read();
            let enemy_r = enemy.read();
            let run = match run_r.as_ref() { Some(r) => r, None => {
                web_sys::console::log_1(&"[BATTLE] execute_move: run è None, aborto".into());
                return;
            }};
            let foe = match enemy_r.as_ref() { Some(f) => f, None => {
                web_sys::console::log_1(&"[BATTLE] execute_move: enemy è None, aborto".into());
                return;
            }};
            let idx = active_idx.get();
            let player = match run.team.get(idx) { Some(p) => p.clone(), None => {
                web_sys::console::log_1(&format!("[BATTLE] execute_move: team[{idx}] non esiste, aborto").into());
                return;
            }};
            (player, foe.clone())
        };

        const FALLBACK_MOVE: game_core::moves::Move = game_core::moves::Move::new(
            "Tackle", game_core::types::Type::Normal,
            game_core::moves::MoveCategory::Physical, 40, 100, 35,
        );

        let move_name = player_snap.moves.get(move_idx).map(|m| m.name.clone()).unwrap_or_else(|| "FALLBACK");
        web_sys::console::log_1(&format!(
            "[BATTLE] TURNO INIZIA — player={} lv{} hp={}/{} | foe={} lv{} hp={}/{} | mossa=[{move_idx}]{move_name}",
            player_snap.name, player_snap.level, player_snap.current_hp, player_snap.max_hp(),
            foe_snap.name, foe_snap.level, foe_snap.current_hp, foe_snap.max_hp(),
        ).into());

        // 2. Esegui il turno su copie locali.
        let mut foe_snap_mut = foe_snap.clone();
        let mut rng_move = Rng::new(js_sys::Date::now() as u64 ^ 0xDEAD);
        let foe_move = choose_foe_move(&foe_snap_mut, &mut rng_move);
        web_sys::console::log_1(&format!(
            "[BATTLE] foe usa mossa: {} (power={}, type={:?}) | player speed={} foe speed={}",
            foe_move.name, foe_move.power, foe_move.move_type,
            player_snap.base_stats.speed, foe_snap.base_stats.speed,
        ).into());

        let mut rng = Rng::new(js_sys::Date::now() as u64);
        let out = execute_turn(
            &mut player_snap, &mut foe_snap_mut,
            TurnAction::UseMove(move_idx), &foe_move, &mut rng,
        );

        // 3. Estrai tutti i valori dal risultato prima di toccare i signal.
        let p_dmg = out.player_hit.damage;
        let e_dmg = out.enemy_hit.damage;
        let p_heal = out.healed_player;
        let e_heal = out.healed_enemy;
        let result = out.result.clone();
        let player_first = foe_snap.base_stats.speed <= player_snap.base_stats.speed;
        let foe_hp_before = foe_snap.current_hp;
        let foe_max_hp = foe_snap.max_hp();
        let foe_name = foe_snap.name.clone();
        let pname = player_snap.name.clone();
        let kind = kind.clone();

        // Determina le animazioni in base alla mossa usata dal player e dal foe.
        let player_anim: AttackAnim = player_snap.moves.get(move_idx)
            .map(|m| match m.category {
                MoveCategory::Physical => AttackAnim::Physical,
                MoveCategory::Special  => AttackAnim::Special { color: orb_color(&m.move_type) },
                MoveCategory::Status   => AttackAnim::Heal,
            })
            .unwrap_or(AttackAnim::Physical);

        let foe_anim: AttackAnim = match foe_move.category {
            MoveCategory::Physical => AttackAnim::Physical,
            MoveCategory::Special  => AttackAnim::Special { color: orb_color(&foe_move.move_type) },
            MoveCategory::Status   => AttackAnim::Heal,
        };

        const ANIM_MS: u32 = 400;
        const ORB_MS: u32  = 350;
        const FLASH_MS: u32 = 120;

        // SFX da riprodurre all'impatto in base a categoria e effectiveness.
        let player_hit_sfx = hit_sfx_for(&player_anim, out.enemy_hit.effectiveness, out.enemy_hit.damage);
        let foe_hit_sfx    = hit_sfx_for(&foe_anim,   out.player_hit.effectiveness, out.player_hit.damage);

        web_sys::console::log_1(&format!(
            "[BATTLE] RISULTATO — player_first={player_first} | danno_a_foe={e_dmg} (crit={} eff={}) | danno_a_player={p_dmg} (crit={} eff={}) | foe_hp_prima={foe_hp_before} foe_hp_dopo={} | player_hp_dopo={} | result={:?}",
            out.enemy_hit.is_crit, out.enemy_hit.effectiveness,
            out.player_hit.is_crit, out.player_hit.effectiveness,
            foe_snap_mut.current_hp, player_snap.current_hp,
            result,
        ).into());

        // 4. Scrivi i nuovi stati nei signal in un unico batch.
        // ctx.run.update() aggiorna solo il team — la fase rimane InBattle.
        // Il Memo<phase> in game.rs non riesegue perché la fase non cambia,
        // quindi BattleScreen NON viene rimontato.
        input_locked.set(true);
        ctx.run.update(|r| {
            let Some(run) = r else { return };
            let idx = active_idx.get_untracked();
            if let Some(p) = run.team.get_mut(idx) {
                *p = player_snap.clone();
            }
        });
        enemy.update(|e| { *e = Some(foe_snap_mut.clone()); });

        web_sys::console::log_1(&format!(
            "[BATTLE] signal aggiornati — chiamo spawn_local per animazione",
        ).into());

        // 5. Mostra animazione e poi risolvi il risultato.
        leptos::task::spawn_local(async move {
            if player_first {
                // ── Player attacca ──────────────────────────────────────
                dialog.set(format!("{pname} ha attaccato!"));
                player_attack_anim.set(Some(player_anim.clone()));
                let anim_wait = if matches!(player_anim, AttackAnim::Special { .. }) { ORB_MS } else { ANIM_MS };
                gloo_timers::future::TimeoutFuture::new(anim_wait).await;
                if let Some(sfx) = player_hit_sfx { audio.play_sfx(sfx); }
                if e_dmg > 0 {
                    hit_flash.set(true);
                    enemy_attack_anim.set(Some(AttackAnim::Hit));
                    foe_damage_event.set(Some((e_dmg as i32, DamageKind::Damage)));
                    gloo_timers::future::TimeoutFuture::new(FLASH_MS).await;
                    hit_flash.set(false);
                }
                if p_heal > 0 {
                    audio.play_sfx(Sfx::PokemonHealed);
                    damage_event.set(Some((p_heal as i32, DamageKind::Heal)));
                    gloo_timers::future::TimeoutFuture::new(TURN_DELAY_MS).await;
                } else {
                    gloo_timers::future::TimeoutFuture::new(TURN_DELAY_MS - if e_dmg > 0 { FLASH_MS } else { 0 }).await;
                }
                player_attack_anim.set(None);
                enemy_attack_anim.set(None);
                foe_damage_event.set(None);
                damage_event.set(None);

                if result == TurnResult::PlayerWon {
                    end_battle(&kind, ctx.run, enemy, enemy_team, enemy_sprite, cache_for_end_battle.clone(), last_result, catch_candidate, dialog, input_locked, audio.clone());
                    return;
                }

                // ── Foe contrattacca ────────────────────────────────────
                dialog.set(format!("{foe_name} ha contrattaccato!"));
                enemy_attack_anim.set(Some(foe_anim.clone()));
                let foe_wait = if matches!(foe_anim, AttackAnim::Special { .. }) { ORB_MS } else { ANIM_MS };
                gloo_timers::future::TimeoutFuture::new(foe_wait).await;
                if let Some(sfx) = foe_hit_sfx { audio.play_sfx(sfx); }
                if p_dmg > 0 {
                    hit_flash.set(true);
                    player_attack_anim.set(Some(AttackAnim::Hit));
                    damage_event.set(Some((p_dmg as i32, DamageKind::Damage)));
                    gloo_timers::future::TimeoutFuture::new(FLASH_MS).await;
                    hit_flash.set(false);
                }
                if e_heal > 0 {
                    enemy_attack_anim.set(Some(AttackAnim::Heal));
                    foe_damage_event.set(Some((e_heal as i32, DamageKind::Heal)));
                    gloo_timers::future::TimeoutFuture::new(TURN_DELAY_MS).await;
                } else {
                    gloo_timers::future::TimeoutFuture::new(TURN_DELAY_MS - if p_dmg > 0 { FLASH_MS } else { 0 }).await;
                }
                enemy_attack_anim.set(None);
                player_attack_anim.set(None);
                foe_damage_event.set(None);
                damage_event.set(None);
            } else {
                // ── Foe attacca per primo ───────────────────────────────
                dialog.set(format!("{foe_name} ha attaccato per primo!"));
                enemy_attack_anim.set(Some(foe_anim.clone()));
                let foe_wait = if matches!(foe_anim, AttackAnim::Special { .. }) { ORB_MS } else { ANIM_MS };
                gloo_timers::future::TimeoutFuture::new(foe_wait).await;
                if let Some(sfx) = foe_hit_sfx { audio.play_sfx(sfx); }
                if p_dmg > 0 {
                    hit_flash.set(true);
                    player_attack_anim.set(Some(AttackAnim::Hit));
                    damage_event.set(Some((p_dmg as i32, DamageKind::Damage)));
                    gloo_timers::future::TimeoutFuture::new(FLASH_MS).await;
                    hit_flash.set(false);
                }
                if e_heal > 0 {
                    enemy_attack_anim.set(Some(AttackAnim::Heal));
                    foe_damage_event.set(Some((e_heal as i32, DamageKind::Heal)));
                    gloo_timers::future::TimeoutFuture::new(TURN_DELAY_MS).await;
                } else {
                    gloo_timers::future::TimeoutFuture::new(TURN_DELAY_MS - if p_dmg > 0 { FLASH_MS } else { 0 }).await;
                }
                enemy_attack_anim.set(None);
                player_attack_anim.set(None);
                foe_damage_event.set(None);
                damage_event.set(None);

                if result == TurnResult::EnemyWon {
                    audio.play_sfx(Sfx::Faint);
                    handle_player_fainted(ctx.run, active_idx, panel, dialog, input_locked, player_fainted);
                    return;
                }

                // ── Player contrattacca ─────────────────────────────────
                dialog.set(format!("{pname} ha contrattaccato!"));
                player_attack_anim.set(Some(player_anim.clone()));
                let anim_wait = if matches!(player_anim, AttackAnim::Special { .. }) { ORB_MS } else { ANIM_MS };
                gloo_timers::future::TimeoutFuture::new(anim_wait).await;
                if let Some(sfx) = player_hit_sfx { audio.play_sfx(sfx); }
                if e_dmg > 0 {
                    hit_flash.set(true);
                    enemy_attack_anim.set(Some(AttackAnim::Hit));
                    foe_damage_event.set(Some((e_dmg as i32, DamageKind::Damage)));
                    gloo_timers::future::TimeoutFuture::new(FLASH_MS).await;
                    hit_flash.set(false);
                }
                if p_heal > 0 {
                    audio.play_sfx(Sfx::PokemonHealed);
                    damage_event.set(Some((p_heal as i32, DamageKind::Heal)));
                    gloo_timers::future::TimeoutFuture::new(TURN_DELAY_MS).await;
                } else {
                    gloo_timers::future::TimeoutFuture::new(TURN_DELAY_MS - if e_dmg > 0 { FLASH_MS } else { 0 }).await;
                }
                player_attack_anim.set(None);
                enemy_attack_anim.set(None);
                foe_damage_event.set(None);
                damage_event.set(None);
            }

            match result {
                TurnResult::PlayerWon => {
                    end_battle(&kind, ctx.run, enemy, enemy_team, enemy_sprite, cache_for_end_battle.clone(), last_result, catch_candidate, dialog, input_locked, audio.clone());
                }
                TurnResult::EnemyWon => {
                    audio.play_sfx(Sfx::Faint);
                    handle_player_fainted(ctx.run, active_idx, panel, dialog, input_locked, player_fainted);
                }
                _ => {
                    dialog.set("Cosa farà il tuo Pokémon?".to_string());
                    input_locked.set(false);
                }
            }
        });
    });

    let audio_for_item = audio.clone();
    let use_item = StoredValue::new(move |item: ItemKind, target_idx: usize, move_idx: Option<usize>| {
        let audio = audio_for_item.clone();
        input_locked.set(true);
        panel.set(ActionPanel::Main);

        let item_name = item.name();
        let used = {
            let mut guard = ctx.run.write();
            guard.as_mut().and_then(|run| {
                let target = run.team.get_mut(target_idx)?;
                run.inventory.use_item(&item, target, move_idx).ok().map(|_| item_name)
            })
        };
        if let Some(name) = used {
            audio.play_sfx(Sfx::PokemonHealed);
            dialog.set(format!("Hai usato {name}!"));
        }

        if let Some(foe) = enemy.get() {
            let mut rng_ai = Rng::new(js_sys::Date::now() as u64 ^ 0xBEEF);
            let foe_move = choose_foe_move(&foe, &mut rng_ai);
            let foe_name = foe.name.clone();
            leptos::task::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(TURN_DELAY_MS).await;
                let dmg = {
                    let mut guard = ctx.run.write();
                    guard.as_mut().and_then(|run| {
                        let active = run.team.get_mut(active_idx.get())?;
                        let mut rng = Rng::new(js_sys::Date::now() as u64);
                        let res = calculate_damage(&foe, active, &foe_move, &mut rng);
                        active.take_damage(res.damage);
                        Some(res.damage)
                    }).unwrap_or(0)
                };
                damage_event.set(Some((dmg as i32, DamageKind::Damage)));
                dialog.set(format!("{foe_name} ha attaccato!"));
                gloo_timers::future::TimeoutFuture::new(TURN_DELAY_MS).await;
                damage_event.set(None);
                let fainted = ctx.run.with(|r| r.as_ref().and_then(|r| r.team.get(active_idx.get()).map(|p| p.is_fainted())).unwrap_or(false));
                if fainted {
                    handle_player_fainted(ctx.run, active_idx, panel, dialog, input_locked, player_fainted);
                } else {
                    dialog.set("Cosa farà il tuo Pokémon?".to_string());
                    input_locked.set(false);
                }
            });
            return;
        }
        dialog.set("Cosa farà il tuo Pokémon?".to_string());
        input_locked.set(false);
    });

    let switch_pokemon = StoredValue::new(move |new_idx: usize, forced: bool| {
        active_idx.set(new_idx);
        player_fainted.set(false);
        panel.set(ActionPanel::Main);
        let name = ctx.run.with(|r| r.as_ref().and_then(|r| r.team.get(new_idx).map(|p| p.name.clone())).unwrap_or_default());
        dialog.set(format!("Vai, {name}!"));
        // Usa lo sprite già cachato se disponibile, altrimenti fetcha
        let cached = team_sprites.with(|v| v.get(new_idx).and_then(|s| s.clone()));
        if let Some(url) = cached {
            player_sprite.set(Some(url));
        } else {
            let cache = cache.clone();
            let name_clone = name.clone();
            leptos::task::spawn_local(async move {
                if let Ok(data) = cache.fetch(&name_clone).await {
                    player_sprite.set(data.sprites.back_default);
                }
            });
        }
        if forced { input_locked.set(false); return; }

        if let Some(foe) = enemy.get() {
            let mut rng_ai = Rng::new(js_sys::Date::now() as u64 ^ 0xCAFE);
            let foe_move = choose_foe_move(&foe, &mut rng_ai);
            let foe_name = foe.name.clone();
            leptos::task::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(TURN_DELAY_MS).await;
                let dmg = {
                    let mut guard = ctx.run.write();
                    guard.as_mut().and_then(|run| {
                        let active = run.team.get_mut(new_idx)?;
                        let mut rng = Rng::new(js_sys::Date::now() as u64);
                        let res = calculate_damage(&foe, active, &foe_move, &mut rng);
                        active.take_damage(res.damage);
                        Some(res.damage)
                    }).unwrap_or(0)
                };
                damage_event.set(Some((dmg as i32, DamageKind::Damage)));
                dialog.set(format!("{foe_name} ha attaccato!"));
                gloo_timers::future::TimeoutFuture::new(TURN_DELAY_MS).await;
                damage_event.set(None);

                let is_fainted = ctx.run.with(|r| {
                    r.as_ref().and_then(|r| r.team.get(new_idx).map(|p| p.is_fainted())).unwrap_or(false)
                });
                if is_fainted {
                    handle_player_fainted(ctx.run, active_idx, panel, dialog, input_locked, player_fainted);
                } else {
                    dialog.set("Cosa farà il tuo Pokémon?".to_string());
                    input_locked.set(false);
                }
            });
            return;
        }
        input_locked.set(false);
    });

    let kind_label = match &kind_for_label {
        BattleKind::Wild      => ("🌿 Pokémon Selvatico", "battle-kind-badge battle-kind-badge--wild"),
        BattleKind::Trainer   => ("👤 Allenatore",         "battle-kind-badge battle-kind-badge--trainer"),
        BattleKind::GymLeader => ("🏆 Capopalestra",       "battle-kind-badge battle-kind-badge--gym"),
    };

    // Sprite trainer da Pokémon Showdown — ciclici per gym/trainer index
    const TRAINER_SPRITES: &[&str] = &[
        "youngster", "lass", "camper", "picnicker", "bugcatcher",
        "hiker", "swimmer-m", "guitarist", "beauty", "acetrainer-m",
        "acetrainer-f", "blackbelt", "engineer", "gentleman",
        "cooltrainer-m", "cooltrainer-f", "sailor", "biker",
        "rocker", "juggler",
    ];
    const GYM_LEADER_SPRITES: &[&str] = &[
        "brock", "misty", "lt.surge", "erika", "koga",
        "sabrina", "blaine", "giovanni",
    ];

    let (trainer_sprite_sig, trainer_label_sig) = match &kind_for_label {
        BattleKind::Wild => (None, None),
        BattleKind::Trainer => {
            let (gym_idx, trainer_num) = ctx.run.with(|r| r.as_ref()
                .map(|r| (r.gym.gym_index as usize, r.gym.trainers_defeated as usize))
                .unwrap_or((0, 0)));
            let sprite_idx = (gym_idx * 3 + trainer_num) % TRAINER_SPRITES.len();
            let sprite_name = TRAINER_SPRITES[sprite_idx];
            let url = format!("https://play.pokemonshowdown.com/sprites/trainers/{sprite_name}.png");
            let label = format!("Allenatore {}", trainer_num + 1);
            (
                Some(Signal::derive(move || Some(url.clone()))),
                Some(Signal::derive(move || Some(label.clone()))),
            )
        }
        BattleKind::GymLeader => {
            let gym_idx = ctx.run.with(|r| r.as_ref().map(|r| r.gym.gym_index as usize).unwrap_or(0));
            let sprite_name = GYM_LEADER_SPRITES[gym_idx % GYM_LEADER_SPRITES.len()];
            let url = format!("https://play.pokemonshowdown.com/sprites/trainers/{sprite_name}.png");
            let gym_num = gym_idx + 1;
            let label = format!("Capopalestra {gym_num}");
            (
                Some(Signal::derive(move || Some(url.clone()))),
                Some(Signal::derive(move || Some(label.clone()))),
            )
        }
    };

    view! {
        <div class="battle-screen">
            <div class=kind_label.1>{kind_label.0}</div>
            <BattleLayout
                trainer_sprite=trainer_sprite_sig.unwrap_or_else(|| Signal::derive(|| None))
                trainer_label=trainer_label_sig.unwrap_or_else(|| Signal::derive(|| None))
                enemy_card=view! {
                    <PokemonCard
                        name=Signal::derive(move || enemy.with(|e| e.as_ref().map(|e| e.name.clone()).unwrap_or_default()))
                        level=Signal::derive(move || enemy.with(|e| e.as_ref().map(|e| e.level as u32).unwrap_or(5)))
                        hp=Signal::derive(move || enemy.with(|e| e.as_ref().map(|e| e.current_hp).unwrap_or(0)))
                        max_hp=Signal::derive(move || enemy.with(|e| e.as_ref().map(|e| e.max_hp()).unwrap_or(1)))
                        pokemon_type=Signal::derive(move || enemy.with(|e| e.as_ref().map(|e| e.primary_type)))
                    />
                }
                enemy_sprite=Signal::derive(move || enemy_sprite.get())
                player_card=view! {
                    <PokemonCard
                        name=Signal::derive(move || ctx.run.with(|r| r.as_ref().and_then(|r| r.team.get(active_idx.get()).map(|p| p.name.clone())).unwrap_or_default()))
                        level=Signal::derive(move || ctx.run.with(|r| r.as_ref().and_then(|r| r.team.get(active_idx.get()).map(|p| p.level as u32)).unwrap_or(5)))
                        hp=Signal::derive(move || ctx.run.with(|r| r.as_ref().and_then(|r| r.team.get(active_idx.get()).map(|p| p.current_hp)).unwrap_or(0)))
                        max_hp=Signal::derive(move || ctx.run.with(|r| r.as_ref().and_then(|r| r.team.get(active_idx.get()).map(|p| p.max_hp())).unwrap_or(1)))
                        is_player=true
                        pokemon_type=Signal::derive(move || ctx.run.with(|r| r.as_ref().and_then(|r| r.team.get(active_idx.get()).map(|p| p.primary_type))))
                    />
                }
                player_sprite=Signal::derive(move || player_sprite.get())
                player_fainted=Signal::derive(move || player_fainted.get())
                player_attack_anim=Signal::derive(move || player_attack_anim.get())
                enemy_attack_anim=Signal::derive(move || enemy_attack_anim.get())
                hit_flash=Signal::derive(move || hit_flash.get())
                damage=view! { <DamageNumber value=damage_event.read_only() /> }
                foe_damage=view! { <DamageNumber value=foe_damage_event.read_only() /> }
                dialog=view! {
                    <span class="text-preline">{move || dialog.get()}</span>
                }
                actions=view! {
                    // ── Modal mosse ───────────────────────────────────────────
                    {move || {
                        if let ActionPanel::FightModal = panel.get() {
                            let moves = ctx.run.with(|r| r.as_ref().and_then(|r| r.team.get(active_idx.get()).map(|p| p.moves.clone())).unwrap_or_default());
                            return view! {
                                <div class="battle-modal-overlay" on:click=move |_| panel.set(ActionPanel::Main)>
                                    <div class="battle-modal" on:click=|e| e.stop_propagation()>
                                        <h3 class="battle-modal__title">"⚔ Mosse"</h3>
                                        {moves.into_iter().enumerate().map(|(i, m)| {
                                            let no_pp = m.current_pp == 0;
                                            let type_name = format!("{:?}", m.move_type).to_lowercase();
                                            let power_label = if m.power > 0 { m.power.to_string() } else { "—".to_string() };
                                            view! {
                                                <div class="battle-move-row">
                                                    <button class="battle-btn battle-btn--move"
                                                        disabled=move || no_pp || input_locked.get()
                                                        on:click=move |_| {
                                                            panel.set(ActionPanel::Main);
                                                            leptos::task::spawn_local(async move {
                                                                execute_move.get_value()(i);
                                                            });
                                                        }>
                                                        <TypeBadge type_name=type_name />
                                                        <span class="battle-btn__move-name">{m.name}</span>
                                                        <span class="battle-btn__move-power">"PWR "{power_label}</span>
                                                        <span class="battle-btn__move-pp">{m.current_pp}"/" {m.max_pp}</span>
                                                    </button>
                                                    <button class="battle-btn battle-btn--info"
                                                        on:click=move |_| panel.set(ActionPanel::MoveInfo(i))
                                                        title="Info mossa">"ⓘ"</button>
                                                </div>
                                            }
                                        }).collect_view()}
                                        <button class="battle-modal__close" on:click=move |_| panel.set(ActionPanel::Main)>"Chiudi"</button>
                                    </div>
                                </div>
                            }.into_any();
                        }
                        view! { <></> }.into_any()
                    }}

                    // ── Modal dettaglio mossa ─────────────────────────────────
                    {move || {
                        if let ActionPanel::MoveInfo(idx) = panel.get() {
                            let moves = ctx.run.with(|r| r.as_ref().and_then(|r| r.team.get(active_idx.get()).map(|p| p.moves.clone())).unwrap_or_default());
                            if let Some(m) = moves.get(idx).cloned() {
                                let type_name = format!("{:?}", m.move_type).to_lowercase();
                                let category = format!("{:?}", m.category);
                                return view! {
                                    <div class="battle-modal-overlay" on:click=move |_| panel.set(ActionPanel::FightModal)>
                                        <div class="battle-modal" on:click=|e| e.stop_propagation()>
                                            <div class="battle-modal__header">
                                                <span class="battle-modal__move-name">{m.name}</span>
                                                <TypeBadge type_name=type_name />
                                            </div>
                                            <div class="battle-modal__row">
                                                <span class="battle-modal__label">"Categoria"</span>
                                                <span class="battle-modal__value">{category}</span>
                                            </div>
                                            <div class="battle-modal__row">
                                                <span class="battle-modal__label">"Potenza"</span>
                                                <span class="battle-modal__value">
                                                    {if m.power > 0 { m.power.to_string() } else { "—".to_string() }}
                                                </span>
                                            </div>
                                            <div class="battle-modal__row">
                                                <span class="battle-modal__label">"Precisione"</span>
                                                <span class="battle-modal__value">{m.accuracy}"%"</span>
                                            </div>
                                            <div class="battle-modal__row">
                                                <span class="battle-modal__label">"PP"</span>
                                                <span class="battle-modal__value">{m.max_pp}</span>
                                            </div>
                                            <button class="battle-modal__close" on:click=move |_| panel.set(ActionPanel::FightModal)>"← Indietro"</button>
                                        </div>
                                    </div>
                                }.into_any();
                            }
                        }
                        view! { <></> }.into_any()
                    }}

                    // ── Modal borsa ───────────────────────────────────────────
                    {move || {
                        if let ActionPanel::BagModal = panel.get() {
                            let inv: Vec<_> = ctx.run.with(|r| r.as_ref().map(|r| r.inventory.items.clone()).unwrap_or_default()).into_iter().collect();
                            let has_fainted = ctx.run.with(|r| r.as_ref().map(|r| r.team.iter().any(|p| p.is_fainted())).unwrap_or(false));
                            return view! {
                                <div class="battle-modal-overlay" on:click=move |_| panel.set(ActionPanel::Main)>
                                    <div class="battle-modal" on:click=|e| e.stop_propagation()>
                                        <h3 class="battle-modal__title">"🎒 Borsa"</h3>
                                        {if inv.is_empty() {
                                            view! { <p class="battle-bag__empty">"Borsa vuota"</p> }.into_any()
                                        } else {
                                            inv.into_iter().map(|(kind, qty)| {
                                                let disabled = matches!(&kind, ItemKind::Revive) && !has_fainted;
                                                let k = kind.clone();
                                                view! {
                                                    <button class="battle-btn battle-btn--item" disabled=disabled
                                                        on:click=move |_| panel.set(ActionPanel::BagTargetSelect(k.clone()))>
                                                        {kind.name()} " ×" {qty}
                                                    </button>
                                                }
                                            }).collect_view().into_any()
                                        }}
                                        <button class="battle-modal__close" on:click=move |_| panel.set(ActionPanel::Main)>"Chiudi"</button>
                                    </div>
                                </div>
                            }.into_any();
                        }
                        view! { <></> }.into_any()
                    }}

                    // ── Modal selezione target oggetto ────────────────────────
                    {move || {
                        if let ActionPanel::BagTargetSelect(ref item) = panel.get() {
                            let item = item.clone();
                            let team = ctx.run.with(|r| r.as_ref().map(|r| r.team.clone()).unwrap_or_default());
                            return view! {
                                <div class="battle-modal-overlay" on:click=move |_| panel.set(ActionPanel::BagModal)>
                                    <div class="battle-modal" on:click=|e| e.stop_propagation()>
                                        <h3 class="battle-modal__title">"Su quale Pokémon?"</h3>
                                        <div class="battle-modal__psc-list">
                                            {team.into_iter().enumerate().map(|(i, p)| {
                                                let item = item.clone();
                                                let is_revive = matches!(&item, ItemKind::Revive);
                                                let disabled = match &item {
                                                    ItemKind::Revive => !p.is_fainted(),
                                                    ItemKind::Potion | ItemKind::SuperPotion | ItemKind::FullRestore => p.is_fainted(),
                                                    _ => false,
                                                };
                                                view! {
                                                    <PokemonSelectCard
                                                        name=p.name.clone()
                                                        level=p.level as u32
                                                        hp=p.current_hp
                                                        max_hp=p.max_hp()
                                                        img_url=None
                                                        is_fainted=p.is_fainted()
                                                        disabled=disabled
                                                        allow_fainted=is_revive
                                                        pokemon_type=Some(p.primary_type)
                                                        on_click=move || use_item.get_value()(item.clone(), i, None)
                                                    />
                                                }
                                            }).collect_view()}
                                        </div>
                                        <button class="battle-modal__close" on:click=move |_| panel.set(ActionPanel::BagModal)>"← Indietro"</button>
                                    </div>
                                </div>
                            }.into_any();
                        }
                        view! { <></> }.into_any()
                    }}

                    // ── Modal selezione Pokémon ───────────────────────────────
                    {move || {
                        if let ActionPanel::PokemonModal { forced } = panel.get() {
                            let team = ctx.run.with(|r| r.as_ref().map(|r| r.team.clone()).unwrap_or_default());
                            let current = active_idx.get();
                            return view! {
                                <div class="battle-modal-overlay"
                                    on:click=move |_| { if !forced { panel.set(ActionPanel::Main); } }>
                                    <div class="battle-modal" on:click=|e| e.stop_propagation()>
                                        <h3 class="battle-modal__title">"👥 Squadra"</h3>
                                        <div class="battle-modal__psc-list">
                                            {team.into_iter().enumerate().map(|(i, p)| {
                                                let is_active = i == current;
                                                let sprite = team_sprites.with(|v| v.get(i).and_then(|s| s.clone()));
                                                view! {
                                                    <PokemonSelectCard
                                                        name=p.name.clone()
                                                        level=p.level as u32
                                                        hp=p.current_hp
                                                        max_hp=p.max_hp()
                                                        img_url=sprite
                                                        is_active=is_active
                                                        is_fainted=p.is_fainted()
                                                        disabled=is_active
                                                        pokemon_type=Some(p.primary_type)
                                                        on_click=move || switch_pokemon.get_value()(i, forced)
                                                    />
                                                }
                                            }).collect_view()}
                                        </div>
                                        {(!forced).then(|| view! {
                                            <button class="battle-modal__close" on:click=move |_| panel.set(ActionPanel::Main)>"Chiudi"</button>
                                        })}
                                    </div>
                                </div>
                            }.into_any();
                        }
                        view! { <></> }.into_any()
                    }}

                    // ── Pannello principale — 3 pulsanti azione ─
                    <div class="battle-main-actions">
                        <button class="battle-btn battle-btn--fight" disabled=move || input_locked.get()
                            on:click=move |_| panel.set(ActionPanel::FightModal)>
                            <span class="battle-btn__icon">"⚔️"</span>
                            <span class="battle-btn__label">"LOTTA"</span>
                        </button>
                        <button class="battle-btn battle-btn--bag" disabled=move || input_locked.get()
                            on:click=move |_| panel.set(ActionPanel::BagModal)>
                            <span class="battle-btn__icon">"🎒"</span>
                            <span class="battle-btn__label">"BORSA"</span>
                        </button>
                        <button class="battle-btn battle-btn--pokemon" disabled=move || input_locked.get()
                            on:click=move |_| panel.set(ActionPanel::PokemonModal { forced: false })>
                            <span class="battle-btn__icon">"🔴"</span>
                            <span class="battle-btn__label">"POKÉMON"</span>
                        </button>
                    </div>
                }
            />
        </div>
    }
}

fn end_battle(
    kind: &BattleKind,
    run: RwSignal<Option<game_core::run::RunState>>,
    enemy: RwSignal<Option<game_core::pokemon::Pokemon>>,
    enemy_team: RwSignal<Vec<game_core::pokemon::Pokemon>>,
    enemy_sprite: RwSignal<Option<String>>,
    cache: crate::core::cache::PokemonCache,
    last_result: RwSignal<Option<PostBattleResult>>,
    catch_candidate: RwSignal<Option<game_core::pokemon::Pokemon>>,
    dialog: RwSignal<String>,
    input_locked: RwSignal<bool>,
    audio: crate::audio::AudioManager,
) {
    web_sys::console::log_1(&format!(
        "[BATTLE] end_battle INIZIO — kind={kind:?}"
    ).into());

    let foe = match enemy.get() { Some(f) => f, None => {
        web_sys::console::log_1(&"[BATTLE] end_battle: enemy signal è None, aborto".into());
        return;
    }};

    // Controlla se ci sono altri Pokémon nel team avversario.
    let next_foe = enemy_team.with(|t| t.first().cloned());
    if let Some(next) = next_foe {
        // Ancora battaglia — applica la ricompensa intermedia e passa al prossimo Pokémon.
        enemy_team.update(|t| { t.remove(0); });
        run.update(|r| {
            let Some(run) = r else { return };
            run.apply_reward(&foe, kind);
        });
        web_sys::console::log_1(&format!(
            "[BATTLE] end_battle — prossimo nemico: {} lv{} (ancora {} rimanenti)",
            next.name, next.level, enemy_team.with(|t| t.len())
        ).into());
        // Aggiorna sprite prima di mostrare il nuovo nemico
        let next_name = next.name.clone();
        enemy_sprite.set(None);
        enemy.set(Some(next));
        dialog.set("Il nemico manda un altro Pokémon!".to_string());
        leptos::task::spawn_local(async move {
            if let Ok(data) = cache.fetch(&next_name).await {
                enemy_sprite.set(data.sprites.front_default);
            }
            input_locked.set(false);
        });
        return;
    }

    // Tutti i Pokémon avversari sconfitti — on_*_defeated applica ricompensa + cambia fase.
    // Per i selvatici si può sempre catturare dopo la sconfitta.
    let can_catch = matches!(kind, BattleKind::Wild);

    web_sys::console::log_1(&format!(
        "[BATTLE] end_battle — foe={} | can_catch={can_catch}",
        foe.name,
    ).into());

    let phase_prima = run.with(|r| r.as_ref().map(|r| format!("{:?}", r.phase)));
    web_sys::console::log_1(&format!(
        "[BATTLE] end_battle — fase prima di on_*_defeated: {phase_prima:?}"
    ).into());

    run.update(|r| {
        let Some(run) = r else { return };
        match kind {
            BattleKind::Wild => run.on_wild_defeated(&foe),
            BattleKind::Trainer => run.on_trainer_defeated(&foe),
            BattleKind::GymLeader => { let _ = run.on_gym_leader_defeated(&foe); }
        }
    });

    let phase_dopo = run.with(|r| r.as_ref().map(|r| format!("{:?}", r.phase)));
    web_sys::console::log_1(&format!(
        "[BATTLE] end_battle — fase DOPO on_*_defeated: {phase_dopo:?}"
    ).into());

    let reward = {
        let guard = run.read();
        guard.as_ref().map(|_| game_core::run::rewards::calculate_reward(&foe, kind))
    };
    let (exp, money) = reward.map(|r| (r.exp, r.money)).unwrap_or((0, 0));

    // Se catturabile, salva il candidato nel signal — PostBattle lo leggerà per mostrare i pulsanti.
    if can_catch {
        catch_candidate.set(Some(foe));
    }

    // Victory jingle → poi musica pokecenter
    let victory_track = match kind {
        BattleKind::Wild      => MusicTrack::VictoryWild,
        BattleKind::Trainer   => MusicTrack::VictoryTrainer,
        BattleKind::GymLeader => MusicTrack::VictoryTrainer,
    };
    audio.play_music(victory_track);

    last_result.set(Some(PostBattleResult {
        exp_gained: exp,
        money_gained: money,
        levels_gained: 0,
        can_catch,
        battle_kind: kind.clone(),
    }));

    web_sys::console::log_1(&format!(
        "[BATTLE] end_battle — last_result impostato: exp={exp} money={money} can_catch={can_catch}"
    ).into());

    web_sys::console::log_1(&"[BATTLE] end_battle FINE".into());
}

/// Sceglie il SFX di impatto. Ritorna None se il danno è 0 (mossa senza danno diretto).
fn hit_sfx_for(anim: &AttackAnim, effectiveness: f32, damage: u32) -> Option<Sfx> {
    if damage == 0 {
        return None;
    }
    if effectiveness > 1.5 {
        return Some(Sfx::HitSuperEffective);
    }
    if effectiveness < 0.5 {
        return Some(Sfx::HitNotEffective);
    }
    Some(match anim {
        AttackAnim::Special { .. } => Sfx::HitSpecial,
        _ => Sfx::HitPhysical,
    })
}

fn handle_player_fainted(
    run: RwSignal<Option<game_core::run::RunState>>,
    active_idx: RwSignal<usize>,
    panel: RwSignal<ActionPanel>,
    dialog: RwSignal<String>,
    input_locked: RwSignal<bool>,
    player_fainted_sig: RwSignal<bool>,
) {
    web_sys::console::log_1(&"[BATTLE] handle_player_fainted — cerco prossimo Pokémon vivo".into());
    let next = run.with(|r| r.as_ref().and_then(|r| {
        r.team.iter().enumerate().find(|(_, p)| !p.is_fainted()).map(|(i, _)| i)
    }));
    match next {
        Some(_) => {
            web_sys::console::log_1(&"[BATTLE] handle_player_fainted — animazione sconfitta poi modal".into());
            // Mostra animazione sconfitta, poi apre il modal dopo il delay
            player_fainted_sig.set(true);
            dialog.set("Il tuo Pokémon è stato sconfitto!".to_string());
            leptos::task::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(900).await;
                panel.set(ActionPanel::PokemonModal { forced: true });
                dialog.set("Scegli il prossimo Pokémon!".to_string());
                input_locked.set(false);
            });
        }
        None => {
            web_sys::console::log_1(&"[BATTLE] handle_player_fainted — nessun Pokémon vivo, chiamo check_game_over".into());
            run.update(|r| { if let Some(run) = r { run.check_game_over(); } });
        }
    }
}
