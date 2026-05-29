use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum HpState {
    Full,
    Mid,
    Low,
    Critical,
}

impl HpState {
    pub fn from_percent(pct: f64) -> Self {
        if pct > 50.0 { Self::Full }
        else if pct > 25.0 { Self::Mid }
        else if pct > 10.0 { Self::Low }
        else { Self::Critical }
    }

    fn modifier(&self) -> &'static str {
        match self {
            Self::Full => "hp-bar--full",
            Self::Mid => "hp-bar--mid",
            Self::Low => "hp-bar--low",
            Self::Critical => "hp-bar--critical",
        }
    }
}

#[component]
pub fn HPBar(#[prop(into)] hp: Signal<u32>, #[prop(into)] max_hp: Signal<u32>) -> impl IntoView {
    let percent = move || {
        let h = hp.try_get().unwrap_or(0);
        let m = max_hp.try_get().unwrap_or(1).max(1);
        h as f64 / m as f64 * 100.0
    };
    let state = move || HpState::from_percent(percent());
    let modifier = move || state().modifier();
    let width = move || format!("{:.1}%", percent().min(100.0));

    view! {
        <div class="hp-bar">
            <div class="hp-bar__track">
                <div
                    class=move || format!("hp-bar__fill {}", modifier())
                    style=move || format!("width: {}", width())
                />
            </div>
        </div>
    }
}
