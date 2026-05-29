use leptos::prelude::*;
use game_core::run::MAX_TEAM_SIZE;
use crate::pages::game::use_game_context;
use crate::components::game::post_battle::use_catch_requested;

/// Pokémon da catturare, settato da BattleScreen prima della transizione a PostBattle.
pub fn use_catch_candidate() -> RwSignal<Option<game_core::pokemon::Pokemon>> {
    use_context::<RwSignal<Option<game_core::pokemon::Pokemon>>>()
        .expect("catch_candidate signal non trovato")
}

#[component]
pub fn CatchScreen() -> impl IntoView {
    let ctx = use_game_context();
    let catch_requested = use_catch_requested();
    let candidate = use_catch_candidate();


    let on_confirm = move |_| {
        let Some(pokemon) = candidate.get() else { return };
        ctx.run.update(|r| {
            let Some(run) = r else { return };
            if run.team.len() < MAX_TEAM_SIZE {
                let _ = run.catch(pokemon.clone());
                catch_requested.set(false);
                candidate.set(None);
            }
            // Se pieno, aspetta selezione slot sostituzione
        });
    };

    let on_replace = move |slot: usize| {
        let Some(pokemon) = candidate.get() else { return };
        ctx.run.update(|r| {
            let Some(run) = r else { return };
            let _ = run.replace_team_slot(slot, pokemon.clone());
        });
        catch_requested.set(false);
        candidate.set(None);
    };

    let on_cancel = move |_| {
        catch_requested.set(false);
        candidate.set(None);
    };

    view! {
        <div class="catch-screen">
            <div class="catch-screen__overlay">
                {move || {
                    let Some(p) = candidate.get() else {
                        return view! { <></> }.into_any();
                    };
                    let team_full = ctx.run.read().as_ref()
                        .map(|r| r.team.len() >= MAX_TEAM_SIZE)
                        .unwrap_or(false);

                    view! {
                        <div class="catch-screen__content">
                            <h2 class="catch-screen__title">
                                "Vuoi catturare " {p.name.clone()} "?"
                            </h2>

                            {(!team_full).then(|| view! {
                                <div class="catch-screen__actions">
                                    <button class="catch-screen__btn catch-screen__btn--confirm"
                                        on:click=on_confirm>
                                        "Cattura"
                                    </button>
                                    <button class="catch-screen__btn catch-screen__btn--cancel"
                                        on:click=on_cancel>
                                        "Lascia perdere"
                                    </button>
                                </div>
                            })}

                            {team_full.then(move || {
                                let team = ctx.run.read().as_ref()
                                    .map(|r| r.team.clone())
                                    .unwrap_or_default();
                                view! {
                                    <div class="catch-screen__replace">
                                        <p class="catch-screen__replace-title">
                                            "Il team è pieno. Scegli chi sostituire:"
                                        </p>
                                        {team.into_iter().enumerate().map(|(i, member)| {
                                            view! {
                                                <button
                                                    class="catch-screen__replace-slot"
                                                    on:click=move |_| on_replace(i)
                                                >
                                                    {member.name.clone()}
                                                    " Lv." {member.level}
                                                    " HP " {member.current_hp} "/" {member.max_hp()}
                                                </button>
                                            }
                                        }).collect_view()}
                                        <button class="catch-screen__btn catch-screen__btn--cancel"
                                            on:click=on_cancel>
                                            "Annulla"
                                        </button>
                                    </div>
                                }
                            })}
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
