use rinch::prelude::*;
use std::cell::RefCell;

#[derive(Clone, Debug, PartialEq)]
pub enum AppRoute {
    Login,
    Register,
    Home,
    Group { host: String, group_id: String },
    Channel { host: String, group_id: String, channel_id: String },
    ComposeArticle { host: String, group_id: String, channel_id: String },
    Profile,
}

thread_local! {
    static NAV_SIGNAL: RefCell<Option<Signal<AppRoute>>> = const { RefCell::new(None) };
}

pub fn init_nav() {
    // Check if there's an existing session — start at Home if so
    let initial_route = if crate::auth_session::load_session().is_some() {
        AppRoute::Home
    } else {
        AppRoute::Login
    };
    let sig = use_signal(|| initial_route);
    NAV_SIGNAL.with(|n| {
        *n.borrow_mut() = Some(sig);
    });
}

pub fn get_nav() -> Signal<AppRoute> {
    NAV_SIGNAL.with(|n| {
        n.borrow()
            .expect("Navigation not initialized - call init_nav first")
    })
}

pub fn navigate(route: AppRoute) {
    get_nav().set(route);
}

pub fn navigate_home() {
    navigate(AppRoute::Home);
}

pub fn navigate_login() {
    navigate(AppRoute::Login);
}

pub fn navigate_to_channel(host: String, group_id: String, channel_id: String) {
    navigate(AppRoute::Channel {
        host,
        group_id,
        channel_id,
    });
}

pub fn navigate_to_group(host: String, group_id: String) {
    navigate(AppRoute::Group { host, group_id });
}
