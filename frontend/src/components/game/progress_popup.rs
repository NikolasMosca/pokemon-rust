use leptos::prelude::*;
use game_core::run::gym::{GymProgress, TOTAL_GYMS, TRAINERS_PER_GYM};

#[component]
pub fn ProgressPopup(
    gym: GymProgress,
    on_close: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let on_close_btn = on_close.clone();
    let on_close_backdrop = on_close.clone();

    view! {
        <div class="progress-popup__backdrop" on:click=move |_| on_close_backdrop()>
            <div class="progress-popup" on:click=|e| e.stop_propagation()>
                <div class="progress-popup__header">
                    <h2 class="progress-popup__title">"Progressi"</h2>
                    <button class="progress-popup__close" on:click=move |_| on_close_btn()>"✕"</button>
                </div>

                <div class="progress-popup__path">
                    {(0..TOTAL_GYMS).map(|i| {
                        let is_complete = i < gym.gym_index;
                        let is_current = i == gym.gym_index;

                        let gym_class = if is_complete {
                            "progress-popup__gym progress-popup__gym--complete"
                        } else if is_current {
                            "progress-popup__gym progress-popup__gym--current"
                        } else {
                            "progress-popup__gym progress-popup__gym--locked"
                        };

                        view! {
                            <div class=gym_class>
                                <div class="progress-popup__gym-header">
                                    <span class="progress-popup__gym-dot">
                                        {if is_complete { "●" } else if is_current { "◉" } else { "○" }}
                                    </span>
                                    <span class="progress-popup__gym-name">
                                        "Palestra " {i + 1}
                                    </span>
                                    <span class="progress-popup__gym-status">
                                        {if is_complete { " — COMPLETATA" }
                                         else if is_current { " — IN CORSO" }
                                         else { " — BLOCCATA" }}
                                    </span>
                                </div>

                                {is_current.then(|| {
                                    let trainers_done = gym.trainers_defeated;
                                    view! {
                                        <div class="progress-popup__gym-detail">
                                            {(0..TRAINERS_PER_GYM).map(|t| {
                                                let done = t < trainers_done;
                                                view! {
                                                    <div class="progress-popup__trainer">
                                                        "├─ Allenatore " {t + 1}
                                                        {if done { " ✓" } else { " ✗" }}
                                                    </div>
                                                }
                                            }).collect_view()}
                                            <div class="progress-popup__leader">
                                                "└─ Capopalestra "
                                                {if trainers_done >= TRAINERS_PER_GYM { "✓" } else { "✗" }}
                                            </div>
                                        </div>
                                    }
                                })}
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}
