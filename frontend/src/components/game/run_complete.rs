use leptos::prelude::*;
use crate::pages::game::use_game_context;

#[component]
pub fn RunComplete() -> impl IntoView {
    let ctx = use_game_context();

    let badges = move || ctx.run.read().as_ref().map(|r| r.gym.badges()).unwrap_or(8);

    let on_restart = move |_| {
        ctx.run.set(None);
    };

    view! {
        <div class="run-complete">
            <h1 class="run-complete__title">"Congratulazioni!"</h1>
            <div class="run-complete__badges">
                {move || (0..badges()).map(|_| view! {
                    <span class="run-complete__badge">"🏅"</span>
                }).collect_view()}
            </div>
            <p class="run-complete__message">
                "Hai ottenuto tutti gli 8 badge di Kanto!"
            </p>
            <p class="run-complete__message run-complete__message--sub">
                "Sei pronto per affrontare la Lega Pokémon!"
            </p>
            <button class="run-complete__btn" on:click=on_restart>
                "Nuova partita"
            </button>
        </div>
    }
}
