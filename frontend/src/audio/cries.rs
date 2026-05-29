use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AudioBuffer, AudioContext, GainNode};

const CRIES_BASE: &str =
    "https://raw.githubusercontent.com/PokeAPI/cries/main/cries/pokemon/latest";

pub struct CriesPlayer {
    gain: GainNode,
}

impl CriesPlayer {
    pub fn new(ctx: &AudioContext, volume: f32) -> Result<Self, wasm_bindgen::JsValue> {
        let gain = ctx.create_gain()?;
        gain.gain().set_value(volume);
        gain.connect_with_audio_node(&ctx.destination())?;
        Ok(Self { gain })
    }

    pub fn set_volume(&self, volume: f32) {
        self.gain.gain().set_value(volume);
    }

    /// Versione standalone: non richiede &mut self, non tiene borrow aperto
    /// attraverso await. Usa solo Cache API del browser come cache persistente.
    pub async fn play_id_standalone(ctx: &AudioContext, volume: f32, pokemon_id: u32) {
        let url = format!("{}/{}.ogg", CRIES_BASE, pokemon_id);
        let buffer = match fetch_and_cache(ctx, &url).await {
            Ok(b) => b,
            Err(e) => {
                web_sys::console::warn_1(
                    &format!("cry fetch failed for id {pokemon_id}: {e}").into()
                );
                return;
            }
        };

        let Ok(gain) = ctx.create_gain() else { return };
        gain.gain().set_value(volume);
        let _ = gain.connect_with_audio_node(&ctx.destination());

        let Ok(source) = ctx.create_buffer_source() else { return };
        source.set_buffer(Some(&buffer));
        let _ = source.connect_with_audio_node(&gain);
        let _ = source.start();
    }
}

/// Fetcha un audio URL passando prima dalla Cache API del browser.
/// Se non è in cache, lo scarica e lo salva per le sessioni future.
async fn fetch_and_cache(ctx: &AudioContext, url: &str) -> Result<AudioBuffer, String> {
    let window = web_sys::window().ok_or("no window")?;

    // Prova dalla Cache API
    if let Some(buffer) = try_from_cache(ctx, url).await {
        return Ok(buffer);
    }

    // Fetch dalla rete
    let resp_val = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|e| format!("{:?}", e))?;
    let resp: web_sys::Response = resp_val.dyn_into().map_err(|_| "not Response")?;

    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }

    // Salva in cache per sessioni future (fire-and-forget)
    if let Ok(caches) = window.caches() {
        let url_owned = url.to_string();
        let fut = async move {
            let Ok(cache_val) = JsFuture::from(caches.open("cries-v1")).await else { return };
            let cache: web_sys::Cache = match cache_val.dyn_into() {
                Ok(c) => c, Err(_) => return,
            };
            let _ = JsFuture::from(cache.add_with_str(&url_owned)).await;
        };
        leptos::task::spawn_local(fut);
    }

    let ab = JsFuture::from(resp.array_buffer().map_err(|e| format!("{:?}", e))?)
        .await
        .map_err(|e| format!("{:?}", e))?;

    let decoded = JsFuture::from(
        ctx.decode_audio_data(&ab.into()).map_err(|e| format!("{:?}", e))?
    )
    .await
    .map_err(|e| format!("{:?}", e))?;

    decoded.dyn_into::<AudioBuffer>().map_err(|_| "not AudioBuffer".to_string())
}

async fn try_from_cache(ctx: &AudioContext, url: &str) -> Option<AudioBuffer> {
    let window = web_sys::window()?;
    let caches = window.caches().ok()?;
    let cache: web_sys::Cache = JsFuture::from(caches.open("cries-v1"))
        .await.ok()?.dyn_into().ok()?;
    let resp: web_sys::Response = JsFuture::from(cache.match_with_str(url))
        .await.ok()?.dyn_into().ok()?;
    let ab = JsFuture::from(resp.array_buffer().ok()?).await.ok()?;
    JsFuture::from(ctx.decode_audio_data(&ab.into()).ok()?)
        .await.ok()?.dyn_into::<AudioBuffer>().ok()
}
