use rinch::prelude::*;
use std::cell::RefCell;

use crate::auth_session::AuthSession;

#[derive(Clone, Copy)]
pub struct AuthStore {
    pub session: Signal<Option<AuthSession>>,
    pub is_loading: Signal<bool>,
    pub error: Signal<Option<String>>,
    pub server_url: Signal<String>,
}

thread_local! {
    static AUTH_STORE: RefCell<Option<AuthStore>> = const { RefCell::new(None) };
}

impl AuthStore {
    pub fn init() -> Self {
        let session = use_signal(|| crate::storage::load::<AuthSession>("ofscp_session"));
        let is_loading = use_signal(|| false);
        let error = use_signal(|| None::<String>);
        let server_url = use_signal(|| crate::auth_session::load_domain());

        let store = Self {
            session,
            is_loading,
            error,
            server_url,
        };

        AUTH_STORE.with(|s| {
            *s.borrow_mut() = Some(store);
        });

        store
    }

    pub fn set_loading(&self, loading: bool) {
        self.is_loading.set(loading);
    }

    pub fn set_error(&self, err: impl Into<String>) {
        self.error.set(Some(err.into()));
    }

    pub fn clear_error(&self) {
        self.error.set(None);
    }

    pub fn set_session(&self, session: AuthSession) {
        crate::auth_session::save_session(&session);
        self.session.set(Some(session));
    }

    pub fn clear_session(&self) {
        crate::auth_session::clear_session();
        self.session.set(None);
    }

    pub fn set_server_url(&self, url: String) {
        crate::auth_session::save_domain(&url);
        self.server_url.set(url);
    }

    pub fn is_authenticated(&self) -> bool {
        self.session.get().is_some()
    }

    pub fn user_id(&self) -> Option<String> {
        self.session.get().map(|s| s.user_id.clone())
    }

    pub fn handle(&self) -> Option<String> {
        self.session
            .get()
            .map(|s| s.user_id.split('@').next().unwrap_or(&s.user_id).to_string())
    }

    pub fn domain(&self) -> String {
        self.server_url.get().clone()
    }

    pub fn make_client(&self) -> crate::api_client::ApiClient {
        let session = self.session.get();
        let domain = self.server_url.get();
        crate::auth_session::make_client(session.as_ref(), &domain)
    }
}

pub fn get_auth_store() -> AuthStore {
    AUTH_STORE.with(|s| {
        s.borrow()
            .expect("AuthStore not initialized - call AuthStore::init first")
    })
}
