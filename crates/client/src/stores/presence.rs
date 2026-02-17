use rinch::prelude::*;
use rorumall_shared::{Availability, Presence};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct PresenceStore {
    pub current: Signal<Option<Presence>>,
    pub others: Signal<HashMap<String, Presence>>,
}

thread_local! {
    static PRESENCE_STORE: RefCell<Option<PresenceStore>> = const { RefCell::new(None) };
}

impl PresenceStore {
    pub fn init() -> Self {
        let current = use_signal(|| None::<Presence>);
        let others = use_signal(|| HashMap::<String, Presence>::new());

        let store = Self { current, others };

        PRESENCE_STORE.with(|s| {
            *s.borrow_mut() = Some(store);
        });

        store
    }

    pub fn set_current(&self, presence: Presence) {
        self.current.set(Some(presence));
    }

    pub fn clear_current(&self) {
        self.current.set(None);
    }

    pub fn update_user(&self, handle: &str, domain: &str, presence: Presence) {
        let key = format!("{}@{}", handle, domain);
        self.others.update(|m| { m.insert(key, presence); });
    }

    pub fn get_user(&self, handle: &str, domain: &str) -> Option<Presence> {
        let key = format!("{}@{}", handle, domain);
        self.others.get().get(&key).cloned()
    }

    pub fn get_availability(&self, handle: &str, domain: &str) -> Availability {
        self.get_user(handle, domain)
            .map(|p| p.availability)
            .unwrap_or(Availability::Offline)
    }
}

pub fn get_presence_store() -> PresenceStore {
    PRESENCE_STORE.with(|s| {
        s.borrow()
            .expect("PresenceStore not initialized")
    })
}
