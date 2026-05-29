use leptos::prelude::*;
use crate::components::attack_anim::AttackAnim;

#[component]
pub fn BattleLayout(
    enemy_card: impl IntoView + 'static,
    #[prop(into)] enemy_sprite: Signal<Option<String>>,
    player_card: impl IntoView + 'static,
    #[prop(into)] player_sprite: Signal<Option<String>>,
    #[prop(into)] player_fainted: Signal<bool>,
    #[prop(into)] player_attack_anim: Signal<Option<AttackAnim>>,
    #[prop(into)] enemy_attack_anim: Signal<Option<AttackAnim>>,
    #[prop(into)] hit_flash: Signal<bool>,
    damage: impl IntoView + 'static,
    foe_damage: impl IntoView + 'static,
    dialog: impl IntoView + 'static,
    actions: impl IntoView + 'static,
    #[prop(into)] trainer_sprite: Signal<Option<String>>,
    #[prop(into)] trainer_label: Signal<Option<String>>,
) -> impl IntoView {
    view! {
        // Flash di impatto (overlay fisso bianco a schermo intero)
        {move || hit_flash.get().then(|| view! {
            <div class="battle-hit-flash" />
        })}

        <div class="battle-layout">
            <div class="battle-layout__scene">

                // ── Sprite nemico ──────────────────────────────────────────
                <div class="battle-layout__platform-enemy">
                    {move || enemy_sprite.get().map(|url| {
                        let anim = enemy_attack_anim.get();
                        let orb_color = if let Some(AttackAnim::Special { color }) = &anim {
                            Some(*color)
                        } else {
                            None
                        };
                        let cls = sprite_class("enemy", &anim);
                        view! {
                            <div class="battle-layout__sprite-wrap">
                                <img class=cls src=url alt="enemy" />
                                {orb_color.map(|c| view! {
                                    <div
                                        class="battle-orb battle-orb--enemy-to-player"
                                        style=format!("--orb-color:{c};--orb-glow:{c}88;")
                                    />
                                })}
                            </div>
                        }
                    })}
                </div>

                // ── Sprite player ──────────────────────────────────────────
                <div class="battle-layout__platform-player">
                    {move || player_sprite.get().map(|url| {
                        let fainted = player_fainted.get();
                        let anim = player_attack_anim.get();
                        let orb_color = if let Some(AttackAnim::Special { color }) = &anim {
                            Some(*color)
                        } else {
                            None
                        };
                        let cls = if fainted {
                            "battle-layout__sprite battle-layout__sprite--player battle-layout__sprite--fainted".to_string()
                        } else {
                            sprite_class("player", &anim)
                        };
                        view! {
                            <div class="battle-layout__sprite-wrap">
                                <img class=cls src=url alt="player" />
                                {orb_color.map(|c| view! {
                                    <div
                                        class="battle-orb battle-orb--player-to-enemy"
                                        style=format!("--orb-color:{c};--orb-glow:{c}88;")
                                    />
                                })}
                            </div>
                        }
                    })}
                </div>

                // Trainer sprite in alto a destra (None per selvatici)
                {move || trainer_sprite.get().map(|url| view! {
                    <div class="battle-layout__trainer-area">
                        <img class="battle-layout__trainer-sprite" src=url alt="trainer" />
                        {move || trainer_label.get().map(|label| view! {
                            <span class="battle-layout__trainer-label">{label}</span>
                        })}
                    </div>
                })}

                <div class="battle-layout__card-enemy">
                    {enemy_card}
                    <div class="battle-layout__damage battle-layout__damage--foe">{foe_damage}</div>
                </div>
                <div class="battle-layout__card-player">
                    {player_card}
                    <div class="battle-layout__damage battle-layout__damage--player">{damage}</div>
                </div>
            </div>
            <div class="battle-layout__bottom">
                <div class="battle-layout__dialog">{dialog}</div>
                <div class="battle-layout__actions">{actions}</div>
            </div>
        </div>
    }
}

fn sprite_class(who: &str, anim: &Option<AttackAnim>) -> String {
    let base = format!("battle-layout__sprite battle-layout__sprite--{who}");
    match anim {
        None => base,
        Some(AttackAnim::Physical)      => format!("{base} sprite-lunge-{who}"),
        Some(AttackAnim::Special { .. }) => format!("{base} sprite-charge"),
        Some(AttackAnim::Heal)           => format!("{base} sprite-heal-glow"),
        Some(AttackAnim::Hit)            => format!("{base} sprite-hit"),
    }
}
