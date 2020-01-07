use crate::{
    discord::{Discord, DiscordInner},
    event_handler::VoidEvents,
    sys,
    to_result::ToResult,
    utils::charptr_to_str,
    ClientID, CreateFlags, Result,
};
use std::{convert::TryFrom, ops::DerefMut};

/// # Core
///
/// > [Chapter in official docs](https://discordapp.com/developers/docs/game-sdk/discord)
///
/// ```rust
/// # fn example() {
/// use discord_game_sdk::Discord;
///
/// # const DISCORD_CLIENT_ID: ClientID = 0;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut discord = Discord::new(DISCORD_CLIENT_ID)?;
///
///     loop {
///         discord.run_callbacks()?;
///     }
///
///     Ok(())
/// }
/// # }
/// ```
impl Discord {
    /// Calls [`with_create_flags`] with [`CreateFlags::Default`].
    ///
    /// [`with_create_flags`]: #method.with_create_flags
    /// [`CreateFlags::Default`]: enum.CreateFlags.html#variant.Default
    pub fn new(client_id: ClientID) -> Result<Self> {
        Self::with_create_flags(client_id, CreateFlags::Default)
    }

    /// Creates an instance of the main interface with the Discord Game SDK.
    ///
    /// > [`Create` in official docs](https://discordapp.com/developers/docs/game-sdk/discord#create)  
    pub fn with_create_flags(client_id: ClientID, flags: CreateFlags) -> Result<Self> {
        let mut inner = Box::new(DiscordInner {
            core: std::ptr::null_mut(),
            client_id,
            event_handler: Box::new(VoidEvents),
        });

        let ptr = inner.deref_mut() as *mut DiscordInner as *mut std::ffi::c_void;

        let mut params = create_params(client_id, flags.into(), ptr);

        unsafe {
            sys::DiscordCreate(
                sys::DISCORD_VERSION,
                // XXX: *mut should be *const
                &mut params,
                // XXX: *mut *mut should be *mut *const
                &mut inner.core,
            )
        }
        .to_result()?;

        log::trace!("received pointer to {:p}", inner.core);

        let instance = Discord(inner);

        #[allow(unused_results)]
        unsafe {
            ffi!(instance.set_log_hook(
                sys::DiscordLogLevel_Debug,
                ptr,
                event_handler!(|level: sys::EDiscordLogLevel, message: *const u8| {
                    EventHandler::on_log_message(level.into(), charptr_to_str(message))
                })
            ));

            // Signal managers that we want events ASAP
            ffi!(instance.get_network_manager());
            ffi!(instance.get_overlay_manager());
            ffi!(instance.get_relationship_manager());
            ffi!(instance.get_user_manager());

            ffi!(instance.get_achievement_manager());
            ffi!(instance.get_activity_manager());
            ffi!(instance.get_lobby_manager());
            ffi!(instance.get_store_manager());
            ffi!(instance.get_voice_manager());
        }

        Ok(instance)
    }

    /// Runs all pending SDK callbacks.
    ///
    /// This should be called often, like in the main loop if you're writing a game.
    ///
    /// ## Errors
    ///
    /// If the Discord client was closed, [`Error::NotRunning`] will be returned.
    ///
    /// > [Method in official docs](https://discordapp.com/developers/docs/game-sdk/discord#runcallbacks)
    ///
    /// [emptied]: #method.empty_event_buffers
    /// [`Error::NotRunning`]: enum.Error.html#variant.NotRunning
    pub fn run_callbacks(&mut self) -> Result<()> {
        unsafe { ffi!(self.run_callbacks()) }.to_result()
    }
}

fn create_params(
    client_id: sys::DiscordClientId,
    flags: sys::EDiscordCreateFlags,
    event_data: *mut std::ffi::c_void,
) -> sys::DiscordCreateParams {
    use crate::across_ffi::event_handlers::*;

    sys::DiscordCreateParams {
        client_id,
        // XXX: u64 should be sys::EDiscordCreateFlags
        flags: u64::try_from(flags).unwrap(),

        // XXX: *mut should be *const
        events: std::ptr::null_mut(),
        event_data,

        // XXX: *mut should be *const
        application_events: std::ptr::null_mut(),
        application_version: sys::DISCORD_APPLICATION_MANAGER_VERSION,

        // XXX: *mut should be *const
        user_events: USER as *const _ as *mut _,
        user_version: sys::DISCORD_USER_MANAGER_VERSION,

        // XXX: *mut should be *const
        image_events: std::ptr::null_mut(),
        image_version: sys::DISCORD_IMAGE_MANAGER_VERSION,

        // XXX: *mut should be *const
        activity_events: ACTIVITY as *const _ as *mut _,
        activity_version: sys::DISCORD_ACTIVITY_MANAGER_VERSION,

        // XXX: *mut should be *const
        relationship_events: RELATIONSHIP as *const _ as *mut _,
        relationship_version: sys::DISCORD_RELATIONSHIP_MANAGER_VERSION,

        // XXX: *mut should be *const
        lobby_events: LOBBY as *const _ as *mut _,
        lobby_version: sys::DISCORD_LOBBY_MANAGER_VERSION,

        // XXX: *mut should be *const
        network_events: NETWORK as *const _ as *mut _,
        network_version: sys::DISCORD_NETWORK_MANAGER_VERSION,

        // XXX: *mut should be *const
        overlay_events: OVERLAY as *const _ as *mut _,
        overlay_version: sys::DISCORD_OVERLAY_MANAGER_VERSION,

        // XXX: *mut should be *const
        storage_events: std::ptr::null_mut(),
        storage_version: sys::DISCORD_STORAGE_MANAGER_VERSION,

        // XXX: *mut should be *const
        store_events: STORE as *const _ as *mut _,
        store_version: sys::DISCORD_STORE_MANAGER_VERSION,

        // XXX: *mut should be *const
        voice_events: VOICE as *const _ as *mut _,
        voice_version: sys::DISCORD_VOICE_MANAGER_VERSION,

        // XXX: *mut should be *const
        achievement_events: ACHIEVEMENT as *const _ as *mut _,
        achievement_version: sys::DISCORD_ACHIEVEMENT_MANAGER_VERSION,
    }
}
