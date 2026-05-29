use leptos::prelude::*;
use game_core::run::{RunPhase, MAX_TEAM_SIZE};
use game_core::run::rewards::BattleKind;
use game_core::run::gym::POKECENTER_MAX_PER_GYM;
use crate::components::game::catch_screen::use_catch_candidate;
use crate::pages::game::use_game_context;

#[derive(Clone)]
pub struct PostBattleResult {
    pub exp_gained: u32,
    pub money_gained: u32,
    pub levels_gained: u32,
    pub can_catch: bool,
    pub battle_kind: BattleKind,
}

pub fn use_last_battle_result() -> RwSignal<Option<PostBattleResult>> {
    use_context::<RwSignal<Option<PostBattleResult>>>()
        .expect("PostBattleResult signal non trovato nel context")
}

#[component]
pub fn PostBattle() -> impl IntoView {
    let ctx = use_game_context();
    let result = use_last_battle_result();
    let catch_candidate = use_catch_candidate();

    // true dopo che il giocatore ha scelto cattura o lascia perdere — disabilita i pulsanti
    let catch_decided = RwSignal::new(false);
    // true quando il team è pieno e bisogna scegliere quale pokemon sostituire
    let replace_mode = RwSignal::new(false);

    let on_catch = move |_| {
        let team_full = ctx.run.with(|r| {
            r.as_ref().map(|r| r.team.len() >= MAX_TEAM_SIZE).unwrap_or(false)
        });
        if team_full {
            replace_mode.set(true);
        } else {
            let Some(mut pokemon) = catch_candidate.get() else { return };
            catch_decided.set(true);
            ctx.run.update(|r| {
                let Some(run) = r else { return };
                pokemon.full_heal();
                let _ = run.catch(pokemon);
            });
            catch_candidate.set(None);
        }
    };

    let on_replace = move |slot: usize| {
        let Some(mut pokemon) = catch_candidate.get() else { return };
        catch_decided.set(true);
        replace_mode.set(false);
        ctx.run.update(|r| {
            let Some(run) = r else { return };
            pokemon.full_heal();
            let _ = run.replace_team_slot(slot, pokemon);
        });
        catch_candidate.set(None);
    };

    let on_skip = move |_| {
        catch_decided.set(true);
        replace_mode.set(false);
        catch_candidate.set(None);
    };

    let on_pokecenter = move |_| {
        ctx.run.update(|r| {
            if let Some(run) = r {
                let _ = run.use_pokecenter();
            }
        });
    };

    let on_shop = move |_| {
        ctx.run.update(|r| {
            if let Some(run) = r {
                run.phase = RunPhase::Shop;
            }
        });
    };

    let on_next = move |_| {
        ctx.run.update(|r| {
            if let Some(run) = r {
                run.phase = RunPhase::InBattle {
                    kind: run.gym.next_opponent().into(),
                };
            }
        });
    };

    view! {
        <div class="post-battle">
            <h2 class="post-battle__title">"Battaglia terminata!"</h2>

            {move || result.get().map(|r| view! {
                <div class="post-battle__summary">
                    <div class="post-battle__reward-row">
                        <span class="post-battle__reward-icon">"⭐"</span>
                        <span class="post-battle__reward-label">"EXP guadagnata"</span>
                        <span class="post-battle__reward-value">"+"{r.exp_gained}</span>
                    </div>
                    <div class="post-battle__reward-row">
                        <span class="post-battle__reward-icon">"₽"</span>
                        <span class="post-battle__reward-label">"Soldi guadagnati"</span>
                        <span class="post-battle__reward-value">"+"{r.money_gained}</span>
                    </div>
                    {(r.levels_gained > 0).then(|| view! {
                        <div class="post-battle__levelup">
                            <span>"↑"</span>
                            <span>"Livello aumentato!"</span>
                        </div>
                    })}
                </div>
            })}

            // Sezione cattura — visibile solo se il candidato è disponibile
            {move || {
                let candidate = catch_candidate.get();
                let decided = catch_decided.get();
                let is_replace = replace_mode.get();
                candidate.map(|p| {
                    let pname = p.name.clone();
                    if is_replace {
                        // Mostra la squadra per scegliere quale sostituire
                        let team = ctx.run.with(|r| {
                            r.as_ref().map(|r| r.team.iter().enumerate()
                                .map(|(i, p)| (i, p.name.clone(), p.current_hp, p.max_hp()))
                                .collect::<Vec<_>>()
                            ).unwrap_or_default()
                        });
                        view! {
                            <div class="post-battle__catch">
                                <p class="post-battle__catch-label">
                                    "Squadra piena! Scegli chi sostituire con " <strong>{pname}</strong>":"
                                </p>
                                <div class="post-battle__catch-actions">
                                    {team.into_iter().map(|(i, name, hp, max_hp)| view! {
                                        <button
                                            class="post-battle__btn post-battle__btn--catch"
                                            on:click=move |_| on_replace(i)
                                        >
                                            <span class="post-battle__btn-icon">"🔄"</span>
                                            <div class="post-battle__btn-text">
                                                {name.clone()}
                                                <div class="post-battle__btn-sub">
                                                    {format!("HP {hp}/{max_hp}")}
                                                </div>
                                            </div>
                                        </button>
                                    }).collect_view()}
                                    <button
                                        class="post-battle__btn post-battle__btn--skip"
                                        on:click=on_skip
                                    >
                                        <span class="post-battle__btn-icon">"✕"</span>
                                        <span class="post-battle__btn-text">"Annulla"</span>
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="post-battle__catch">
                                <p class="post-battle__catch-label">
                                    "Vuoi aggiungere " <strong>{pname}</strong> " alla tua squadra?"
                                </p>
                                <div class="post-battle__catch-actions">
                                    <button
                                        class="post-battle__btn post-battle__btn--catch"
                                        disabled=decided
                                        on:click=on_catch
                                    >
                                        <span class="post-battle__btn-icon">"🔴"</span>
                                        <span class="post-battle__btn-text">"Cattura"</span>
                                    </button>
                                    <button
                                        class="post-battle__btn post-battle__btn--skip"
                                        disabled=decided
                                        on:click=on_skip
                                    >
                                        <span class="post-battle__btn-icon">"✕"</span>
                                        <span class="post-battle__btn-text">"Lascia perdere"</span>
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    }
                })
            }}

            <div class="post-battle__actions">
                {move || {
                    let run = ctx.run.read();
                    let can_pokecenter = run.as_ref()
                        .map(|r| r.gym.can_use_pokecenter())
                        .unwrap_or(false);
                    let uses_left = run.as_ref()
                        .map(|r| POKECENTER_MAX_PER_GYM - r.gym.pokecenter_uses)
                        .unwrap_or(0);

                    view! {
                        <>
                            <button
                                class="post-battle__btn post-battle__btn--pokecenter"
                                on:click=on_pokecenter
                                disabled=!can_pokecenter
                            >
                                <span class="post-battle__btn-icon">"✚"</span>
                                <div class="post-battle__btn-text">
                                    "Pokécenter"
                                    <div class="post-battle__btn-sub">
                                        {if can_pokecenter {
                                            format!("{uses_left} usi rimasti")
                                        } else {
                                            "Nessun uso disponibile".to_string()
                                        }}
                                    </div>
                                </div>
                            </button>
                            <button class="post-battle__btn post-battle__btn--shop" on:click=on_shop>
                                <span class="post-battle__btn-icon">"🛒"</span>
                                <span class="post-battle__btn-text">"Negozio"</span>
                            </button>
                            <button class="post-battle__btn post-battle__btn--next" on:click=on_next>
                                <span class="post-battle__btn-icon">"▶"</span>
                                <span class="post-battle__btn-text">"Prossima battaglia"</span>
                            </button>
                        </>
                    }
                }}
            </div>
        </div>
    }
}

pub fn use_catch_requested() -> RwSignal<bool> {
    use_context::<RwSignal<bool>>().expect("catch_requested signal non trovato")
}
