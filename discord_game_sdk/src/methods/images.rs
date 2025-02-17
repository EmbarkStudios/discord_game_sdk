use crate::{sys, to_result::ToResult, Discord, FetchKind, Image, ImageHandle, Result};
use std::convert::{TryFrom, TryInto};

/// # Images
///
/// > [Chapter in official docs](https://discordapp.com/developers/docs/game-sdk/images)
///
/// ```rust
/// # use discord_game_sdk::*;
/// # fn example(discord: Discord<'_, ()>, user: User) -> Result<()> {
/// discord.fetch_image(
///     ImageHandle::from_user_id(user.id(), 128),
///     FetchKind::UseCached,
///     |discord, handle| {
///         match handle.and_then(|handle| discord.image(handle))  {
///             Ok(image) => {
///                 println!("image dimensions: {:?}", image.dimensions());
///                 // ...
///             },
///             Err(error) => eprintln!("failed to fetch image: {}", error),
///         }
///     },
/// );
/// # Ok(()) }
/// ```
impl<'d, E> Discord<'d, E> {
    /// Prepares an image.
    ///
    /// > [Method in official docs](https://discordapp.com/developers/docs/game-sdk/images#fetch)
    pub fn fetch_image(
        &self,
        handle: ImageHandle,
        refresh: FetchKind,
        callback: impl 'd + FnOnce(&Discord<'d, E>, Result<ImageHandle>),
    ) {
        let (ptr, fun) = self.two_params(
            move |discord, res: sys::EDiscordResult, image_handle: sys::DiscordImageHandle| {
                callback(
                    discord,
                    res.into_result().map(|()| ImageHandle(image_handle)),
                )
            },
        );

        unsafe {
            let mgr = self.image_manager();

            (*mgr).fetch.unwrap()(mgr, handle.0, refresh.into(), ptr, fun)
        }
    }

    /// Get's the dimensions of the source image.
    ///
    /// > [Method in official docs](https://discordapp.com/developers/docs/game-sdk/images#getdimensions)
    pub fn image_dimensions(&self, handle: ImageHandle) -> Result<(u32, u32)> {
        let mut dimensions = sys::DiscordImageDimensions::default();

        unsafe {
            let mgr = self.image_manager();

            (*mgr).get_dimensions.unwrap()(mgr, handle.0, &mut dimensions).into_result()?;
        }

        Ok((dimensions.width, dimensions.height))
    }

    /// Retrieves the data for an image.
    ///
    /// The image must be [fetched](#method.fetch_image) first.
    ///
    /// > [Method in official docs](https://discordapp.com/developers/docs/game-sdk/images#getdata)
    pub fn image(&self, handle: ImageHandle) -> Result<Image> {
        let (width, height) = self.image_dimensions(handle.clone())?;
        let mut data = vec![0; 4 * width as usize * height as usize];

        debug_assert!(u32::try_from(data.len()).is_ok());

        unsafe {
            let mgr = self.image_manager();

            (*mgr).get_data.unwrap()(
                mgr,
                handle.0,
                data.as_mut_ptr(),
                data.len().try_into().unwrap_or(u32::max_value()),
            )
            .into_result()?;
        }

        Ok(Image {
            width,
            height,
            data,
        })
    }
}
