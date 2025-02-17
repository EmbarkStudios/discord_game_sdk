use crate::{sys, to_result::ToResult, Action, Discord, Result};
use std::borrow::Cow;

/// # Overlay
///
/// The terminology employed by the Game SDK is confusing, this crate employs the terms "opened"
/// and "closed" instead:
///
/// |                                          | Game SDK | `discord_game_sdk` |
/// |------------------------------------------|----------|--------------------|
/// | Overlay is appearing and has taken focus | unlocked | opened             |
/// | Overlay is hidden                        | locked   | closed             |
///
/// > [Chapter in official docs](https://discordapp.com/developers/docs/game-sdk/overlay)
impl<'d, E> Discord<'d, E> {
    /// Check whether the user has the overlay enabled or disabled.
    ///
    /// If the overlay is disabled, all the functionality in this manager will still work.
    /// The calls will instead focus the Discord client and show the modal there instead.
    ///
    /// > [Method in official docs](https://discordapp.com/developers/docs/game-sdk/overlay#isenabled)
    ///
    /// ```rust
    /// # use discord_game_sdk::*;
    /// # fn example(discord: Discord<'_, ()>) -> Result<()> {
    /// if discord.overlay_enabled() {
    ///     // ...
    /// }
    /// # Ok(()) }
    /// ```
    pub fn overlay_enabled(&self) -> bool {
        let mut enabled = false;

        unsafe {
            let mgr = self.overlay_manager();

            (*mgr).is_enabled.unwrap()(mgr, &mut enabled)
        }

        enabled
    }

    /// Whether the overlay is appearing and has taken focus.
    ///
    /// > [Method in official docs](https://discordapp.com/developers/docs/game-sdk/overlay#islocked)
    ///
    /// ```rust
    /// # use discord_game_sdk::*;
    /// # fn example(discord: Discord<'_, ()>) -> Result<()> {
    /// if discord.overlay_opened() {
    ///     // ...
    /// }
    /// # Ok(()) }
    /// ```
    pub fn overlay_opened(&self) -> bool {
        let mut locked = false;

        unsafe {
            let mgr = self.overlay_manager();

            (*mgr).is_locked.unwrap()(mgr, &mut locked)
        }

        !locked
    }

    /// Open or close the overlay.
    ///
    /// > [Method in official docs](https://discordapp.com/developers/docs/game-sdk/overlay#setlocked)
    ///
    /// ```rust
    /// # use discord_game_sdk::*;
    /// # fn example(discord: Discord<'_, ()>) -> Result<()> {
    /// discord.set_overlay_opened(false, |discord, result| {
    ///     if let Err(error) = result {
    ///         return eprintln!("failed to set overlay open: {}", error);
    ///     }
    /// });
    /// # Ok(()) }
    /// ```
    pub fn set_overlay_opened(
        &self,
        opened: bool,
        callback: impl 'd + FnOnce(&Discord<'d, E>, Result<()>),
    ) {
        let (ptr, fun) = self.one_param(move |discord, res: sys::EDiscordResult| {
            callback(discord, res.into_result())
        });

        unsafe {
            let mgr = self.overlay_manager();

            (*mgr).set_locked.unwrap()(mgr, !opened, ptr, fun)
        }
    }

    /// Opens the overlay modal for sending game invitations to users, channels, and servers.
    /// If you do not have a valid activity with all the required fields, this call will error.
    ///
    /// > [Method in official docs](https://discordapp.com/developers/docs/game-sdk/overlay#openactivityinvite)
    ///
    /// ```rust
    /// # use discord_game_sdk::*;
    /// # fn example(discord: Discord<'_, ()>) -> Result<()> {
    /// discord.open_invite_overlay(Action::Join, |discord, result| {
    ///     if let Err(error) = result {
    ///         return eprintln!("failed open invite overlay: {}", error);
    ///     }
    /// });
    /// # Ok(()) }
    /// ```
    pub fn open_invite_overlay(
        &self,
        action: Action,
        callback: impl 'd + FnOnce(&Discord<'d, E>, Result<()>),
    ) {
        let (ptr, fun) = self.one_param(move |discord, res: sys::EDiscordResult| {
            callback(discord, res.into_result())
        });

        unsafe {
            let mgr = self.overlay_manager();

            (*mgr).open_activity_invite.unwrap()(mgr, action.into(), ptr, fun)
        }
    }

    /// Opens the overlay modal for joining a Discord guild, given its invite code
    /// (e.g.: `ABCDEF` in `https://discord.gg/ABCDEF` or `https://discordapp.com/invite/ABCDEF`).
    ///
    /// Receiving `Ok(())` does not necessarily mean that the user has joined the guild.
    ///
    /// ## Performance
    ///
    /// A nul byte will be appended to `code` if one is not present.
    ///
    /// > [Method in official docs](https://discordapp.com/developers/docs/game-sdk/overlay#openguildinvite)
    ///
    /// ```rust
    /// # use discord_game_sdk::*;
    /// # fn example(discord: Discord<'_, ()>) -> Result<()> {
    /// discord.open_guild_invite_overlay("discord-gamesdk\0", |discord, result| {
    ///     if let Err(error) = result {
    ///         return eprintln!("failed open guild invite overlay: {}", error);
    ///     }
    /// });
    /// # Ok(()) }
    /// ```
    pub fn open_guild_invite_overlay<'s>(
        &self,
        code: impl Into<Cow<'s, str>>,
        callback: impl 'd + FnOnce(&Discord<'d, E>, Result<()>),
    ) {
        let mut code = code.into();

        if !code.ends_with('\0') {
            code.to_mut().push('\0')
        }

        let (ptr, fun) = self.one_param(move |discord, res: sys::EDiscordResult| {
            callback(discord, res.into_result())
        });

        unsafe {
            let mgr = self.overlay_manager();

            (*mgr).open_guild_invite.unwrap()(mgr, code.as_ptr(), ptr, fun)
        }
    }

    /// Opens the overlay widget for voice settings for the currently connected application.
    /// These settings are unique to each user within the context of your application.
    /// That means that a user can have different favorite voice settings for each of their games.
    ///
    /// > [Method in official docs](https://discordapp.com/developers/docs/game-sdk/overlay#openvoicesettings)
    ///
    /// ```rust
    /// # use discord_game_sdk::*;
    /// # fn example(discord: Discord<'_, ()>) -> Result<()> {
    /// discord.open_voice_settings(|discord, result| {
    ///     if let Err(error) = result {
    ///         return eprintln!("failed open voice settings overlay: {}", error);
    ///     }
    /// });
    /// # Ok(()) }
    /// ```
    pub fn open_voice_settings(&self, callback: impl 'd + FnOnce(&Discord<'d, E>, Result<()>)) {
        let (ptr, fun) = self.one_param(move |discord, res: sys::EDiscordResult| {
            callback(discord, res.into_result())
        });

        unsafe {
            let mgr = self.overlay_manager();

            (*mgr).open_voice_settings.unwrap()(mgr, ptr, fun)
        }
    }
}
