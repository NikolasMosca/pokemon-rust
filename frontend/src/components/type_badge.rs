use leptos::prelude::*;

fn type_modifier(type_name: &str) -> &'static str {
    match type_name {
        "normal"   => "type-badge--normal",
        "fire"     => "type-badge--fire",
        "water"    => "type-badge--water",
        "electric" => "type-badge--electric",
        "grass"    => "type-badge--grass",
        "ice"      => "type-badge--ice",
        "fighting" => "type-badge--fighting",
        "poison"   => "type-badge--poison",
        "ground"   => "type-badge--ground",
        "flying"   => "type-badge--flying",
        "psychic"  => "type-badge--psychic",
        "bug"      => "type-badge--bug",
        "rock"     => "type-badge--rock",
        "ghost"    => "type-badge--ghost",
        "dragon"   => "type-badge--dragon",
        "dark"     => "type-badge--dark",
        "steel"    => "type-badge--steel",
        "fairy"    => "type-badge--fairy",
        _          => "type-badge--unknown",
    }
}

#[component]
pub fn TypeBadge(#[prop(into)] type_name: String) -> impl IntoView {
    let modifier = type_modifier(&type_name);
    let class = format!("type-badge {modifier}");
    view! {
        <span class=class>{type_name}</span>
    }
}
