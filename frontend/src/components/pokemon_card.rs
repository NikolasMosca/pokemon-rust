use leptos::prelude::*;
use game_core::types::Type;
use crate::components::attack_anim::type_css_class;
use crate::components::hp_bar::HPBar;

#[component]
pub fn PokemonCard(
    #[prop(into)] name: Signal<String>,
    #[prop(into)] level: Signal<u32>,
    #[prop(into)] hp: Signal<u32>,
    #[prop(into)] max_hp: Signal<u32>,
    #[prop(default = false)] is_player: bool,
    #[prop(into, default = Signal::derive(|| None))] pokemon_type: Signal<Option<Type>>,
) -> impl IntoView {
    view! {
        <div class=move || {
            let side = if is_player { "pokemon-card--player" } else { "pokemon-card--enemy" };
            let type_cls = pokemon_type.get().as_ref().map(type_css_class).unwrap_or("");
            format!("pokemon-card {side} {type_cls}")
        }>
            <div class="pokemon-card__header">
                <span class="pokemon-card__name">{move || name.try_get().unwrap_or_default()}</span>
                <span class="pokemon-card__level">"Lv." {move || level.try_get().unwrap_or(0)}</span>
            </div>
            <div class="pokemon-card__hp-row">
                <span class="pokemon-card__hp-label">"HP"</span>
                <HPBar hp=hp max_hp=max_hp />
            </div>
            {move || is_player.then(|| view! {
                <div class="pokemon-card__hp-value">
                    {move || hp.try_get().unwrap_or(0)} "/" {move || max_hp.try_get().unwrap_or(0)}
                </div>
            })}
        </div>
    }
}
