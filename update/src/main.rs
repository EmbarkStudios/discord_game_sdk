fn main() {
    let version = std::env::args()
        .nth(1)
        .expect("must specify an SDK version");

    let sdk_zip = reqwest::blocking::get(&format!(
        "https://dl-game-sdk.discordapp.net/{}/discord_game_sdk.zip",
        version
    ))
    .expect("failed to retrieve SDK")
    .bytes()
    .expect("failed to retrieve SDK body");

    let _ = std::fs::remove_dir_all("discord_game_sdk_sys/sdk");

    let sdk_bytes = &sdk_zip[..];

    {
        let mut archive = zip::read::ZipArchive::new(std::io::Cursor::new(sdk_bytes))
            .expect("failed to read zip");
        archive
            .extract("discord_game_sdk_sys/sdk")
            .expect("failed to extract SDK from zip");
    }

    let _ = std::fs::remove_dir_all("discord_game_sdk_sys/sdk/examples");
    let _ = std::fs::remove_dir_all("discord_game_sdk_sys/sdk/cpp");
    let _ = std::fs::remove_dir_all("discord_game_sdk_sys/sdk/csharp");
    let _ = std::fs::remove_dir_all("discord_game_sdk_sys/sdk/lib/x86");
    let _ = std::fs::remove_file("discord_game_sdk_sys/sdk/lib/x86_64/discord_game_sdk.bundle");

    std::fs::rename(
        "discord_game_sdk_sys/sdk/lib/x86_64/discord_game_sdk.dylib",
        "discord_game_sdk_sys/sdk/lib/x86_64/libdiscord_game_sdk.dylib",
    )
    .expect("failed to rename dylib");
    std::fs::rename(
        "discord_game_sdk_sys/sdk/lib/x86_64/discord_game_sdk.so",
        "discord_game_sdk_sys/sdk/lib/x86_64/libdiscord_game_sdk.so",
    )
    .expect("failed to rename so");

    generate_ffi_bindings();
}

fn generate_ffi_bindings() {
    #[derive(Debug)]
    struct Callbacks;

    impl bindgen::callbacks::ParseCallbacks for Callbacks {
        fn int_macro(&self, name: &str, _value: i64) -> Option<bindgen::callbacks::IntKind> {
            // Must match sys::DiscordVersion
            if name.ends_with("_VERSION") {
                Some(bindgen::callbacks::IntKind::I32)
            } else {
                None
            }
        }
    }

    bindgen::builder()
        .header("discord_game_sdk_sys/sdk/c/discord_game_sdk.h")
        .ctypes_prefix("crate::ctypes")
        .derive_copy(true)
        .derive_debug(true)
        .derive_default(true)
        .derive_eq(true)
        .derive_hash(true)
        .derive_partialeq(true)
        .generate_comments(false)
        .impl_debug(true)
        .impl_partialeq(true)
        .parse_callbacks(Box::new(Callbacks))
        .prepend_enum_name(false)
        .allowlist_function("Discord.+")
        .allowlist_type("[EI]?Discord.+")
        .allowlist_var("DISCORD_.+")
        .generate()
        .expect("discord_game_sdk_sys: bindgen could not generate bindings")
        .write_to_file("discord_game_sdk_sys/src/bindings.rs")
        .expect("discord_game_sdk_sys: could not write bindings to file");
}
