use leptos::prelude::*;
use crate::components::{
    battle_button::{BattleAction, BattleButton},
    battle_layout::BattleLayout,
    damage_number::{DamageKind, DamageNumber},
    pokemon_card::PokemonCard,
};
use crate::core::cache::use_pokemon_cache;

const MAX_HP: u32 = 100;
const DAMAGE: u32 = 15;
const HEAL: u32 = 20;

#[component]
pub fn UiKitPage() -> impl IntoView {
    let cache = use_pokemon_cache();

    let (enemy_hp, set_enemy_hp) = signal(MAX_HP);
    let (player_hp, set_player_hp) = signal(MAX_HP);
    let (last_event, set_last_event) = signal::<Option<(i32, DamageKind)>>(None);
    let (dialog_text, set_dialog_text) = signal("What will\nPikachu do?");
    let (enemy_sprite, set_enemy_sprite) = signal::<Option<String>>(None);
    let (player_sprite, set_player_sprite) = signal::<Option<String>>(None);

    let cache_enemy = cache.clone();
    let cache_player = cache.clone();

    leptos::task::spawn_local(async move {
        if let Ok(data) = cache_enemy.fetch("charizard").await {
            set_enemy_sprite.set(data.sprites.front_default);
        }
    });

    leptos::task::spawn_local(async move {
        if let Ok(data) = cache_player.fetch("pikachu").await {
            set_player_sprite.set(data.sprites.back_default);
        }
    });

    let on_action = move |action: BattleAction| match action {
        BattleAction::Fight => {
            set_enemy_hp.update(|hp| *hp = hp.saturating_sub(DAMAGE));
            set_last_event.set(Some((DAMAGE as i32, DamageKind::Damage)));
            set_dialog_text.set("Pikachu used\nThunderbolt!");
        }
        BattleAction::Bag => {
            set_player_hp.update(|hp| *hp = (*hp + HEAL).min(MAX_HP));
            set_last_event.set(Some((HEAL as i32, DamageKind::Heal)));
            set_dialog_text.set("Pikachu used\na Potion!");
        }
        BattleAction::Pokemon => {
            set_dialog_text.set("Choose a\nPokémon!");
        }
        BattleAction::Run => {
            set_enemy_hp.set(MAX_HP);
            set_player_hp.set(MAX_HP);
            set_last_event.set(None);
            set_dialog_text.set("What will\nPikachu do?");
        }
    };

    view! {
        <BattleLayout
            enemy_card=view! {
                <PokemonCard
                    name=Signal::derive(|| "Charizard".to_string())
                    level=Signal::derive(|| 50u32)
                    hp=Signal::derive(move || enemy_hp.get())
                    max_hp=Signal::derive(|| MAX_HP)
                />
            }
            enemy_sprite=Signal::derive(move || enemy_sprite.get())
            player_card=view! {
                <PokemonCard
                    name=Signal::derive(|| "Pikachu".to_string())
                    level=Signal::derive(|| 42u32)
                    hp=Signal::derive(move || player_hp.get())
                    max_hp=Signal::derive(|| MAX_HP)
                    is_player=true
                />
            }
            player_sprite=Signal::derive(move || player_sprite.get())
            player_fainted=Signal::derive(|| false)
            player_attack_anim=Signal::derive(|| None)
            enemy_attack_anim=Signal::derive(|| None)
            hit_flash=Signal::derive(|| false)
            trainer_sprite=Signal::derive(|| None::<String>)
            trainer_label=Signal::derive(|| None::<String>)
            damage=view! { <DamageNumber value=last_event /> }
            foe_damage=view! { <></> }
            dialog=view! {
                <span class="text-preline">{dialog_text}</span>
            }
            actions=view! {
                <BattleButton action=BattleAction::Fight on_click=on_action />
                <BattleButton action=BattleAction::Bag on_click=on_action />
                <BattleButton action=BattleAction::Pokemon on_click=on_action />
            }
        />
    }
}
