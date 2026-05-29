#![allow(non_snake_case)]

mod audio;
mod components;
mod core;
mod pages;

use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use audio::provide_audio_manager;
use core::cache::provide_pokemon_cache;
use core::storage::provide_store;
use pages::{game::GamePage, home::HomePage, ui_kit::UiKitPage};

#[component]
fn App() -> impl IntoView {
    provide_audio_manager();
    provide_pokemon_cache();

    leptos::task::spawn_local(async {
        provide_store().await;
    });

    view! {
        <Router>
            <Routes fallback=|| view! { <p>"404"</p> }>
                <Route path=path!("/") view=HomePage />
                <Route path=path!("/game") view=GamePage />
                <Route path=path!("/ui-kit") view=UiKitPage />
            </Routes>
        </Router>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
