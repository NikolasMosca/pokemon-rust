use leptos::prelude::*;
use game_core::run::{RunPhase, RunState};
use game_core::pokemon::Pokemon;
use crate::components::game::{
    battle_screen::BattleScreen,
    game_over::GameOver,
    pokecenter_screen::PokecenterScreen,
    post_battle::{PostBattle, PostBattleResult},
    progress_popup::ProgressPopup,
    run_complete::RunComplete,
    shop_screen::ShopScreen,
    starter_select::StarterSelect,
};
use crate::components::attack_anim::AttackAnim;
use crate::components::damage_number::DamageKind;

#[derive(Clone)]
pub struct BattleSignals {
    pub enemy: RwSignal<Option<Pokemon>>,
    /// Team avversario rimanente (per trainer/gym leader multi-Pokémon).
    /// Il primo elemento è il prossimo Pokémon da affrontare.
    pub enemy_team: RwSignal<Vec<Pokemon>>,
    pub enemy_sprite: RwSignal<Option<String>>,
    pub player_sprite: RwSignal<Option<String>>,
    /// Sprite URL per ogni slot del team del giocatore (indice = slot team).
    pub team_sprites: RwSignal<Vec<Option<String>>>,
    pub active_idx: RwSignal<usize>,
    pub dialog: RwSignal<String>,
    pub damage_event: RwSignal<Option<(i32, DamageKind)>>,
    /// Numero danno/cura sopra la card del foe (separato da damage_event che è per il player).
    pub foe_damage_event: RwSignal<Option<(i32, DamageKind)>>,
    pub input_locked: RwSignal<bool>,
    /// Quando true lo sprite del player mostra l'animazione di sconfitta.
    pub player_fainted: RwSignal<bool>,
    /// Animazione attacco corrente del player (None = idle).
    pub player_attack_anim: RwSignal<Option<AttackAnim>>,
    /// Animazione attacco corrente del nemico (None = idle).
    pub enemy_attack_anim: RwSignal<Option<AttackAnim>>,
    /// Quando true mostra il flash bianco di impatto.
    pub hit_flash: RwSignal<bool>,
}

impl BattleSignals {
    fn new() -> Self {
        Self {
            enemy: RwSignal::new(None),
            enemy_team: RwSignal::new(Vec::new()),
            enemy_sprite: RwSignal::new(None),
            player_sprite: RwSignal::new(None),
            team_sprites: RwSignal::new(Vec::new()),
            active_idx: RwSignal::new(0),
            dialog: RwSignal::new(String::new()),
            damage_event: RwSignal::new(None),
            foe_damage_event: RwSignal::new(None),
            input_locked: RwSignal::new(true),
            player_fainted: RwSignal::new(false),
            player_attack_anim: RwSignal::new(None),
            enemy_attack_anim: RwSignal::new(None),
            hit_flash: RwSignal::new(false),
        }
    }
}

#[derive(Clone)]
pub struct GameContext {
    pub run: RwSignal<Option<RunState>>,
    pub show_progress: RwSignal<bool>,
    pub battle: BattleSignals,
}

impl GameContext {
    fn new() -> Self {
        Self {
            run: RwSignal::new(None),
            show_progress: RwSignal::new(false),
            battle: BattleSignals::new(),
        }
    }
}

pub fn use_game_context() -> GameContext {
    use_context::<GameContext>().expect("GameContext non trovato")
}

#[component]
pub fn GamePage() -> impl IntoView {
    let ctx = GameContext::new();
    provide_context(ctx.clone());

    // Signal aggiuntivi nel context
    provide_context(RwSignal::new(None::<PostBattleResult>));
    provide_context(RwSignal::new(false));  // catch_requested (usato da use_catch_requested in post_battle)
    provide_context(RwSignal::new(None::<game_core::pokemon::Pokemon>)); // catch_candidate

    let run = ctx.run;
    let show_progress = ctx.show_progress;

    // Memo che si aggiorna SOLO quando la fase cambia — non ad ogni scrittura al team.
    // Senza questo, ogni update_untracked agli HP del team durante il turno causa
    // il rimontaggio di BattleScreen perché run.read() riesegue l'intero match.
    let phase = Memo::new(move |_| run.with(|r| r.as_ref().map(|r| r.phase.clone())));

    view! {
        <div class="game-page">
            {move || run.read().is_some().then(|| view! {
                <button
                    class="game-page__progress-btn"
                    title="Progressi"
                    on:click=move |_| show_progress.update(|v| *v = !*v)
                >
                    "ℹ"
                </button>
            })}

            {move || (show_progress.get() && run.read().is_some()).then(|| {
                let gym = run.read().as_ref().map(|r| r.gym.clone()).unwrap();
                view! {
                    <ProgressPopup
                        gym=gym
                        on_close=move || show_progress.set(false)
                    />
                }
            })}

            {move || {
                match phase.get() {
                    None => view! { <StarterSelect /> }.into_any(),
                    Some(RunPhase::InBattle { kind }) => view! {
                        <BattleScreen kind=kind />
                    }.into_any(),
                    Some(RunPhase::PostBattle) => view! {
                        <PostBattle />
                    }.into_any(),
                    Some(RunPhase::Pokecenter) => view! {
                        <PokecenterScreen />
                    }.into_any(),
                    Some(RunPhase::Shop) => view! {
                        <ShopScreen />
                    }.into_any(),
                    Some(RunPhase::GameOver) => view! {
                        <GameOver />
                    }.into_any(),
                    Some(RunPhase::RunComplete) => view! {
                        <RunComplete />
                    }.into_any(),
                }
            }}
        </div>
    }
}
