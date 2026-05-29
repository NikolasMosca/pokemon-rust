use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::{AudioBuffer, AudioContext, GainNode};

/// Suoni SFX disponibili.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Sfx {
    HitPhysical,
    HitSpecial,
    HitSuperEffective,
    HitNotEffective,
    Faint,
    CatchSuccess,
    LevelUp,
    MenuSelect,
    MenuConfirm,
    PokemonHealed,
}

impl Sfx {
    pub fn path(&self) -> &'static str {
        match self {
            Self::HitPhysical       => "/assets/audio/sfx/hit_physical.ogg",
            Self::HitSpecial        => "/assets/audio/sfx/hit_special.ogg",
            Self::HitSuperEffective => "/assets/audio/sfx/hit_supereffective.ogg",
            Self::HitNotEffective   => "/assets/audio/sfx/hit_noteffective.ogg",
            Self::Faint             => "/assets/audio/sfx/faint.ogg",
            Self::CatchSuccess      => "/assets/audio/sfx/catch_success.ogg",
            Self::LevelUp           => "/assets/audio/sfx/level_up.m4a",
            Self::MenuSelect        => "/assets/audio/sfx/menu_select.ogg",
            Self::MenuConfirm       => "/assets/audio/sfx/menu_confirm.ogg",
            Self::PokemonHealed     => "/assets/audio/sfx/pokemon_healed.m4a",
        }
    }
}

/// Anti-spam: tempo minimo in ms tra due play dello stesso SFX.
const ANTI_SPAM_MS: f64 = 80.0;

pub struct SfxPlayer {
    buffers:    HashMap<Sfx, AudioBuffer>,
    gain:       GainNode,
    last_played: HashMap<Sfx, f64>,
}

impl SfxPlayer {
    pub fn new(ctx: &AudioContext, volume: f32) -> Result<Self, wasm_bindgen::JsValue> {
        let gain = ctx.create_gain()?;
        gain.gain().set_value(volume);
        gain.connect_with_audio_node(&ctx.destination())?;
        Ok(Self {
            buffers: HashMap::new(),
            gain,
            last_played: HashMap::new(),
        })
    }

    pub fn cache_buffer(&mut self, sfx: Sfx, buffer: AudioBuffer) {
        self.buffers.insert(sfx, buffer);
    }

    /// Riproduce il suono se disponibile in cache e non in anti-spam.
    pub fn play(&mut self, ctx: &AudioContext, sfx: Sfx) {
        let now = ctx.current_time();
        let last = self.last_played.get(&sfx).copied().unwrap_or(0.0);
        if (now - last) * 1000.0 < ANTI_SPAM_MS {
            return;
        }
        let Some(buffer) = self.buffers.get(&sfx) else { return };
        let Ok(source) = ctx.create_buffer_source() else { return };
        source.set_buffer(Some(buffer));
        let _ = source.connect_with_audio_node(&self.gain);
        let _ = source.start();
        self.last_played.insert(sfx, now);
    }

    pub fn set_volume(&self, volume: f32) {
        self.gain.gain().set_value(volume);
    }
}
