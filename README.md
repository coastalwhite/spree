# Spree

A simple power menu written in Rust. I wrote this because I wanted to experiment
with [Slint](https://slint.dev/) and was missing a nice and simple power menu.
This is alpha software at best. I currently have no plans to support or update
this much.

## Installation

To build you need the `cargo` toolchain for Rust.

```
cargo build --release

cp ./target/release/spree /usr/bin/spree

mkdir -p "$HOME/.config/spree"
cp ./config.toml "$HOME/.config/spree/"
```

Then, you can add a keybinding to your hotkey daemon, desktop environment,
compositor or window manager that calls `spree`. Note you will need to change
the current `config.toml` to suit your environment. Specifically, the `lock` and
`logout` buttons need a different `command` value if you are not using `sway`.

## Usage

You can setup a number of buttons in the `~/.config/spree/config.toml` that call
specific commands. When spree there are three ways to select a button.

1. Mouse
2. Keybinding for the button in the `config.toml`. To confirm a choice press the
   *Return* key.
3. Using the arrows, tab or VIM-style `h` and `l` to switch through the options.
   To confirm a choice press the *Return* key.

To close or deselect an option, press the *Escape* key.

## License

Licensed under an MIT license.