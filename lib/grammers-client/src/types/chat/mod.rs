// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
mod channel;
mod group;
mod user;

use grammers_session::PackedType;
use grammers_tl_types as tl;

pub use channel::Channel;
pub use grammers_session::PackedChat;
pub use group::Group;
pub use user::{Platform, RestrictionReason, User};

/// A chat.
///
/// Chats represent places where you can share messages with others.
///
/// * Private conversations with other people are treated as the chat of the user itself.
/// * Conversations in a group, whether it's private or public, are simply known as groups.
/// * Conversations where only administrators broadcast messages are known as channels.
#[derive(Clone, Debug)]
pub enum Chat {
    /// A [`User`].
    User(User),

    /// A [`Group`] chat.
    Group(Group),

    /// A broadcast [`Channel`].
    Channel(Channel),
}

impl Chat {
    pub(crate) fn from_user(user: tl::enums::User) -> Self {
        Self::User(User::from_raw(user))
    }

    pub(crate) fn from_chat(chat: tl::enums::Chat) -> Self {
        use tl::enums::Chat as C;

        match chat {
            C::Empty(_) | C::Chat(_) | C::Forbidden(_) => Self::Group(Group::from_raw(chat)),
            C::Channel(ref channel) => {
                if channel.broadcast {
                    Self::Channel(Channel::from_raw(chat))
                } else {
                    Self::Group(Group::from_raw(chat))
                }
            }
            C::ChannelForbidden(ref channel) => {
                if channel.broadcast {
                    Self::Channel(Channel::from_raw(chat))
                } else {
                    Self::Group(Group::from_raw(chat))
                }
            }
        }
    }

    /// Return the unique identifier for this chat.
    ///
    /// Every account will see the same identifier for the same chat.
    ///
    /// This identifier will never change. However, small group chats may be migrated to
    /// megagroups. If this happens, both the old small group chat and the new megagroup
    /// exist as separate chats with different identifiers, but they are linked with a
    /// property.
    pub fn id(&self) -> i32 {
        match self {
            Self::User(user) => user.id(),
            Self::Group(group) => group.id(),
            Self::Channel(channel) => channel.id(),
        }
    }

    /// Return the access hash for this chat
    pub fn access_hash(&self) -> Option<i64> {
        match self {
            Self::User(user) => user.access_hash(),
            Self::Group(grp) => grp.access_hash(),
            Self::Channel(chan) => chan.access_hash(),
        }
    }

    /// Return the name of this chat.
    ///
    /// For private conversations (users), this is their first name. For groups and channels,
    /// this is their title.
    ///
    /// The name may be empty if the chat is inaccessible or if the account was deleted.
    pub fn name(&self) -> &str {
        match self {
            Self::User(user) => user.first_name(),
            Self::Group(group) => group.title(),
            Self::Channel(channel) => channel.title(),
        }
    }

    /// Pack this chat into a smaller representation that can be loaded later.
    pub fn pack(&self) -> PackedChat {
        match self {
            Self::User(user) => user.pack(),
            Self::Group(chat) => chat.pack(),
            Self::Channel(channel) => channel.pack(),
        }
    }

    pub(crate) fn unpack(packed: PackedChat) -> Self {
        match packed.ty {
            PackedType::User | PackedType::Bot => {
                let mut user = User::from_raw(tl::types::UserEmpty { id: packed.id }.into());
                user.0.access_hash = packed.access_hash;
                Chat::User(user)
            }
            PackedType::Chat => Chat::Group(Group::from_raw(
                tl::types::ChatEmpty { id: packed.id }.into(),
            )),
            PackedType::Megagroup => Chat::Group(Group::from_raw(
                tl::types::ChannelForbidden {
                    id: packed.id,
                    broadcast: false,
                    megagroup: true,
                    access_hash: packed.access_hash.unwrap_or(0),
                    title: String::new(),
                    until_date: None,
                }
                .into(),
            )),
            PackedType::Broadcast | PackedType::Gigagroup => Chat::Channel(Channel::from_raw(
                tl::types::ChannelForbidden {
                    id: packed.id,
                    broadcast: true,
                    megagroup: false,
                    access_hash: packed.access_hash.unwrap_or(0),
                    title: String::new(),
                    until_date: None,
                }
                .into(),
            )),
        }
    }
}

impl From<Chat> for PackedChat {
    fn from(chat: Chat) -> Self {
        chat.pack()
    }
}

impl From<&Chat> for PackedChat {
    fn from(chat: &Chat) -> Self {
        chat.pack()
    }
}
