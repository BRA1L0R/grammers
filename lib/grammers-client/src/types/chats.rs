// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use crate::ClientHandle;
use grammers_mtsender::InvocationError;
use grammers_tl_types as tl;
use std::{
    future::Future,
    mem::drop,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

type FutOutput = Result<(), InvocationError>;
type FutStore = Pin<Box<dyn Future<Output = FutOutput> + Send>>;

/// Builder for editing the administrator rights of a user in a specific channel.
///
/// Use [`ClientHandle::set_admin_rights`] to retrieve an instance of this type.
pub struct AdminRightsBuilder {
    client: ClientHandle,
    channel: tl::enums::InputChannel,
    user: tl::enums::InputUser,
    rights: tl::types::ChatAdminRights,
    rank: String,
    fut: Option<FutStore>,
}

impl Future for AdminRightsBuilder {
    type Output = FutOutput;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<FutOutput> {
        if self.fut.is_none() {
            let call = tl::functions::channels::EditAdmin {
                channel: self.channel.clone(),
                user_id: self.user.clone(),
                admin_rights: tl::enums::ChatAdminRights::Rights(self.rights.clone()),
                rank: self.rank.clone(),
            };
            let mut c = self.client.clone();
            self.fut = Some(Box::pin(async move { c.invoke(&call).await.map(drop) }));
        }
        Future::poll(self.fut.as_mut().unwrap().as_mut(), cx)
    }
}

impl AdminRightsBuilder {
    pub(crate) fn new(
        client: ClientHandle,
        channel: tl::enums::InputChannel,
        user: tl::enums::InputUser,
    ) -> Self {
        Self {
            client,
            channel,
            user,
            rank: "".into(),
            rights: tl::types::ChatAdminRights {
                anonymous: false,
                change_info: false,
                post_messages: false,
                edit_messages: false,
                delete_messages: false,
                ban_users: false,
                invite_users: false,
                pin_messages: false,
                add_admins: false,
                manage_call: false,
            },
            fut: None,
        }
    }

    /// Load the current rights of the user. This lets you trivially grant or take away specific
    /// permissions without changing any of the previous ones.
    pub async fn load_current(&mut self) -> Result<&mut Self, InvocationError> {
        let tl::enums::channels::ChannelParticipant::Participant(user) = self
            .client
            .invoke(&tl::functions::channels::GetParticipant {
                channel: self.channel.clone(),
                user_id: self.user.clone(),
            })
            .await?;
        match user.participant {
            tl::enums::ChannelParticipant::Creator(c) => {
                self.rights = c.admin_rights.into();
                self.rank = c.rank.unwrap_or_else(String::new);
            }
            tl::enums::ChannelParticipant::Admin(a) => {
                self.rights = a.admin_rights.into();
                self.rank = a.rank.unwrap_or_else(String::new);
            }
            _ => (),
        }

        Ok(self)
    }

    /// Whether the user will remain anonymous when sending messages.
    ///
    /// The sender of the anonymous messages becomes the group itself.
    ///
    /// Note that other people in the channel may be able to identify the anonymous admin by its
    /// custom rank, so additional care is needed when using both anonymous and custom ranks.
    ///
    /// For example, if multiple anonymous admins share the same title, users won't be able to
    /// distinguish them.
    pub fn anonymous(&mut self, val: bool) -> &mut Self {
        self.rights.anonymous = val;
        self
    }

    /// Whether the user is able to manage calls in the group.
    pub fn manage_call(&mut self, val: bool) -> &mut Self {
        self.rights.manage_call = val;
        self
    }

    /// Whether the user is able to change information about the chat such as group description or
    /// not.
    pub fn change_info(&mut self, val: bool) -> &mut Self {
        self.rights.change_info = val;
        self
    }

    /// Whether the user will be able to post in the channel. This will only work in broadcast
    /// channels, not groups.
    pub fn post_messages(&mut self, val: bool) -> &mut Self {
        self.rights.post_messages = val;
        self
    }

    /// Whether the user will be able to edit messages in the channel. This will only work in
    /// broadcast channels, not groups.
    pub fn edit_messages(&mut self, val: bool) -> &mut Self {
        self.rights.edit_messages = val;
        self
    }

    /// Whether the user will be able to delete messages. This includes messages from others.
    pub fn delete_messages(&mut self, val: bool) -> &mut Self {
        self.rights.delete_messages = val;
        self
    }

    /// Whether the user will be able to edit the restrictions of other users. This effectively
    /// lets the administrator ban (or kick) people.
    pub fn ban_users(&mut self, val: bool) -> &mut Self {
        self.rights.ban_users = val;
        self
    }

    /// Whether the user will be able to invite other users.
    pub fn invite_users(&mut self, val: bool) -> &mut Self {
        self.rights.invite_users = val;
        self
    }

    /// Whether the user will be able to pin messages.
    pub fn pin_messages(&mut self, val: bool) -> &mut Self {
        self.rights.pin_messages = val;
        self
    }

    /// Whether the user will be able to add other administrators with the same or less
    /// permissions than the user itself.
    pub fn add_admins(&mut self, val: bool) -> &mut Self {
        self.rights.add_admins = val;
        self
    }

    /// The custom rank  (also known as "admin title" or "badge") to show for this administrator.
    ///
    /// This text will be shown instead of the "admin" badge.
    ///
    /// When left unspecified or empty, the default localized "admin" badge will be shown.
    pub fn rank<S: Into<String>>(&mut self, val: S) -> &mut Self {
        self.rank = val.into();
        self
    }
}

/// Builder for editing the rights of a non-admin user in a specific channel.
///
/// Use [`ClientHandle::set_banned_rights`] to retrieve an instance of this type.
pub struct BannedRightsBuilder {
    client: ClientHandle,
    channel: tl::enums::InputChannel,
    user: tl::enums::InputUser,
    rights: tl::types::ChatBannedRights,
    fut: Option<FutStore>,
}

impl Future for BannedRightsBuilder {
    type Output = FutOutput;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<FutOutput> {
        if self.fut.is_none() {
            let call = tl::functions::channels::EditBanned {
                channel: self.channel.clone(),
                user_id: self.user.clone(),
                banned_rights: tl::enums::ChatBannedRights::Rights(self.rights.clone()),
            };
            let mut c = self.client.clone();
            self.fut = Some(Box::pin(async move { c.invoke(&call).await.map(drop) }));
        }
        Future::poll(self.fut.as_mut().unwrap().as_mut(), cx)
    }
}

impl BannedRightsBuilder {
    pub(crate) fn new(
        client: ClientHandle,
        channel: tl::enums::InputChannel,
        user: tl::enums::InputUser,
    ) -> Self {
        Self {
            client,
            channel,
            user,
            rights: tl::types::ChatBannedRights {
                view_messages: false,
                send_messages: false,
                send_media: false,
                send_stickers: false,
                send_gifs: false,
                send_games: false,
                send_inline: false,
                embed_links: false,
                send_polls: false,
                change_info: false,
                invite_users: false,
                pin_messages: false,
                until_date: 0,
            },
            fut: None,
        }
    }

    /// Load the current rights of the user. This lets you trivially grant or take away specific
    /// permissions without changing any of the previous ones.
    pub async fn load_current(&mut self) -> Result<&mut Self, InvocationError> {
        let tl::enums::channels::ChannelParticipant::Participant(user) = self
            .client
            .invoke(&tl::functions::channels::GetParticipant {
                channel: self.channel.clone(),
                user_id: self.user.clone(),
            })
            .await?;
        match user.participant {
            tl::enums::ChannelParticipant::Banned(u) => {
                self.rights = u.banned_rights.into();
            }
            _ => (),
        }

        Ok(self)
    }

    /// Whether the user is able to view messages or not. Forbidding someone from viewing messages
    /// effectively bans (kicks) them.
    pub fn view_messages(&mut self, val: bool) -> &mut Self {
        // `true` indicates "take away", but in the builder it makes more sense that `false` means
        // "they won't have this permission". All methods perform this negation for that reason.
        self.rights.view_messages = !val;
        self
    }

    /// Whether the user is able to send messages or not. The user will remain in the chat, and
    /// can still read the conversation.
    pub fn send_messages(&mut self, val: bool) -> &mut Self {
        self.rights.send_messages = !val;
        self
    }

    /// Whether the user is able to send any form of media or not, such as photos or voice notes.
    pub fn send_media(&mut self, val: bool) -> &mut Self {
        self.rights.send_media = !val;
        self
    }

    /// Whether the user is able to send stickers or not.
    pub fn send_stickers(&mut self, val: bool) -> &mut Self {
        self.rights.send_stickers = !val;
        self
    }

    /// Whether the user is able to send animated gifs or not.
    pub fn send_gifs(&mut self, val: bool) -> &mut Self {
        self.rights.send_gifs = !val;
        self
    }

    /// Whether the user is able to send games or not.
    pub fn send_games(&mut self, val: bool) -> &mut Self {
        self.rights.send_games = !val;
        self
    }

    /// Whether the user is able to use inline bots or not.
    pub fn send_inline(&mut self, val: bool) -> &mut Self {
        self.rights.send_inline = !val;
        self
    }

    /// Whether the user is able to enable the link preview in the messages they send.
    ///
    /// Note that the user will still be able to send messages with links if this permission is
    /// taken away from the user, but these links won't display a link preview.
    pub fn embed_link_previews(&mut self, val: bool) -> &mut Self {
        self.rights.embed_links = !val;
        self
    }

    /// Whether the user is able to send polls or not.
    pub fn send_polls(&mut self, val: bool) -> &mut Self {
        self.rights.send_polls = !val;
        self
    }

    /// Whether the user is able to change information about the chat such as group description or
    /// not.
    pub fn change_info(&mut self, val: bool) -> &mut Self {
        self.rights.change_info = !val;
        self
    }

    /// Whether the user is able to invite other users or not.
    pub fn invite_users(&mut self, val: bool) -> &mut Self {
        self.rights.invite_users = !val;
        self
    }

    /// Whether the user is able to pin messages or not.
    pub fn pin_messages(&mut self, val: bool) -> &mut Self {
        self.rights.pin_messages = !val;
        self
    }

    /// Apply the restrictions until the given epoch time.
    ///
    /// Note that this is absolute time (i.e current time is not added).
    ///
    /// By default, the restriction is permanent.
    pub fn until(&mut self, val: i32) -> &mut Self {
        // TODO this should take a date, not int
        self.rights.until_date = val;
        self
    }

    /// Apply the restriction for a given duration.
    pub fn duration(&mut self, val: Duration) -> &mut Self {
        // TODO this should account for the server time instead (via sender's offset)
        self.rights.until_date = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is before epoch")
            .as_secs() as i32
            + val.as_secs() as i32;

        self
    }
}
