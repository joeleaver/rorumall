use rinch::prelude::*;
use crate::navigation::{init_nav, get_nav, AppRoute};
use crate::stores::{AuthStore, GroupsStore, MessagesStore, MembersStore, PresenceStore, ProfileStore};

#[component]
pub fn app() -> NodeHandle {
    // Initialize navigation
    init_nav();

    // Initialize all stores
    AuthStore::init();
    GroupsStore::init();
    MessagesStore::init();
    MembersStore::init();
    PresenceStore::init();
    ProfileStore::init();

    let nav = get_nav();

    rsx! {
        div {
            class: "app-root",
            style: "width: 100vw; height: 100vh; display: flex; flex-direction: column;",

            if matches!(nav.get().clone(), AppRoute::Login) {
                div {
                    style: "flex: 1; display: flex; flex-direction: column;",
                    {crate::views::login::login_view(__scope)}
                }
            }
            if matches!(nav.get().clone(), AppRoute::Register) {
                div {
                    style: "flex: 1; display: flex; flex-direction: column;",
                    {crate::views::register::register_view(__scope)}
                }
            }
            if matches!(nav.get().clone(), AppRoute::Home | AppRoute::Group { .. } | AppRoute::Channel { .. }) {
                div {
                    style: "flex: 1; display: flex; overflow: hidden;",
                    {crate::views::home::home_view(__scope)}
                }
            }
            if matches!(nav.get().clone(), AppRoute::ComposeArticle { .. }) {
                div {
                    style: "flex: 1; display: flex; flex-direction: column;",
                    {crate::views::article_editor::article_editor_view(__scope)}
                }
            }
            if matches!(nav.get().clone(), AppRoute::Profile) {
                div {
                    style: "flex: 1; display: flex; flex-direction: column; overflow: auto;",
                    {crate::views::profile_view::profile_view(__scope)}
                }
            }
        }
    }
}
