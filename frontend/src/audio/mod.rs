pub mod context;
pub mod cries;
pub mod music;
pub mod sfx;
pub mod volume;

use std::cell::RefCell;
use std::rc::Rc;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use context::AudioCtx;
use cries::CriesPlayer;
use music::{MusicPlayer, MusicTrack, fetch_audio_buffer, cache_audio_url};
use sfx::{Sfx, SfxPlayer};
use volume::VolumeGroups;

const FADE_MS: u32 = 600;

/// Stato interno dell'AudioManager — condiviso via Rc tra i clone.
struct Inner {
    ctx:    AudioCtx,
    music:  MusicPlayer,
    sfx:    SfxPlayer,
    cries:  CriesPlayer,
    volume: VolumeGroups,
    current_track: Option<MusicTrack>,
}

/// Clone condivide lo stesso Inner — tutti i clone puntano allo stesso stato.
#[derive(Clone)]
pub struct AudioManager(SendWrapper<Rc<RefCell<Option<Inner>>>>);

impl AudioManager {
    pub fn new() -> Self {
        Self(SendWrapper::new(Rc::new(RefCell::new(None))))
    }

    /// Va chiamato dentro un event handler (click/touch) per
    /// soddisfare il requisito di interazione utente dei browser mobile.
    pub fn init(&self) {
        // Controlla se già inizializzato — borrow immutabile, rilasciato subito
        {
            let guard = self.0.borrow();
            if guard.is_some() {
                if let Some(inner) = guard.as_ref() {
                    inner.ctx.ensure_running();
                }
                return;
            }
        } // guard rilasciato qui

        let ctx = match AudioCtx::new() {
            Ok(c) => c,
            Err(e) => {
                web_sys::console::warn_1(&format!("AudioContext init failed: {:?}", e).into());
                return;
            }
        };

        let vol = VolumeGroups::load();

        let music = match MusicPlayer::new(&ctx.inner, vol.effective_music()) {
            Ok(m) => m,
            Err(_) => return,
        };
        let sfx = match SfxPlayer::new(&ctx.inner, vol.effective_sfx()) {
            Ok(s) => s,
            Err(_) => return,
        };
        let cries = match CriesPlayer::new(&ctx.inner, vol.effective_cries()) {
            Ok(c) => c,
            Err(_) => return,
        };

        // Scrive lo stato e rilascia immediatamente il guard prima di spawn_local
        {
            let mut guard = self.0.borrow_mut();
            *guard = Some(Inner { ctx, music, sfx, cries, volume: vol, current_track: None });
        } // guard rilasciato qui — preload_sfx può ora borrow liberamente

        // Precarica SFX comuni in background
        let manager = self.clone();
        leptos::task::spawn_local(async move {
            manager.preload_sfx().await;
        });

        // Gestione visibilitychange (pausa/riprendi su tab switch)
        let manager = self.clone();
        setup_visibility_handler(manager);
    }

    // ── Musica ────────────────────────────────────────────────────────────

    pub fn play_music(&self, track: MusicTrack) {
        let manager = self.clone();
        leptos::task::spawn_local(async move {
            let (ctx_ref, already_playing) = {
                let guard = manager.0.borrow();
                let Some(inner) = guard.as_ref() else { return };
                let already = inner.current_track == Some(track);
                // Estraiamo quello che ci serve prima di rilasciare il borrow
                (inner.ctx.inner.clone(), already)
            };

            if already_playing { return; }

            // Fetch buffer (può prendere tempo — fuori dal borrow)
            let url = track.path();
            let buffer = match fetch_audio_buffer(&ctx_ref, url).await {
                Ok(b) => b,
                Err(e) => {
                    web_sys::console::warn_1(&format!("music fetch failed: {e}").into());
                    return;
                }
            };

            // Cache in background
            let url_owned = url.to_string();
            leptos::task::spawn_local(async move {
                cache_audio_url(&url_owned).await;
            });

            // Aggiorna current_track e leggi il volume — poi rilascia il borrow
            let (gain_node, vol) = {
                let mut guard = manager.0.borrow_mut();
                let Some(inner) = guard.as_mut() else { return };
                inner.current_track = Some(track);
                let vol = inner.volume.effective_music();
                (inner.music.gain_node().clone(), vol)
            }; // borrow rilasciato — crossfade (600ms di await) non tiene il lock

            // Crossfade senza borrow attivo
            {
                let mut guard = manager.0.borrow_mut();
                let Some(inner) = guard.as_mut() else { return };
                inner.music.crossfade_start(&ctx_ref, buffer, track, vol);
            }
            music::MusicPlayer::crossfade_fade_in(&gain_node, vol, FADE_MS).await;
        });
    }

    pub fn stop_music(&self) {
        let mut guard = self.0.borrow_mut();
        let Some(inner) = guard.as_mut() else { return };
        inner.music.stop();
        inner.current_track = None;
    }

    // ── SFX ───────────────────────────────────────────────────────────────

    pub fn play_sfx(&self, sfx: Sfx) {
        let mut guard = self.0.borrow_mut();
        let Some(inner) = guard.as_mut() else { return };
        inner.ctx.ensure_running();
        inner.sfx.play(&inner.ctx.inner, sfx);
    }

    // ── Cries ─────────────────────────────────────────────────────────────

    pub fn play_cry(&self, pokemon_id: u32) {
        let manager = self.clone();
        leptos::task::spawn_local(async move {
            // Estrai ctx e volume senza tenere il borrow attraverso l'await
            let (ctx_ref, volume) = {
                let guard = manager.0.borrow();
                let Some(inner) = guard.as_ref() else { return };
                (inner.ctx.inner.clone(), inner.volume.effective_cries())
            }; // borrow rilasciato prima dell'await — play_sfx può ora accedere liberamente

            cries::CriesPlayer::play_id_standalone(&ctx_ref, volume, pokemon_id).await;
        });
    }

    // ── Visibility / mobile ───────────────────────────────────────────────

    pub fn on_visible(&self) {
        let guard = self.0.borrow();
        let Some(inner) = guard.as_ref() else { return };
        inner.ctx.ensure_running();
    }

    pub fn on_hidden(&self) {
        let guard = self.0.borrow();
        let Some(inner) = guard.as_ref() else { return };
        let _ = inner.ctx.inner.suspend();
    }

    // ── Preload SFX ───────────────────────────────────────────────────────

    async fn preload_sfx(&self) {
        use music::fetch_audio_buffer;
        let sfx_list = [
            Sfx::HitPhysical, Sfx::HitSpecial,
            Sfx::HitSuperEffective, Sfx::HitNotEffective,
            Sfx::Faint, Sfx::LevelUp,
            Sfx::MenuSelect, Sfx::MenuConfirm,
            Sfx::CatchSuccess, Sfx::PokemonHealed,
        ];

        for sfx in sfx_list {
            let ctx_ref = {
                let guard = self.0.borrow();
                let Some(inner) = guard.as_ref() else { return };
                inner.ctx.inner.clone()
            };
            if let Ok(buffer) = fetch_audio_buffer(&ctx_ref, sfx.path()).await {
                let mut guard = self.0.borrow_mut();
                let Some(inner) = guard.as_mut() else { return };
                inner.sfx.cache_buffer(sfx, buffer);
            }
        }
    }
}

fn setup_visibility_handler(manager: AudioManager) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else { return };
    let closure = Closure::<dyn Fn()>::new(move || {
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if doc.hidden() {
                manager.on_hidden();
            } else {
                manager.on_visible();
            }
        }
    });
    let _ = document.add_event_listener_with_callback(
        "visibilitychange",
        closure.as_ref().unchecked_ref(),
    );
    closure.forget(); // vive per tutta la durata della pagina
}

/// Fornisce l'AudioManager come context Leptos e lo registra.
pub fn provide_audio_manager() -> AudioManager {
    let manager = AudioManager::new();
    provide_context(manager.clone());
    manager
}

pub fn use_audio() -> AudioManager {
    use_context::<AudioManager>().expect("AudioManager non trovato nel context")
}
