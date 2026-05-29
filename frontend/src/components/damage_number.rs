use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum DamageKind {
    Damage,
    Heal,
}

#[component]
pub fn DamageNumber(
    value: ReadSignal<Option<(i32, DamageKind)>>,
) -> impl IntoView {
    let class = move || match value.get() {
        Some((_, DamageKind::Heal)) => "damage-number damage-number--heal damage-number--visible",
        Some((_, DamageKind::Damage)) => "damage-number damage-number--damage damage-number--visible",
        None => "damage-number",
    };

    let text = move || match value.get() {
        Some((v, DamageKind::Heal)) => format!("+{}", v),
        Some((v, DamageKind::Damage)) => format!("-{}", v),
        None => String::new(),
    };

    view! {
        <div class=class>{text}</div>
    }
}
