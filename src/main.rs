slint::include_modules!();

use std::path::PathBuf;

use clap::Parser;
use slint::platform::Key;
use slint::SharedString;

use crate::config::Config;

mod config;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: Option<PathBuf>,

    #[arg(short, long, default_value_t = false)]
    dry_run: bool,
}

const MAX_BUTTONS: usize = 10;

fn get_config_dir() -> PathBuf {
    if let Ok(config_dir) = ::std::env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(config_dir);
        if path.exists() {
            return path;
        }
    }

    if let Ok(home_dir) = ::std::env::var("HOME") {
        let home_path = PathBuf::from(home_dir);
        if home_path.join(".config").exists() {
            return home_path.join(".config");
        }
    }

    eprintln!("Failed to find config directory");
    ::std::process::exit(1);
}

fn main() {
    let args = Args::parse();

    assert!(u16::try_from(MAX_BUTTONS).is_ok());

    let config_path = args.config.unwrap_or_else(|| {
        let config_path = get_config_dir();
        config_path.join("spree").join("config.toml")
    });

    let config = ::std::fs::read_to_string(&config_path).unwrap_or_else(|err| {
        eprintln!(
            "Failed to open configuration file at '{path}'. Reason: {err}",
            path = config_path.display()
        );
        ::std::process::exit(1);
    });

    let config: Config = toml::from_str(&config).unwrap_or_else(|err| {
        eprintln!(
            "Failed to parse configuration file at '{path}'. Reason: {err}",
            path = config_path.display()
        );
        ::std::process::exit(1);
    });

    let main_window = MainWindow::new().unwrap();

    let Config {
        primary,
        secondary,
        enable_tab_navigation,
        enable_vim_navigation,
        enable_arrow_navigation,
        border_width,
        border_color,
        padding,
        spacing,
        button_dim,
        buttons,
    } = config;

    let btns = buttons;

    let primary = primary.take();
    let secondary = secondary.take();

    main_window.set_primary_color(primary);
    main_window.set_secondary_color(secondary);
    main_window.set_dim_padding(padding);
    main_window.set_dim_spacing(spacing);
    main_window.set_border_width(border_width);
    main_window.set_border_color(border_color.take());
    main_window.set_button_dim(button_dim);

    let num_btns = btns.len();
    if num_btns > MAX_BUTTONS {
        eprintln!("Too many buttons. (current = {num_btns}, max = {MAX_BUTTONS})");
        std::process::exit(1);
    }

    if num_btns == 0 {
        eprintln!("No buttons given.");
        std::process::exit(1);
    }

    let button_keys = btns
        .iter()
        .map(|btn| {
            let s = btn.key.as_ref()?;
            let mut chars = s.chars();
            let btn = chars.next()?;

            if chars.next().is_some() {
                return Some(Err(s.clone()));
            }

            if btn == char::from(Key::Tab) {
                return Some(Err(s.clone()));
            }

            if btn == char::from(Key::Return) {
                return Some(Err(s.clone()));
            }

            if btn == char::from(Key::LeftArrow) {
                return Some(Err(s.clone()));
            }

            if btn == char::from(Key::RightArrow) {
                return Some(Err(s.clone()));
            }

            if matches!(btn, 'h' | 'l') && enable_vim_navigation {
                eprintln!("Key conflict with VIM-style navigation bindings");
                ::std::process::exit(1);
            }

            Some(Ok(btn))
        })
        .enumerate()
        .map(|(i, c)| Ok((i as u16, c.transpose()?)))
        .collect::<Result<Vec<(u16, Option<char>)>, String>>()
        .unwrap_or_else(|err| {
            eprintln!("Invalid button key '{err}'");
            ::std::process::exit(1);
        });

    // We can just leak this. It will be cleaned up when the program is done and the data is
    // reasonably small.
    let btns = btns.leak();

    use aho_corasick::{AhoCorasick, MatchKind};

    let primary_color = format!(
        "rgba({},{},{},{})",
        primary.red(),
        primary.green(),
        primary.blue(),
        f32::from(primary.alpha()) / 255.,
    );
    let secondary_color = format!(
        "rgba({},{},{},{})",
        secondary.red(),
        secondary.green(),
        secondary.blue(),
        f32::from(secondary.alpha()) / 255.,
    );

    let patterns = &["%primaryColor%", "%secondaryColor%"];

    let ac = AhoCorasick::builder()
        .match_kind(MatchKind::LeftmostFirst)
        .build(patterns)
        .unwrap();

    let btns_model = std::rc::Rc::new(slint::VecModel::from(
        btns.iter()
            .map(|btn| {
                let focussed_icon =
                    ac.replace_all(&btn.icon.content, &[&primary_color, &secondary_color]);
                let unfocussed_icon =
                    ac.replace_all(&btn.icon.content, &[&secondary_color, &primary_color]);

                ButtonData {
                    focussed_icon: slint::Image::load_from_svg_data(focussed_icon.as_bytes())
                        .unwrap(),
                    unfocussed_icon: slint::Image::load_from_svg_data(unfocussed_icon.as_bytes())
                        .unwrap(),
                }
            })
            .collect::<Vec<ButtonData>>(),
    ));

    main_window.on_exit(|| {
        std::process::exit(0);
    });

    let dry_run = args.dry_run;
    main_window.on_btn_clicked(move |i| {
        if dry_run {
            println!("Invoked button #{i}");
            return;
        }

        let argv: &[String] = btns[i as usize].command.as_ref();

        if let Err(err) = std::process::Command::new(&argv[0])
            .args(&argv[1..])
            .status()
        {
            eprintln!("Failed to run command for button #{i}. Reason: {err}");
        }
    });

    let num_btns = button_keys.len() as u16;

    main_window.on_key_pressed(move |selected, key| {
        let left = if selected == -1 || selected == 0 {
            num_btns as i32 - 1
        } else {
            selected - 1
        };

        let right = if selected == -1 || selected == num_btns as i32 - 1 {
            0
        } else {
            selected + 1
        };

        // Tab navigation
        if enable_tab_navigation && key.text == SharedString::from(Key::Tab) {
            if key.modifiers.control || key.modifiers.alt || key.modifiers.meta {
                return -1;
            }

            return if key.modifiers.shift { left } else { right };
        }

        if enable_vim_navigation && key.text.as_str() == "h" {
            if key.modifiers.control
                || key.modifiers.alt
                || key.modifiers.meta
                || key.modifiers.shift
            {
                return -1;
            }

            return left;
        }

        if enable_vim_navigation && key.text.as_str() == "l" {
            if key.modifiers.control
                || key.modifiers.alt
                || key.modifiers.meta
                || key.modifiers.shift
            {
                return -1;
            }

            return right;
        }

        if enable_arrow_navigation && key.text == SharedString::from(Key::LeftArrow) {
            if key.modifiers.control
                || key.modifiers.alt
                || key.modifiers.meta
                || key.modifiers.shift
            {
                return -1;
            }

            return left;
        }

        if enable_arrow_navigation && key.text == SharedString::from(Key::RightArrow) {
            if key.modifiers.control
                || key.modifiers.alt
                || key.modifiers.meta
                || key.modifiers.shift
            {
                return -1;
            }

            return right;
        }

        for (i, btn) in button_keys.iter() {
            let Some(k) = btn else {
                continue;
            };

            if key.text.as_str().starts_with(*k) && key.text.len() == k.len_utf8() {
                return (*i).into();
            }
        }

        -1
    });

    main_window.set_btns(btns_model.into());

    // Focus on the FocusScope to handle key presses on startup without requiring any clicking
    main_window.invoke_allow_keypresses();

    main_window.show().unwrap();
    slint::run_event_loop().unwrap();
}
