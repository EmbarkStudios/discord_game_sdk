use crate::{
    callbacks::ResultCallback, event, sys, to_result::ToResult, Achievement, Discord, Result,
};

/// # Achievements
///
/// <https://discordapp.com/developers/docs/game-sdk/achievements>
impl<'a> Discord<'a> {
    /// `percent_complete` must be [0..=100]
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/achievements#setuserachievement>
    pub fn set_achievement(
        &mut self,
        achievement_id: i64,
        percent_complete: u8,
        callback: impl FnMut(&mut Discord, Result<()>) + 'a,
    ) {
        unsafe {
            ffi!(self
                .get_achievement_manager()
                .set_user_achievement(achievement_id, percent_complete)
                .and_then(ResultCallback::new(callback)))
        }
    }

    /// <https://discordapp.com/developers/docs/game-sdk/achievements#fetchuserachievements>
    pub fn fetch_achievements(&mut self, callback: impl FnMut(&mut Discord, Result<()>) + 'a) {
        unsafe {
            ffi!(self
                .get_achievement_manager()
                .fetch_user_achievements()
                .and_then(ResultCallback::new(callback)))
        }
    }

    /// <https://discordapp.com/developers/docs/game-sdk/achievements#getuserachievement>
    pub fn achievement(&mut self, achievement_id: i64) -> Result<Achievement> {
        let mut achievement = sys::DiscordUserAchievement::default();

        unsafe {
            ffi!(self
                .get_achievement_manager()
                .get_user_achievement(achievement_id, &mut achievement))
        }
        .to_result()?;

        Ok(achievement.into())
    }

    /// <https://discordapp.com/developers/docs/game-sdk/achievements#countuserachievements>  
    /// <https://discordapp.com/developers/docs/game-sdk/achievements#getuserachievementat>
    pub fn all_achievements(&mut self) -> Result<Vec<Achievement>> {
        let mut count: i32 = 0;

        unsafe {
            ffi!(self
                .get_achievement_manager()
                .count_user_achievements(&mut count))
        }

        let mut result = Vec::with_capacity(count as usize);
        let mut achievement = sys::DiscordUserAchievement::default();

        for index in 0..count {
            unsafe {
                ffi!(self
                    .get_achievement_manager()
                    .get_user_achievement_at(index, &mut achievement))
            }
            .to_result()?;

            result.push(achievement.into());
        }

        Ok(result)
    }

    /// <https://discordapp.com/developers/docs/game-sdk/achievements#onuserachievementupdate>
    pub fn recv_achievements_update(
        &'_ self,
    ) -> impl '_ + Iterator<Item = event::achievements::Update> {
        self.receivers.achievements_update.try_iter()
    }
}
