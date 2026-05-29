use leptos::prelude::*;
use game_core::types::Type;
use crate::components::attack_anim::type_css_class;
use crate::components::hp_bar::HPBar;

#[component]
pub fn PokemonSelectCard(
    #[prop(into)] name: String,
    level: u32,
    hp: u32,
    max_hp: u32,
    #[prop(default = false)] is_active: bool,
    #[prop(default = false)] is_fainted: bool,
    #[prop(default = false)] disabled: bool,
    #[prop(default = false)] allow_fainted: bool,
    img_url: Option<String>,
    #[prop(default = None)] pokemon_type: Option<Type>,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    let class = move || {
        let mut c = "psc".to_string();
        if let Some(ref t) = pokemon_type {
            c.push(' ');
            c.push_str(type_css_class(t));
        }
        if is_active  { c.push_str(" psc--active"); }
        if is_fainted { c.push_str(" psc--fainted"); }
        c
    };

    view! {
        <button class=class disabled=disabled || (is_fainted && !allow_fainted) on:click=move |_| on_click()>
            <div class="psc__ball">
                {img_url.map(|url| view! {
                    <img class="psc__sprite" src=url alt=name.clone() />
                })}
            </div>
            <div class="psc__info">
                <div class="psc__header">
                    <span class="psc__name">{name}</span>
                    <span class="psc__level">"Lv."{ level}</span>
                </div>
                <HPBar
                    hp=Signal::derive(move || hp)
                    max_hp=Signal::derive(move || max_hp)
                />
                <div class="psc__hp-value">
                    {hp}" / "{max_hp}
                </div>
            </div>
        </button>
    }
}
