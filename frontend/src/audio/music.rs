use std::cell::RefCell;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext, GainNode};
use gloo_timers::future::TimeoutFuture;

/// Tracce musicali disponibili.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MusicTrack {
    BattleWild,
    BattleTrainer,
    BattleGym,
    VictoryWild,
    VictoryTrainer,
    Pokecenter,
}

impl MusicTrack {
    pub fn path(&self) -> &'static str {
        match self {
            Self::BattleWild      => "/assets/audio/music/battle_wild.m4a",
            Self::BattleTrainer   => "/assets/audio/music/battle_trainer.m4a",
            Self::BattleGym       => "/assets/audio/music/battle_gym.m4a",
            Self::VictoryWild     => "/assets/audio/music/victory_wild.m4a",
            Self::VictoryTrainer  => "/assets/audio/music/victory_trainer.m4a",
            Self::Pokecenter      => "/assets/audio/music/pokecenter.m4a",
        }
    }

    pub fn loops(&self) -> bool {
        true
    }
}

pub struct MusicPlayer {
    source: Option<AudioBufferSourceNode>,
    gain:   GainNode,
}

impl MusicPlayer {
    pub fn new(ctx: &AudioContext, volume: f32) -> Result<Self, wasm_bindgen::JsValue> {
        let gain = ctx.create_gain()?;
        gain.gain().set_value(volume);
        gain.connect_with_audio_node(&ctx.destination())?;
        Ok(Self { source: None, gain })
    }

    pub fn gain_node(&self) -> &GainNode {
        &self.gain
    }

    /// Ferma la traccia corrente (sincrono) e avvia la nuova a volume 0.
    /// Il fade-in va fatto separatamente via `crossfade_fade_in`.
    pub fn crossfade_start(
        &mut self,
        ctx: &AudioContext,
        buffer: AudioBuffer,
        track: MusicTrack,
        _volume: f32,
    ) {
        self.stop();
        self.gain.gain().set_value(0.0);
        let _ = self.play(ctx, buffer, track.loops());
    }

    /// Fade-in sul GainNode dato — non richiede &mut self, usabile fuori dal borrow.
    pub async fn crossfade_fade_in(gain: &GainNode, target: f32, fade_ms: u32) {
        let steps = 20u32;
        let step_ms = fade_ms / steps;
        let delta = target / steps as f32;
        for i in 1..=steps {
            let v = (delta * i as f32).clamp(0.0, 1.0);
            gain.gain().set_value(v);
            TimeoutFuture::new(step_ms).await;
        }
        gain.gain().set_value(target);
    }

    /// Ferma la traccia corrente con fade-out, poi avvia la nuova.
    /// Usare solo quando non c'è rischio di borrow sovrapposti.
    pub async fn crossfade(
        &mut self,
        ctx: &AudioContext,
        buffer: AudioBuffer,
        track: MusicTrack,
        volume: f32,
        fade_ms: u32,
    ) {
        // Fade out
        if self.source.is_some() {
            self.fade_to(0.0, fade_ms).await;
            self.stop();
        }
        // Fade in
        self.gain.gain().set_value(0.0);
        let _ = self.play(ctx, buffer, track.loops());
        self.fade_to(volume, fade_ms).await;
    }

    pub fn play(
        &mut self,
        ctx: &AudioContext,
        buffer: AudioBuffer,
        loop_: bool,
    ) -> Result<(), wasm_bindgen::JsValue> {
        let source = ctx.create_buffer_source()?;
        source.set_buffer(Some(&buffer));
        source.set_loop(loop_);
        source.connect_with_audio_node(&self.gain)?;
        source.start()?;
        self.source = Some(source);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(src) = self.source.take() {
            let _ = src.stop();
        }
    }

    pub fn set_volume(&self, volume: f32) {
        self.gain.gain().set_value(volume);
    }

    async fn fade_to(&self, target: f32, duration_ms: u32) {
        let steps = 20u32;
        let step_ms = duration_ms / steps;
        let current = self.gain.gain().value();
        let delta = (target - current) / steps as f32;
        for i in 1..=steps {
            let v = (current + delta * i as f32).clamp(0.0, 1.0);
            self.gain.gain().set_value(v);
            TimeoutFuture::new(step_ms).await;
        }
        self.gain.gain().set_value(target);
    }
}

/// Fetcha e decodifica un AudioBuffer dall'URL dato.
pub async fn fetch_audio_buffer(
    ctx: &AudioContext,
    url: &str,
) -> Result<AudioBuffer, String> {
    let window = web_sys::window().ok_or("no window")?;

    // Prova prima dalla Cache API
    if let Some(buffer) = try_cache(ctx, url).await {
        return Ok(buffer);
    }

    // Fetch dalla rete
    let resp = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|e| format!("fetch error: {:?}", e))?;
    let resp: web_sys::Response = resp.dyn_into().map_err(|_| "not a Response")?;

    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let array_buffer = JsFuture::from(
        resp.array_buffer().map_err(|e| format!("{:?}", e))?
    )
    .await
    .map_err(|e| format!("array_buffer error: {:?}", e))?;

    let audio_buffer = JsFuture::from(
        ctx.decode_audio_data(&array_buffer.into())
            .map_err(|e| format!("decode error: {:?}", e))?
    )
    .await
    .map_err(|e| format!("decode await error: {:?}", e))?;

    Ok(audio_buffer.dyn_into::<AudioBuffer>().map_err(|_| "not AudioBuffer")?)
}

async fn try_cache(ctx: &AudioContext, url: &str) -> Option<AudioBuffer> {
    let window = web_sys::window()?;
    let caches = window.caches().ok()?;
    let cache: web_sys::Cache = JsFuture::from(caches.open("audio-v1"))
        .await.ok()?.dyn_into().ok()?;

    let resp: web_sys::Response = JsFuture::from(cache.match_with_str(url))
        .await.ok()?.dyn_into().ok()?;

    let ab = JsFuture::from(resp.array_buffer().ok()?)
        .await.ok()?;

    JsFuture::from(ctx.decode_audio_data(&ab.into()).ok()?)
        .await.ok()?
        .dyn_into::<AudioBuffer>().ok()
}

/// Salva la risposta nella Cache API per uso futuro.
pub async fn cache_audio_url(url: &str) {
    let Some(window) = web_sys::window() else { return };
    let Ok(caches) = window.caches() else { return };
    let Ok(cache) = JsFuture::from(caches.open("audio-v1")).await else { return };
    let cache: web_sys::Cache = match cache.dyn_into() {
        Ok(c) => c, Err(_) => return,
    };
    let _ = JsFuture::from(cache.add_with_str(url)).await;
}
