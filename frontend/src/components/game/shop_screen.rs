use leptos::prelude::*;
use game_core::inventory::ItemKind;
use game_core::run::RunPhase;
use crate::pages::game::use_game_context;

const SHOP_ITEMS: &[ItemKind] = &[
    ItemKind::Potion,
    ItemKind::SuperPotion,
    ItemKind::FullRestore,
    ItemKind::Revive,
    ItemKind::Ether,
    ItemKind::MaxEther,
];

fn item_icon(kind: &ItemKind) -> &'static str {
    match kind {
        ItemKind::Potion        => "🧪",
        ItemKind::SuperPotion   => "💊",
        ItemKind::FullRestore   => "✨",
        ItemKind::Revive        => "💫",
        ItemKind::Ether         => "🔵",
        ItemKind::MaxEther      => "⚡",
    }
}

#[component]
pub fn ShopScreen() -> impl IntoView {
    let ctx = use_game_context();
    let message = RwSignal::new(None::<(String, bool)>); // (testo, successo)

    let on_buy = move |kind: ItemKind| {
        ctx.run.update(|r| {
            let Some(run) = r else { return };
            if run.inventory.buy(kind.clone(), 1) {
                message.set(Some((format!("{} acquistato!", kind.name()), true)));
            } else {
                message.set(Some(("Soldi insufficienti!".to_string(), false)));
            }
        });
    };

    let on_close = move |_| {
        ctx.run.update(|r| {
            if let Some(run) = r {
                run.phase = RunPhase::PostBattle;
            }
        });
    };

    view! {
        <div class="shop-screen">
            <div class="shop-screen__header">
                <h2 class="shop-screen__title">"🛒 Negozio"</h2>
                {move || ctx.run.read().as_ref().map(|r| view! {
                    <span class="shop-screen__money">"₽ " {r.inventory.money}</span>
                })}
            </div>

            {move || message.get().map(|(msg, ok)| {
                let style = if ok {
                    "shop-screen__message shop-screen__message--ok"
                } else {
                    "shop-screen__message shop-screen__message--err"
                };
                view! { <p class=style>{msg}</p> }
            })}

            <div class="shop-screen__items">
                {SHOP_ITEMS.iter().map(|kind| {
                    let kind = kind.clone();
                    let on_buy = on_buy.clone();
                    let icon = item_icon(&kind);
                    let desc = kind.description();
                    view! {
                        <div class="shop-screen__item">
                            <span class="shop-screen__item-icon">{icon}</span>
                            <div class="shop-screen__item-info">
                                <span class="shop-screen__item-name">{kind.name()}</span>
                            </div>
                            <div class="shop-screen__item-right">
                                <span class="shop-screen__item-price">"₽ " {kind.price()}</span>
                                <button
                                    class="shop-screen__buy-btn"
                                    on:click=move |_| on_buy(kind.clone())
                                >
                                    "Compra"
                                </button>
                            </div>
                            <span class="shop-screen__item-desc">{desc}</span>
                        </div>
                    }
                }).collect_view()}
            </div>

            <button class="shop-screen__close-btn" on:click=on_close>
                "← Torna alla base"
            </button>
        </div>
    }
}
