/// Valori default volutamente bassi per non essere invasivi.
pub const DEFAULT_MASTER: f32 = 0.6;
pub const DEFAULT_MUSIC:  f32 = 0.4;
pub const DEFAULT_SFX:    f32 = 0.7;
pub const DEFAULT_CRIES:  f32 = 0.52;

const KEY_MASTER: &str = "audio_master";
const KEY_MUSIC:  &str = "audio_music";
const KEY_SFX:    &str = "audio_sfx";
const KEY_CRIES:  &str = "audio_cries";

#[derive(Clone, Copy, Debug)]
pub struct VolumeGroups {
    pub master: f32,
    pub music:  f32,
    pub sfx:    f32,
    pub cries:  f32,
}

impl VolumeGroups {
    pub fn load() -> Self {
        Self {
            master: load_f32(KEY_MASTER, DEFAULT_MASTER),
            music:  load_f32(KEY_MUSIC,  DEFAULT_MUSIC),
            sfx:    load_f32(KEY_SFX,    DEFAULT_SFX),
            cries:  load_f32(KEY_CRIES,  DEFAULT_CRIES),
        }
    }

    pub fn save(&self) {
        save_f32(KEY_MASTER, self.master);
        save_f32(KEY_MUSIC,  self.music);
        save_f32(KEY_SFX,    self.sfx);
        save_f32(KEY_CRIES,  self.cries);
    }

    /// Volume effettivo = group * master
    pub fn effective_music(&self)  -> f32 { self.music  * self.master }
    pub fn effective_sfx(&self)    -> f32 { self.sfx    * self.master }
    pub fn effective_cries(&self)  -> f32 { self.cries  * self.master }
}

fn load_f32(key: &str, default: f32) -> f32 {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(key).ok().flatten())
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(default)
}

fn save_f32(key: &str, value: f32) {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
    {
        let _ = storage.set_item(key, &value.to_string());
    }
}
