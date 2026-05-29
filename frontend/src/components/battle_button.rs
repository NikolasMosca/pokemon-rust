use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum BattleAction {
    Fight,
    Bag,
    Pokemon,
    Run,
}

impl BattleAction {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Fight => "LOTTA",
            Self::Bag => "BORSA",
            Self::Pokemon => "POKéMON",
            Self::Run => "FUGA",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Fight => "⚔️",
            Self::Bag => "🎒",
            Self::Pokemon => "🔴",
            Self::Run => "💨",
        }
    }

    fn modifier(&self) -> &'static str {
        match self {
            Self::Fight => "battle-button--fight",
            Self::Bag => "battle-button--bag",
            Self::Pokemon => "battle-button--pokemon",
            Self::Run => "battle-button--run",
        }
    }
}

#[component]
pub fn BattleButton(
    action: BattleAction,
    on_click: impl Fn(BattleAction) + 'static,
) -> impl IntoView {
    let class = format!("battle-button {}", action.modifier());
    view! {
        <button class=class on:click=move |_| on_click(action)>
            <span class="battle-button__icon">{action.icon()}</span>
            <span class="battle-button__label">{action.label()}</span>
        </button>
    }
}
