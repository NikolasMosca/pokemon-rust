use leptos::prelude::*;
use leptos_router::components::A;
use crate::audio::{use_audio, music::MusicTrack};

#[component]
pub fn HomePage() -> impl IntoView {
    let audio = use_audio();
    let show_rules = RwSignal::new(false);

    let on_click = move |_| {
        audio.init();
        audio.play_music(MusicTrack::Pokecenter);
    };

    view! {
        <div class="home-page">
            <img class="home-page__logo" src="/assets/images/logo.png" alt="Pokémon" />
            <img class="home-page__rust" src="/assets/images/rust.png" alt="Rust" />
            <A href="/game">
                <button class="home-page__start" on:click=on_click>"START"</button>
            </A>
            <button class="home-page__rules-btn" on:click=move |_| show_rules.set(true)>
                "Regole del gioco"
            </button>

            {move || show_rules.get().then(|| view! {
                <div class="home-rules-overlay" on:click=move |_| show_rules.set(false)>
                    <div class="home-rules-modal" on:click=|e| e.stop_propagation()>
                        <h2 class="home-rules-modal__title">"Come si gioca"</h2>
                        <div class="home-rules-modal__body">
                            <div class="home-rules-modal__section">
                                <h3>"Obiettivo"</h3>
                                <p>"Sconfiggi i Capopalestra delle 8 palestre per completare il gioco. Le battaglie si affrontano in sequenza: prima gli allenatori della palestra, poi il Capopalestra."</p>
                            </div>
                            <div class="home-rules-modal__section">
                                <h3>"Scelta iniziale"</h3>
                                <p>"All'inizio scegli il tuo Pokémon di partenza. Da quel momento costruisci la tua squadra catturando i Pokémon selvatici che incontri."</p>
                            </div>
                            <div class="home-rules-modal__section">
                                <h3>"Battaglie"</h3>
                                <p>"Ogni turno scegli una mossa. Puoi anche usare un oggetto dalla borsa o cambiare Pokémon attivo. Vince chi porta a zero gli HP dell'avversario."</p>
                            </div>
                            <div class="home-rules-modal__section">
                                <h3>"Cattura"</h3>
                                <p>"Dopo aver sconfitto un Pokémon selvatico puoi scegliere di catturarlo. Se hai già 6 Pokémon in squadra, dovrai scegliere quale sostituire."</p>
                            </div>
                            <div class="home-rules-modal__section">
                                <h3>"Pokécenter"</h3>
                                <p>"Dopo ogni battaglia hai a disposizione il Pokécenter per curare tutta la squadra. Hai 3 utilizzi per ogni palestra — si ripristinano quando avanzi alla palestra successiva."</p>
                            </div>
                            <div class="home-rules-modal__section">
                                <h3>"Negozio"</h3>
                                <p>"Dopo ogni battaglia puoi accedere al negozio per acquistare oggetti con i soldi guadagnati. Gli oggetti si usano durante la battaglia dalla borsa."</p>
                            </div>
                            <div class="home-rules-modal__section">
                                <h3>"Game Over"</h3>
                                <p>"Se tutti i tuoi Pokémon vengono sconfitti è game over. Si ricomincia dall'inizio."</p>
                            </div>
                            <div class="home-rules-modal__section home-rules-modal__section--note">
                                <h3>"Note su questa versione"</h3>
                                <p>"Le mosse di tipo Status e le mosse multiturno non sono implementate. Ai Pokémon non verranno assegnate mosse di questo tipo."</p>
                            </div>
                        </div>
                        <button class="home-rules-modal__close" on:click=move |_| show_rules.set(false)>
                            "Chiudi"
                        </button>
                    </div>
                </div>
            })}
        </div>
    }
}
