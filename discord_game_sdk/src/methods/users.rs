use crate::{sys, to_result::ToResult, Discord, PremiumKind, Result, User, UserFlags, UserID};

/// # Users
///
/// > [Chapter in official docs](https://discordapp.com/developers/docs/game-sdk/users)
impl<'d, E> Discord<'d, E> {
    /// Get the current user.
    ///
    /// More information can be found through the HTTP API.
    ///
    /// ## Errors
    ///
    /// Returns [`Error::NotFound`](enum.Error.html#variant.NotFound) until the event
    /// [`EventHandler::on_current_user_update`](trait.EventHandler.html#method.on_current_user_update)
    /// has fired.
    ///
    /// > [Method in official docs](https://discordapp.com/developers/docs/game-sdk/users#getcurrentuser)
    ///
    /// ```rust
    /// # use discord_game_sdk::*;
    /// # fn example(discord: Discord<'_, ()>) -> Result<()> {
    /// let current_user = discord.current_user()?;
    /// # Ok(()) }
    /// ```
    pub fn current_user(&self) -> Result<User> {
        let mut user = User(sys::DiscordUser::default());

        unsafe {
            let mgr = self.user_manager();

            (*mgr).get_current_user.unwrap()(mgr, &mut user.0).into_result()?;
        }

        Ok(user)
    }

    /// Get user information for a given ID.
    ///
    /// > [Method in official docs](https://discordapp.com/developers/docs/game-sdk/users#getuser)
    ///
    /// ```rust
    /// # use discord_game_sdk::*;
    /// # fn example(discord: Discord<'_, ()>) -> Result<()> {
    /// # let id_to_lookup = 0;
    /// discord.user(id_to_lookup, |discord, result| {
    ///     match result {
    ///         Ok(user) => {
    ///             // ...
    ///         }
    ///         Err(error) => eprintln!("failed to fetch user: {}", error),
    ///     }
    /// });
    /// # Ok(()) }
    /// ```
    pub fn user(
        &self,
        user_id: UserID,
        callback: impl 'd + FnOnce(&Discord<'d, E>, Result<&User>),
    ) {
        let (ptr, fun) = self.two_params(
            move |discord, res: sys::EDiscordResult, user: *mut sys::DiscordUser| {
                callback(
                    discord,
                    res.into_result().map(|()| unsafe { &*(user as *mut User) }),
                )
            },
        );

        unsafe {
            let mgr = self.user_manager();

            (*mgr).get_user.unwrap()(mgr, user_id, ptr, fun)
        }
    }

    /// Get the Premium type for the currently connected user.
    ///
    /// > [Method in official docs](https://discordapp.com/developers/docs/game-sdk/users#getcurrentuserpremiumtype)
    ///
    /// ```rust
    /// # use discord_game_sdk::*;
    /// # fn example(discord: Discord<'_, ()>) -> Result<()> {
    /// let premium = discord.current_user_premium_kind()?;
    /// # Ok(()) }
    /// ```
    pub fn current_user_premium_kind(&self) -> Result<PremiumKind> {
        let mut premium_type = sys::EDiscordPremiumType::default();

        unsafe {
            let mgr = self.user_manager();

            (*mgr).get_current_user_premium_type.unwrap()(mgr, &mut premium_type).into_result()?;
        }

        Ok(PremiumKind::from(premium_type))
    }

    /// Return a bitfield of all flags set for the current user.
    ///
    /// > [Method in official docs](https://discordapp.com/developers/docs/game-sdk/users#currentuserhasflag)
    ///
    /// ```rust
    /// # use discord_game_sdk::*;
    /// # fn example(discord: Discord<'_, ()>) -> Result<()> {
    /// let flags = discord.current_user_flags()?;
    /// # Ok(()) }
    /// ```
    pub fn current_user_flags(&self) -> Result<UserFlags> {
        let mut flags = UserFlags::empty();

        for flag in &[
            UserFlags::PARTNER,
            UserFlags::HYPE_SQUAD_EVENTS,
            UserFlags::HYPE_SQUAD_HOUSE_1,
            UserFlags::HYPE_SQUAD_HOUSE_2,
            UserFlags::HYPE_SQUAD_HOUSE_3,
        ] {
            let mut contains = false;

            unsafe {
                let mgr = self.user_manager();

                (*mgr).current_user_has_flag.unwrap()(mgr, flag.bits(), &mut contains)
                    .into_result()?;
            }

            flags.set(*flag, contains);
        }

        Ok(flags)
    }
}
