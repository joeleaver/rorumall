use chrono::{DateTime, Utc};
use rinch::prelude::*;
use rorumall_shared::{Attachment, MessageType};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Debug)]
pub struct StoredMessage {
    pub id: String,
    pub user_id: String,
    pub title: Option<String>,
    pub content: String,
    pub message_type: MessageType,
    pub created_at: DateTime<Utc>,
    pub parent_id: Option<String>,
    pub parent_message_type: Option<MessageType>,
    pub attachments: Vec<Attachment>,
}

#[derive(Default, Clone, PartialEq)]
pub struct ChannelMessages {
    pub messages: Vec<StoredMessage>,
    pub is_loaded: bool,
}

impl ChannelMessages {
    pub fn add_message(&mut self, msg: StoredMessage) -> bool {
        if self.messages.iter().any(|m| m.id == msg.id) {
            return false;
        }
        let pos = self
            .messages
            .binary_search_by(|m| m.created_at.cmp(&msg.created_at))
            .unwrap_or_else(|pos| pos);
        self.messages.insert(pos, msg);
        true
    }

    pub fn set_history(&mut self, mut messages: Vec<StoredMessage>) {
        messages.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        self.messages = messages;
        self.is_loaded = true;
    }
}

#[derive(Clone, Copy)]
pub struct MessagesStore {
    pub messages: Signal<HashMap<String, ChannelMessages>>,
}

thread_local! {
    static MESSAGES_STORE: RefCell<Option<MessagesStore>> = const { RefCell::new(None) };
}

impl MessagesStore {
    pub fn init() -> Self {
        let messages = use_signal(|| HashMap::<String, ChannelMessages>::new());
        let store = Self { messages };
        MESSAGES_STORE.with(|s| {
            *s.borrow_mut() = Some(store);
        });
        store
    }

    pub fn get_channel_messages(&self, channel_id: &str) -> Option<ChannelMessages> {
        self.messages.get().get(channel_id).cloned()
    }

    pub fn add_message(&self, channel_id: &str, msg: StoredMessage) {
        self.messages.update(|map| {
            map.entry(channel_id.to_string())
                .or_default()
                .add_message(msg);
        });
    }

    pub fn set_channel_history(&self, channel_id: &str, messages: Vec<StoredMessage>) {
        self.messages.update(|map| {
            map.entry(channel_id.to_string())
                .or_default()
                .set_history(messages);
        });
    }

    pub fn is_channel_loaded(&self, channel_id: &str) -> bool {
        self.messages
            .get()
            .get(channel_id)
            .map(|ch| ch.is_loaded)
            .unwrap_or(false)
    }
}

pub fn get_messages_store() -> MessagesStore {
    MESSAGES_STORE.with(|s| {
        s.borrow()
            .expect("MessagesStore not initialized")
    })
}
