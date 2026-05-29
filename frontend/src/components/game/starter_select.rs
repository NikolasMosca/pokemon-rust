use leptos::prelude::*;
use game_core::run::RunState;
use crate::audio::{use_audio, music::MusicTrack};
use crate::components::pokemon_select_card::PokemonSelectCard;
use crate::core::cache::use_pokemon_cache;
use crate::core::generator::{assign_moves, build_pokemon, pick_starters};
use crate::pages::game::use_game_context;
use game_core::battle::rng::Rng;

#[component]
pub fn StarterSelect() -> impl IntoView {
    let ctx = use_game_context();
    let cache = use_pokemon_cache();
    let audio = use_audio();

    Effect::new(move |_| {
        audio.init();
        audio.play_music(MusicTrack::Pokecenter);
    });

    let starters: [&'static str; 3] = {
        let mut rng = Rng::new(js_sys::Date::now() as u64);
        pick_starters(&mut rng)
    };

    // (sprite, max_hp)
    let starter_data = RwSignal::new(vec![None::<(Option<String>, u32)>; 3]);
    let loading = RwSignal::new(true);

    {
        let cache = cache.clone();
        let names = starters;
        leptos::task::spawn_local(async move {
            let mut results = vec![None; 3];
            for (i, name) in names.iter().enumerate() {
                if let Ok(data) = cache.fetch(name).await {
                    let sprite = data.sprites.front_default.clone();
                    // HP base a livello 5
                    let base_hp = data.stats.iter()
                        .find(|s| s.stat.name == "hp")
                        .map(|s| s.base_stat)
                        .unwrap_or(45);
                    let max_hp = (base_hp * 2 * 5 / 100 + 5 + 10) as u32;
                    results[i] = Some((sprite, max_hp));
                }
            }
            starter_data.set(results);
            loading.set(false);
        });
    }

    let on_select = StoredValue::new(move |idx: usize| {
        let cache = cache.clone();
        let name = starters[idx];
        leptos::task::spawn_local(async move {
            let Ok(data) = cache.fetch(name).await else { return };
            let mut pokemon = build_pokemon(&data, 5, &[]);
            let mut rng = Rng::new(js_sys::Date::now() as u64 ^ (idx as u64 * 0xABCD));
            assign_moves(&mut pokemon, 5, &mut rng);
            let run = RunState::new(pokemon);
            ctx.run.set(Some(run));
        });
    });

    view! {
        <div class="starter-select">
            <h1 class="starter-select__title">"Scegli il tuo Pokémon di partenza"</h1>
            {move || loading.get().then(|| view! {
                <p class="starter-select__loading">"Caricamento..."</p>
            })}
            <div class="psc-grid">
                {move || {
                    let data = starter_data.get();
                    starters.iter().enumerate().map(|(i, &name)| {
                        let entry = data.get(i).and_then(|d| d.clone());
                        let sprite: Option<String> = entry.as_ref().and_then(|(s, _)| s.clone());
                        let max_hp = entry.as_ref().map(|(_, h)| *h).unwrap_or(1);
                        let pname = name.to_string();
                        view! {
                            <PokemonSelectCard
                                name=pname
                                level=5u32
                                hp=max_hp
                                max_hp=max_hp
                                img_url={sprite}
                                disabled=entry.is_none()
                                on_click=move || on_select.get_value()(i)
                            />
                        }
                    }).collect_view()
                }}
            </div>
        </div>
    }
}
