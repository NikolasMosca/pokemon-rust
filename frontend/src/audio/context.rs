use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{AudioContext, AudioContextOptions, AudioContextState};

/// Wrapper attorno ad AudioContext con unlock automatico per mobile.
/// I browser mobile richiedono che l'AudioContext venga ripreso
/// dopo una interazione utente — questo viene gestito qui.
///
/// Clone è possibile perché AudioContext è un JsValue (reference-counted JS object).
#[derive(Clone)]
pub struct AudioCtx {
    pub inner: AudioContext,
}

impl AudioCtx {
    pub fn new() -> Result<Self, JsValue> {
        let opts = AudioContextOptions::new();
        // sample rate standard — lasciamo al browser scegliere
        let ctx = AudioContext::new_with_context_options(&opts)?;
        Ok(Self { inner: ctx })
    }

    /// Chiama resume() se il context è sospeso (mobile safety).
    /// Va chiamato dentro un event handler utente (click/touch).
    pub fn ensure_running(&self) {
        if self.inner.state() == AudioContextState::Suspended {
            let _ = self.inner.resume();
        }
    }

    pub fn current_time(&self) -> f64 {
        self.inner.current_time()
    }

    pub fn destination(&self) -> web_sys::AudioDestinationNode {
        self.inner.destination()
    }
}
