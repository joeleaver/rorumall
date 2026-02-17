use rinch::prelude::*;
use rorumall_shared::{Channel, Group, UserJoinedGroup};
use std::cell::RefCell;

#[derive(Clone, Copy)]
pub struct GroupsStore {
    pub joined_groups: Signal<Vec<UserJoinedGroup>>,
    pub current_group: Signal<Option<Group>>,
    pub channels: Signal<Vec<Channel>>,
    pub current_channel_id: Signal<Option<String>>,
}

thread_local! {
    static GROUPS_STORE: RefCell<Option<GroupsStore>> = const { RefCell::new(None) };
}

impl GroupsStore {
    pub fn init() -> Self {
        let joined_groups = use_signal(|| Vec::<UserJoinedGroup>::new());
        let current_group = use_signal(|| None::<Group>);
        let channels = use_signal(|| Vec::<Channel>::new());
        let current_channel_id = use_signal(|| None::<String>);

        let store = Self {
            joined_groups,
            current_group,
            channels,
            current_channel_id,
        };

        GROUPS_STORE.with(|s| {
            *s.borrow_mut() = Some(store);
        });

        store
    }

    pub fn set_joined_groups(&self, groups: Vec<UserJoinedGroup>) {
        self.joined_groups.set(groups);
    }

    pub fn set_current_group(&self, group: Option<Group>) {
        self.current_group.set(group);
    }

    pub fn set_channels(&self, channels: Vec<Channel>) {
        self.channels.set(channels);
    }

    pub fn set_current_channel(&self, channel_id: Option<String>) {
        self.current_channel_id.set(channel_id);
    }

    pub fn add_joined_group(&self, group: UserJoinedGroup) {
        self.joined_groups.update(|groups| {
            if !groups.iter().any(|g| g.group_id == group.group_id) {
                groups.push(group);
            }
        });
    }

    pub fn remove_joined_group(&self, group_id: &str) {
        self.joined_groups
            .update(|groups| groups.retain(|g| g.group_id != group_id));
    }
}

pub fn get_groups_store() -> GroupsStore {
    GROUPS_STORE.with(|s| {
        s.borrow()
            .expect("GroupsStore not initialized")
    })
}
