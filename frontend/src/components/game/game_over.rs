use leptos::prelude::*;
use crate::pages::game::use_game_context;

#[component]
pub fn GameOver() -> impl IntoView {
    let ctx = use_game_context();

    let on_restart = move |_| {
        ctx.run.set(None);
    };

    view! {
        <div class="game-over">
            <h1 class="game-over__title">"Game Over"</h1>
            <p class="game-over__message">
                "Tutti i tuoi Pokémon sono stati sconfitti."
            </p>
            <button class="game-over__btn" on:click=on_restart>
                "Ricomincia"
            </button>
        </div>
    }
}
