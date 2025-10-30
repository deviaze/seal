# Install

## Windows

1. Download the [latest release](<https://github.com/deviaze/seal/releases/latest>) of *seal* for your system, or if you want the most up-to-date beta, download the newest [build artifact from GitHub Actions](https://github.com/deviaze/seal/actions).
2. Move *seal* somewhere you typically keep your locally-installed programs; I recommend `C:\Users\<USERNAME>\.local\bin\seal.exe`. To add this directory to your PowerShell profile `$PATH`, open your PowerShell profile in VSCode by running `code $PROFILE` and then add `$env:Path += ";C:\Users\<USERNAME>\.local\bin"` somewhere near the bottom of the file.
3. Close and reopen your Windows Terminal and make sure running `seal --help` displays *seal*'s help message. This should ensure *seal* is available in your `$PATH`.
4. Webview dependencies (for GUI programs, when implemented) should already be installed via Microsoft Edge.

## macOS

1. Download either the [latest release](<https://github.com/deviaze/seal/releases/latest>) of *seal*, a recent [build artifact](https://github.com/deviaze/seal/actions), or if you have Rust installed, compile *seal* locally by cloning this repository and running `cargo build --release`.
2. Move *seal* to a location like `/usr/local/bin/seal` or `~/bin/seal`.
3. Because *seal* is not signed/notarized, macOS will block it from running by default. To allow it, first run `./seal --help` to cause macOS to show a warning, and then go to Mac **System Settings → Privacy & Security** and check the bottom for a message like "seal was blocked from use because it is not from an identified developer." Click **Allow Anyway**.
4. Go back to your terminal and try `./seal --help` again. This may cause another warning to pop up with another confirmation dialog. Click **Open**.
5. To ensure *seal* is available everywhere, make sure it's added to your shell's `$PATH`. For example, if you placed it in `~/bin`, add `export PATH="$HOME/bin:$PATH"` to your shell config (.zshrc, .bash_profile, etc.)
6. Every time you update, redownload, or recompile *seal* you might have to redo those steps and explicitly allow it again. You can also disable Gatekeeper entirely if you want to (look up documentation for that if you're so inclined)
7. Confirm *seal* works by running `seal --help`.
8. I do not know about webview/GUI dependencies on macOS; will update this closer to webview implementation on that.

## Linux

1. Download either the [latest release](<https://github.com/deviaze/seal/releases/latest>) of *seal*, a recent [build artifact](<https://github.com/deviaze/seal/actions>), or compile *seal* locally by cloning this repository and running `cargo b --release`.
2. Move *seal* to `~/.local/bin/seal` (or wherever else you're comfortable) and make sure it's added to your `$PATH` with `export PATH="$HOME/.local/bin:$PATH` or similar.
3. Confirm *seal* works by running `seal --help`.

## Android (Termux)

1. You need to build *seal* yourself, which means you need the Rust toolchain installed w/ nightly. Getting Rust Nightly installed on Termux is a big PITA so I'mma try to help you not have to discover everything yourself like I did.
2. `pkg update && pkg upgrade` your package manager.
3. You have to add the [Termux User Repository (TUR)](<https://github.com/termux-user-repository/tur>) with `pkg install tur-repo`
4. Now you can install rustc, cargo, and nightly with `pkg install rustc cargo rustc-nightly`
5. To make Rust default to nightly, you need to set an environment variable: `RUSTC=$PREFIX/opt/rust-nightly/bin/rustc` and you might want to prepend that path to your `$PATH` as well.
6. Clone *seal*, `cd` into it, and see if you can `cargo b --release`. If you are on androidabi instead of linux-android you might have additional problems compiling.
7. Ping me on Discord if you're having trouble getting *seal* to work on Android.
