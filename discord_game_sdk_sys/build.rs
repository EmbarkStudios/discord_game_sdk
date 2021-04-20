fn main() {
    println!("cargo:rustc-link-lib=discord_game_sdk");
    println!(
        "cargo:rustc-link-search={}/sdk/lib/x86_64",
        std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir not set")
    );
}
