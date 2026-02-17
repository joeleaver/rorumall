use rinch::prelude::*;
use rorumall_shared::UserProfile;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct ProfileStore {
    pub current: Signal<Option<UserProfile>>,
    pub cache: Signal<HashMap<String, UserProfile>>,
}

thread_local! {
    static PROFILE_STORE: RefCell<Option<ProfileStore>> = const { RefCell::new(None) };
}

impl ProfileStore {
    pub fn init() -> Self {
        let current = use_signal(|| None::<UserProfile>);
        let cache = use_signal(|| HashMap::<String, UserProfile>::new());

        let store = Self { current, cache };

        PROFILE_STORE.with(|s| {
            *s.borrow_mut() = Some(store);
        });

        store
    }

    pub fn set_current(&self, profile: UserProfile) {
        self.current.set(Some(profile));
    }

    pub fn clear_current(&self) {
        self.current.set(None);
    }

    pub fn cache_profile(&self, profile: UserProfile) {
        let key = format!("{}@{}", profile.handle, profile.domain);
        self.cache.update(|m| { m.insert(key, profile); });
    }

    pub fn get_cached(&self, handle: &str, domain: &str) -> Option<UserProfile> {
        let key = format!("{}@{}", handle, domain);
        self.cache.get().get(&key).cloned()
    }
}

pub fn get_profile_store() -> ProfileStore {
    PROFILE_STORE.with(|s| {
        s.borrow()
            .expect("ProfileStore not initialized")
    })
}
