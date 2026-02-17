use rinch::prelude::*;
use rorumall_shared::{GroupMember, GroupRole};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct MembersStore {
    pub members: Signal<HashMap<String, Vec<GroupMember>>>,
    pub my_roles: Signal<HashMap<String, Vec<String>>>,
    pub group_roles: Signal<HashMap<String, Vec<GroupRole>>>,
}

thread_local! {
    static MEMBERS_STORE: RefCell<Option<MembersStore>> = const { RefCell::new(None) };
}

impl MembersStore {
    pub fn init() -> Self {
        let members = use_signal(|| HashMap::<String, Vec<GroupMember>>::new());
        let my_roles = use_signal(|| HashMap::<String, Vec<String>>::new());
        let group_roles = use_signal(|| HashMap::<String, Vec<GroupRole>>::new());

        let store = Self {
            members,
            my_roles,
            group_roles,
        };

        MEMBERS_STORE.with(|s| {
            *s.borrow_mut() = Some(store);
        });

        store
    }

    pub fn set_group_members(&self, group_id: &str, member_list: Vec<GroupMember>) {
        self.members
            .update(|m| { m.insert(group_id.to_string(), member_list); });
    }

    pub fn get_group_members(&self, group_id: &str) -> Option<Vec<GroupMember>> {
        self.members.get().get(group_id).cloned()
    }

    pub fn set_my_roles(&self, group_id: &str, roles: Vec<String>) {
        self.my_roles
            .update(|m| { m.insert(group_id.to_string(), roles); });
    }

    pub fn get_my_base_role(&self, group_id: &str) -> Option<String> {
        self.my_roles.get().get(group_id).map(|roles| {
            roles
                .iter()
                .find(|r| *r == "owner" || *r == "admin" || *r == "member")
                .cloned()
                .unwrap_or_else(|| "member".to_string())
        })
    }

    pub fn remove_member(&self, group_id: &str, user_id: &str) {
        self.members.update(|m| {
            if let Some(members) = m.get_mut(group_id) {
                members.retain(|member| member.user_id != user_id);
            }
        });
    }

    pub fn set_group_roles(&self, group_id: &str, roles: Vec<GroupRole>) {
        self.group_roles
            .update(|m| { m.insert(group_id.to_string(), roles); });
    }

    pub fn get_group_roles(&self, group_id: &str) -> Option<Vec<GroupRole>> {
        self.group_roles.get().get(group_id).cloned()
    }

    pub fn add_role(&self, group_id: &str, role: GroupRole) {
        self.group_roles.update(|m| {
            if let Some(roles) = m.get_mut(group_id) {
                let pos = roles
                    .iter()
                    .position(|r| r.position < role.position)
                    .unwrap_or(roles.len());
                roles.insert(pos, role);
            }
        });
    }

    pub fn remove_role(&self, group_id: &str, role_id: &str) {
        self.group_roles.update(|m| {
            if let Some(roles) = m.get_mut(group_id) {
                roles.retain(|r| r.id != role_id);
            }
        });
    }

    pub fn clear_group(&self, group_id: &str) {
        self.members.update(|m| { m.remove(group_id); });
        self.my_roles.update(|m| { m.remove(group_id); });
        self.group_roles.update(|m| { m.remove(group_id); });
    }
}

pub fn get_members_store() -> MembersStore {
    MEMBERS_STORE.with(|s| {
        s.borrow()
            .expect("MembersStore not initialized")
    })
}
