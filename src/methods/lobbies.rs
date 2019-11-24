use crate::{
    callbacks::{ResultCallback, ResultFromPtrCallback},
    event, iter, sys,
    to_result::ToResult,
    utils::{charbuf_len, charbuf_to_str},
    Discord, Lobby, LobbyMemberTransaction, LobbyTransaction, Reliability, Result, SearchQuery,
};
use std::mem::size_of;

/// # Lobbies
///
/// Provides the ability to group players together and run matchmaking-type searches
/// over the pool of existing groups.
///
/// Some operations must be ran from your game backend:
/// [Reference](https://discordapp.com/developers/docs/game-sdk/lobbies#the-api-way).
///
/// <https://discordapp.com/developers/docs/game-sdk/lobbies>
impl<'a> Discord<'a> {
    /// Create a new lobby. The current user will automatically join and become the owner.
    ///
    /// [`LobbyTransaction::owner`](struct.LobbyTransaction.html#method.owner) *MUST NOT* be called.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#createlobby>
    pub fn create_lobby(
        &mut self,
        transaction: &LobbyTransaction,
        mut callback: impl FnMut(&mut Discord, Result<Lobby>) + 'a,
    ) {
        let mut ptr = std::ptr::null_mut();

        if let Err(e) = unsafe {
            ffi!(self
                .get_lobby_manager()
                .get_lobby_create_transaction(&mut ptr))
            .to_result()
        } {
            return callback(self, Err(e));
        }

        if let Err(e) = unsafe { transaction.process(ptr) } {
            return callback(self, Err(e));
        }

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .create_lobby(ptr)
                .and_then(ResultFromPtrCallback::new(callback)))
        }
    }

    /// Updates a lobby with data from the given transaction.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#updatelobby>
    pub fn update_lobby(
        &mut self,
        lobby_id: i64,
        transaction: &LobbyTransaction,
        mut callback: impl FnMut(&mut Discord, Result<()>) + 'a,
    ) {
        let mut ptr = std::ptr::null_mut();

        if let Err(e) = unsafe {
            ffi!(self
                .get_lobby_manager()
                .get_lobby_update_transaction(lobby_id, &mut ptr))
            .to_result()
        } {
            return callback(self, Err(e));
        }

        if let Err(e) = unsafe { transaction.process(ptr) } {
            return callback(self, Err(e));
        }

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .update_lobby(lobby_id, ptr)
                .and_then(ResultCallback::new(callback)))
        }
    }

    /// Deletes a given lobby.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#deletelobby>
    pub fn delete_lobby(
        &mut self,
        lobby_id: i64,
        callback: impl FnMut(&mut Discord, Result<()>) + 'a,
    ) {
        unsafe {
            ffi!(self
                .get_lobby_manager()
                .delete_lobby(lobby_id)
                .and_then(ResultCallback::new(callback)))
        }
    }

    /// Connects the current user to a given lobby.
    /// You can be connected to up to five lobbies at a time.
    ///
    /// `secret` must not contain any nul bytes, it will grow by one byte.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#connectlobby>
    pub fn connect_lobby(
        &mut self,
        lobby_id: i64,
        mut secret: String,
        callback: impl FnMut(&mut Discord, Result<Lobby>) + 'a,
    ) {
        secret.push('\0');

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .connect_lobby(lobby_id, secret.as_ptr() as *mut _)
                .and_then(ResultFromPtrCallback::new(callback)))
        }
    }

    /// Connects the current user to a lobby using the special activity secret from the lobby
    /// which is a concatenated lobby ID and its secret.
    ///
    /// `activity_secret` must not contain any nul bytes, it will grow by one byte.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#connectlobbywithactivitysecret>
    pub fn connect_lobby_with_activity_secret(
        &mut self,
        mut activity_secret: String,
        callback: impl FnMut(&mut Discord, Result<Lobby>) + 'a,
    ) {
        activity_secret.push('\0');

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .connect_lobby_with_activity_secret(activity_secret.as_ptr() as *mut _)
                .and_then(ResultFromPtrCallback::new(callback)))
        }
    }

    /// Disconnects the current user from a lobby.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#disconnectlobby>
    pub fn disconnect_lobby(
        &mut self,
        lobby_id: i64,
        callback: impl FnMut(&mut Discord, Result<()>) + 'a,
    ) {
        unsafe {
            ffi!(self
                .get_lobby_manager()
                .disconnect_lobby(lobby_id)
                .and_then(ResultCallback::new(callback)))
        }
    }

    /// Gets the lobby object for a given ID.
    ///
    /// A [`lobby_search`](#method.lobby_search) must have completed before hand.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#getlobby>
    pub fn lobby(&mut self, lobby_id: i64) -> Result<Lobby> {
        let mut lobby = sys::DiscordLobby::default();
        unsafe {
            ffi!(self
                .get_lobby_manager()
                .get_lobby(lobby_id, &mut lobby as *mut _))
        }
        .to_result()?;

        Ok(Lobby::from(lobby))
    }

    /// Gets the activity secret for a given lobby.
    /// It should be used to populate
    /// [`Activity::with_join_secret`](struct.Activity.html#method.with_join_secret).
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#getlobbyactivitysecret>
    pub fn lobby_activity_secret(&mut self, lobby_id: i64) -> Result<String> {
        let mut secret: sys::DiscordLobbySecret = [0; size_of::<sys::DiscordLobbySecret>()];

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .get_lobby_activity_secret(lobby_id, &mut secret as *mut _))
        }
        .to_result()?;

        Ok(charbuf_to_str(&secret[..charbuf_len(&secret)]).to_string())
    }

    /// Returns lobby metadata value for a given key.
    ///
    /// `key` must not contain any nul bytes, it will grow by one byte.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#getlobbymetadatavalue>
    pub fn lobby_metadata(&mut self, lobby_id: i64, mut key: String) -> Result<String> {
        let mut value: sys::DiscordMetadataValue = [0; size_of::<sys::DiscordMetadataValue>()];

        key.push('\0');

        unsafe {
            ffi!(self.get_lobby_manager().get_lobby_metadata_value(
                lobby_id,
                key.as_mut_ptr() as *mut _,
                &mut value
            ))
        }
        .to_result()?;

        Ok(charbuf_to_str(&value[..charbuf_len(&value)]).to_string())
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#lobbymetadatacount>  
    pub fn lobby_metadata_count(&mut self, lobby_id: i64) -> Result<i32> {
        let mut count = 0;

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .lobby_metadata_count(lobby_id, &mut count as *mut _))
        }
        .to_result()?;

        Ok(count)
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#getlobbymetadatakey>  
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#getlobbymetadatavalue>
    pub fn lobby_metadata_at(&mut self, lobby_id: i64, index: i32) -> Result<(String, String)> {
        let mut key: sys::DiscordMetadataKey = [0; size_of::<sys::DiscordMetadataKey>()];
        let mut value: sys::DiscordMetadataValue = [0; size_of::<sys::DiscordMetadataValue>()];

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .get_lobby_metadata_key(lobby_id, index as i32, &mut key))
        }
        .to_result()?;

        unsafe {
            ffi!(self.get_lobby_manager().get_lobby_metadata_value(
                lobby_id,
                key.as_mut_ptr(),
                &mut value
            ))
        }
        .to_result()?;

        Ok((
            charbuf_to_str(&key[..charbuf_len(&key)]).to_string(),
            charbuf_to_str(&value[..charbuf_len(&value)]).to_string(),
        ))
    }

    pub fn iter_lobby_metadata(
        &'a mut self,
        lobby_id: i64,
    ) -> Result<
        impl 'a
            + Iterator<Item = Result<(String, String)>>
            + DoubleEndedIterator
            + ExactSizeIterator
            + std::iter::FusedIterator,
    > {
        let count = self.lobby_metadata_count(lobby_id)?;

        Ok(iter::GenericIter::new(
            self,
            move |d, i| d.lobby_metadata_at(lobby_id, i),
            count,
        ))
    }

    /// Updates lobby member info for a given member of the lobby.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#updatemember>
    pub fn update_member(
        &mut self,
        lobby_id: i64,
        user_id: i64,
        transaction: &LobbyMemberTransaction,
        mut callback: impl FnMut(&mut Discord, Result<()>) + 'a,
    ) {
        let mut ptr = std::ptr::null_mut();

        if let Err(e) = unsafe {
            ffi!(self
                .get_lobby_manager()
                .get_member_update_transaction(lobby_id, user_id, &mut ptr))
            .to_result()
        } {
            return callback(self, Err(e));
        }

        if let Err(e) = unsafe { transaction.process(ptr) } {
            return callback(self, Err(e));
        }

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .update_member(lobby_id, user_id, ptr)
                .and_then(ResultCallback::new(callback)))
        }
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#membercount>  
    pub fn lobby_member_count(&mut self, lobby_id: i64) -> Result<i32> {
        let mut count = 0;

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .member_count(lobby_id, &mut count as *mut _))
        }
        .to_result()?;

        Ok(count)
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#getmemberuserid>
    pub fn lobby_member_id_at(&mut self, lobby_id: i64, index: i32) -> Result<i64> {
        let mut user_id = 0;

        unsafe {
            ffi!(self.get_lobby_manager().get_member_user_id(
                lobby_id,
                index,
                &mut user_id as *mut _
            ))
        }
        .to_result()?;

        Ok(user_id)
    }

    pub fn iter_lobby_member_ids(
        &'a mut self,
        lobby_id: i64,
    ) -> Result<
        impl 'a
            + Iterator<Item = Result<i64>>
            + DoubleEndedIterator
            + ExactSizeIterator
            + std::iter::FusedIterator,
    > {
        let count = self.lobby_member_count(lobby_id)?;

        Ok(iter::GenericIter::new(
            self,
            move |d, i| d.lobby_member_id_at(lobby_id, i),
            count,
        ))
    }

    /// Returns member metadata value for a given key.
    ///
    /// `key` must not contain any nul bytes, it will grow by one byte.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#getmembermetadatavalue>
    pub fn lobby_member_metadata(
        &mut self,
        lobby_id: i64,
        user_id: i64,
        mut key: String,
    ) -> Result<String> {
        let mut value: sys::DiscordMetadataValue = [0; size_of::<sys::DiscordMetadataValue>()];

        key.push('\0');

        unsafe {
            ffi!(self.get_lobby_manager().get_member_metadata_value(
                lobby_id,
                user_id,
                key.as_mut_ptr() as *mut _,
                &mut value
            ))
        }
        .to_result()?;

        Ok(charbuf_to_str(&value[..charbuf_len(&value)]).to_string())
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#membermetadatacount>  
    pub fn lobby_member_metadata_count(&mut self, lobby_id: i64, user_id: i64) -> Result<i32> {
        let mut count: i32 = 0;

        unsafe {
            ffi!(self.get_lobby_manager().member_metadata_count(
                lobby_id,
                user_id,
                &mut count as *mut _
            ))
        }
        .to_result()?;

        Ok(count)
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#getmembermetadatakey>
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#getmembermetadatavalue>
    pub fn lobby_member_metadata_at(
        &mut self,
        lobby_id: i64,
        user_id: i64,
        index: i32,
    ) -> Result<(String, String)> {
        let mut key: sys::DiscordMetadataKey = [0; size_of::<sys::DiscordMetadataKey>()];
        let mut value: sys::DiscordMetadataValue = [0; size_of::<sys::DiscordMetadataValue>()];

        unsafe {
            ffi!(self.get_lobby_manager().get_member_metadata_key(
                lobby_id,
                user_id,
                index as i32,
                &mut key as *mut _
            ))
        }
        .to_result()?;

        unsafe {
            ffi!(self.get_lobby_manager().get_member_metadata_value(
                lobby_id,
                user_id,
                key.as_mut_ptr(),
                &mut value as *mut _
            ))
        }
        .to_result()?;

        Ok((
            charbuf_to_str(&key[..charbuf_len(&key)]).to_string(),
            charbuf_to_str(&value[..charbuf_len(&value)]).to_string(),
        ))
    }

    pub fn iter_lobby_member_metadata(
        &'a mut self,
        lobby_id: i64,
        user_id: i64,
    ) -> Result<
        impl 'a
            + Iterator<Item = Result<(String, String)>>
            + DoubleEndedIterator
            + ExactSizeIterator
            + std::iter::FusedIterator,
    > {
        let count = self.lobby_member_metadata_count(lobby_id, user_id)?;

        Ok(iter::GenericIter::new(
            self,
            move |d, i| d.lobby_member_metadata_at(lobby_id, user_id, i),
            count,
        ))
    }

    /// Sends a message to the lobby on behalf of the current user.
    /// You must be connected to the lobby you are messaging.
    /// You should use this function for message sending if you are not using
    /// the built in networking layer for the lobby.
    /// If you are, you should use SendNetworkMessage instead.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#sendlobbymessage>
    pub fn send_lobby_message(
        &mut self,
        lobby_id: i64,
        buf: impl AsRef<[u8]>,
        callback: impl FnMut(&mut Discord, Result<()>) + 'a,
    ) {
        let buf = buf.as_ref();
        assert!(buf.len() <= u32::max_value() as usize);

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .send_lobby_message(lobby_id, buf.as_ptr() as *mut _, buf.len() as u32)
                .and_then(ResultCallback::new(callback)))
        }
    }

    /// Searches available lobbies based on the search criteria.
    /// Lobbies that meet the criteria are then globally filtered.
    /// The callback fires when the list of lobbies is stable and ready for iteration.
    /// You do not necessarily need to access the filtered lobbies within the context of the result callback.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#search>
    pub fn lobby_search(
        &mut self,
        search: &SearchQuery,
        mut callback: impl FnMut(&mut Discord, Result<()>) + 'a,
    ) {
        let mut ptr = std::ptr::null_mut();

        if let Err(e) =
            unsafe { ffi!(self.get_lobby_manager().get_search_query(&mut ptr)).to_result() }
        {
            return callback(self, Err(e));
        }

        if let Err(e) = unsafe { search.process(ptr) } {
            return callback(self, Err(e));
        }

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .search(ptr)
                .and_then(ResultCallback::new(callback)))
        }
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#lobbycount>
    pub fn lobby_count(&mut self) -> i32 {
        let mut count = 0;

        unsafe { ffi!(self.get_lobby_manager().lobby_count(&mut count)) }

        count
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#getlobbyid>
    pub fn lobby_id_at(&mut self, index: i32) -> Result<i64> {
        let mut lobby_id = 0;

        unsafe { ffi!(self.get_lobby_manager().get_lobby_id(index, &mut lobby_id)) }.to_result()?;

        Ok(lobby_id)
    }

    pub fn iter_lobbies(
        &'a mut self,
    ) -> impl 'a
           + Iterator<Item = Result<i64>>
           + DoubleEndedIterator
           + ExactSizeIterator
           + std::iter::FusedIterator {
        let count = self.lobby_count();

        iter::GenericIter::new(self, |d, i| d.lobby_id_at(i), count)
    }

    /// Connects to the voice channel of the current lobby.
    /// When connected to voice, the user can open their Discord overlay to see a list of other users,
    /// allowing them to mute/deafen themselves as well as mute/adjust the volume of other members.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#connectvoice>
    pub fn connect_lobby_voice(
        &mut self,
        lobby_id: i64,
        callback: impl FnMut(&mut Discord, Result<()>) + 'a,
    ) {
        unsafe {
            ffi!(self
                .get_lobby_manager()
                .connect_voice(lobby_id)
                .and_then(ResultCallback::new(callback)))
        }
    }

    /// Disconnects from the voice channel of a given lobby.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#disconnectvoice>
    pub fn disconnect_lobby_voice(
        &mut self,
        lobby_id: i64,
        callback: impl FnMut(&mut Discord, Result<()>) + 'a,
    ) {
        unsafe {
            ffi!(self
                .get_lobby_manager()
                .disconnect_voice(lobby_id)
                .and_then(ResultCallback::new(callback)))
        }
    }

    /// Connects to the networking layer for the given lobby ID.
    /// Call this when connecting to the lobby.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#connectnetwork>
    pub fn connect_lobby_network(&mut self, lobby_id: i64) -> Result<()> {
        unsafe { ffi!(self.get_lobby_manager().connect_network(lobby_id,)) }.to_result()
    }

    /// Disconnects from the networking layer for the given lobby ID.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#disconnectnetwork>
    pub fn disconnect_lobby_network(&mut self, lobby_id: i64) -> Result<()> {
        unsafe { ffi!(self.get_lobby_manager().disconnect_network(lobby_id,)) }.to_result()
    }

    /// Flushes the network. Call this when you're done sending messages.
    /// This should appear near the end of your game loop.
    ///
    /// https://discordapp.com/developers/docs/game-sdk/lobbies#flushnetwork
    pub fn flush_lobby_network(&mut self) -> Result<()> {
        unsafe { ffi!(self.get_lobby_manager().flush_network()) }.to_result()
    }

    /// Opens a network channel to all users in a lobby on the given channel number.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#opennetworkchannel>
    pub fn open_lobby_network_channel(
        &mut self,
        lobby_id: i64,
        channel_id: u8,
        reliable: Reliability,
    ) -> Result<()> {
        unsafe {
            ffi!(self.get_lobby_manager().open_network_channel(
                lobby_id,
                channel_id,
                reliable.into()
            ))
        }
        .to_result()
    }

    /// Sends a network message.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#sendnetworkmessage>
    pub fn send_lobby_network_message(
        &mut self,
        lobby_id: i64,
        user_id: i64,
        channel_id: u8,
        buf: &[u8],
    ) -> Result<()> {
        assert!(buf.len() <= u32::max_value() as usize);

        unsafe {
            ffi!(self.get_lobby_manager().send_network_message(
                lobby_id,
                user_id,
                channel_id,
                buf.as_ptr() as *mut _,
                buf.len() as u32
            ))
        }
        .to_result()
    }

    /// Fires when a lobby is updated.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#onlobbyupdate>
    pub fn recv_lobbies_update(&self) -> impl '_ + Iterator<Item = event::lobbies::Update> {
        self.receivers.lobbies_update.try_iter()
    }

    /// Fired when a lobby is deleted.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#onlobbydelete>
    pub fn recv_lobbies_delete(&self) -> impl '_ + Iterator<Item = event::lobbies::Delete> {
        self.receivers.lobbies_delete.try_iter()
    }

    /// Fires when a new member joins the lobby.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#onmemberconnect>
    pub fn recv_lobbies_member_connect(
        &self,
    ) -> impl '_ + Iterator<Item = event::lobbies::MemberConnect> {
        self.receivers.lobbies_member_connect.try_iter()
    }

    /// Fires when data for a lobby member is updated.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#onmemberupdate>
    pub fn recv_lobbies_member_update(
        &self,
    ) -> impl '_ + Iterator<Item = event::lobbies::MemberUpdate> {
        self.receivers.lobbies_member_update.try_iter()
    }

    /// Fires when a member leaves the lobby.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#onmemberdisconnect>
    pub fn recv_lobbies_member_disconnect(
        &self,
    ) -> impl '_ + Iterator<Item = event::lobbies::MemberDisconnect> {
        self.receivers.lobbies_member_disconnect.try_iter()
    }

    /// Fires when a message is sent to the lobby.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#onlobbymessage>
    pub fn recv_lobbies_message(&self) -> impl '_ + Iterator<Item = event::lobbies::Message> {
        self.receivers.lobbies_message.try_iter()
    }

    /// Fires when a user connected to voice starts or stops speaking.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#onspeaking>
    pub fn recv_lobbies_speaking(&self) -> impl '_ + Iterator<Item = event::lobbies::Speaking> {
        self.receivers.lobbies_speaking.try_iter()
    }

    /// Fires when the user receives a message from the lobby's networking layer.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#onnetworkmessage>
    pub fn recv_lobbies_network_message(
        &self,
    ) -> impl '_ + Iterator<Item = event::lobbies::NetworkMessage> {
        self.receivers.lobbies_network_message.try_iter()
    }
}
