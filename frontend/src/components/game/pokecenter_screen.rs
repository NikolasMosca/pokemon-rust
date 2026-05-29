use leptos::prelude::*;
use game_core::run::RunPhase;
use crate::pages::game::use_game_context;

#[component]
pub fn PokecenterScreen() -> impl IntoView {
    let ctx = use_game_context();

    let on_heal = move |_| {
        ctx.run.update(|r| {
            if let Some(run) = r {
                let _ = run.use_pokecenter();
            }
        });
    };

    let on_close = move |_| {
        ctx.run.update(|r| {
            if let Some(run) = r {
                run.phase = RunPhase::PostBattle;
            }
        });
    };

    view! {
        <div class="pokecenter-screen">
            <h2 class="pokecenter-screen__title">"Pokécenter"</h2>
            <p class="pokecenter-screen__desc">
                "Vuoi ripristinare tutti i tuoi Pokémon?"
            </p>

            {move || {
                let run = ctx.run.read();
                let Some(r) = run.as_ref() else { return view! {<></>}.into_any() };
                let uses_left = game_core::run::gym::POKECENTER_MAX_PER_GYM - r.gym.pokecenter_uses;
                view! {
                    <p class="pokecenter-screen__uses">
                        "Utilizzi rimasti: " {uses_left}
                    </p>
                }.into_any()
            }}

            <div class="pokecenter-screen__team">
                {move || {
                    ctx.run.read().as_ref().map(|r| {
                        r.team.iter().map(|p| view! {
                            <div class="pokecenter-screen__pokemon">
                                <span class="pokecenter-screen__pokemon-name">{p.name.clone()}</span>
                                <span class="pokecenter-screen__pokemon-hp">
                                    "HP " {p.current_hp} "/" {p.max_hp()}
                                </span>
                            </div>
                        }).collect_view()
                    })
                }}
            </div>

            <div class="pokecenter-screen__actions">
                <button class="pokecenter-screen__btn pokecenter-screen__btn--heal"
                    on:click=on_heal>
                    "Cura i Pokémon"
                </button>
                <button class="pokecenter-screen__btn pokecenter-screen__btn--close"
                    on:click=on_close>
                    "Esci"
                </button>
            </div>
        </div>
    }
}
