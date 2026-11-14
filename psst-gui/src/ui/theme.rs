use std::{
    collections::HashSet,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use druid::{Color, Env, ExtEventSink, FontDescriptor, FontFamily, FontWeight, Insets, Key, Size, Target};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use psst_core::cache::mkdir_if_not_exists;

pub use druid::theme::*;

use crate::{
    cmd,
    data::{AppState, Config, CustomTheme, Theme, ThemeColor},
};

pub fn grid(m: f64) -> f64 {
    GRID * m
}

pub const GRID: f64 = 8.0;

pub const GREY_000: Key<Color> = Key::new("app.grey_000");
pub const GREY_100: Key<Color> = Key::new("app.grey_100");
pub const GREY_200: Key<Color> = Key::new("app.grey_200");
pub const GREY_300: Key<Color> = Key::new("app.grey_300");
pub const GREY_400: Key<Color> = Key::new("app.grey_400");
pub const GREY_500: Key<Color> = Key::new("app.grey_500");
pub const GREY_600: Key<Color> = Key::new("app.grey_600");
pub const GREY_700: Key<Color> = Key::new("app.grey_700");
pub const BLUE_100: Key<Color> = Key::new("app.blue_100");
pub const BLUE_200: Key<Color> = Key::new("app.blue_200");

pub const RED: Key<Color> = Key::new("app.red");

pub const MENU_BUTTON_BG_ACTIVE: Key<Color> = Key::new("app.menu-bg-active");
pub const MENU_BUTTON_BG_INACTIVE: Key<Color> = Key::new("app.menu-bg-inactive");
pub const MENU_BUTTON_FG_ACTIVE: Key<Color> = Key::new("app.menu-fg-active");
pub const MENU_BUTTON_FG_INACTIVE: Key<Color> = Key::new("app.menu-fg-inactive");

pub const UI_FONT_MEDIUM: Key<FontDescriptor> = Key::new("app.ui-font-medium");
pub const UI_FONT_MONO: Key<FontDescriptor> = Key::new("app.ui-font-mono");
pub const TEXT_SIZE_SMALL: Key<f64> = Key::new("app.text-size-small");

pub const ICON_COLOR: Key<Color> = Key::new("app.icon-color");
pub const ICON_SIZE_TINY: Size = Size::new(12.0, 12.0);
pub const ICON_SIZE_SMALL: Size = Size::new(14.0, 14.0);
pub const ICON_SIZE_MEDIUM: Size = Size::new(16.0, 16.0);
pub const ICON_SIZE_LARGE: Size = Size::new(22.0, 22.0);

pub const LINK_HOT_COLOR: Key<Color> = Key::new("app.link-hot-color");
pub const LINK_ACTIVE_COLOR: Key<Color> = Key::new("app.link-active-color");
pub const LINK_COLD_COLOR: Key<Color> = Key::new("app.link-cold-color");

pub fn setup(env: &mut Env, state: &AppState) {
    match state.config.theme {
        Theme::Light => setup_light_theme(env),
        Theme::Dark => setup_dark_theme(env),
    };
    apply_palette_overrides(env, &state.custom_theme);

    env.set(WINDOW_BACKGROUND_COLOR, env.get(GREY_700));
    env.set(TEXT_COLOR, env.get(GREY_100));
    env.set(ICON_COLOR, env.get(GREY_400));
    env.set(PLACEHOLDER_COLOR, env.get(GREY_300));
    env.set(PRIMARY_LIGHT, env.get(BLUE_100));
    env.set(PRIMARY_DARK, env.get(BLUE_200));

    env.set(BACKGROUND_LIGHT, env.get(GREY_700));
    env.set(BACKGROUND_DARK, env.get(GREY_600));
    env.set(FOREGROUND_LIGHT, env.get(GREY_100));
    env.set(FOREGROUND_DARK, env.get(GREY_000));

    match state.config.theme {
        Theme::Light => {
            env.set(BUTTON_LIGHT, env.get(GREY_700));
            env.set(BUTTON_DARK, env.get(GREY_600));
        }
        Theme::Dark => {
            env.set(BUTTON_LIGHT, env.get(GREY_600));
            env.set(BUTTON_DARK, env.get(GREY_700));
        }
    }

    env.set(BORDER_LIGHT, env.get(GREY_400));
    env.set(BORDER_DARK, env.get(GREY_500));

    env.set(SELECTED_TEXT_BACKGROUND_COLOR, env.get(BLUE_200));
    env.set(SELECTION_TEXT_COLOR, env.get(GREY_700));

    env.set(CURSOR_COLOR, env.get(GREY_000));

    env.set(PROGRESS_BAR_RADIUS, 4.0);
    env.set(BUTTON_BORDER_RADIUS, 4.0);
    env.set(BUTTON_BORDER_WIDTH, 1.0);

    env.set(
        UI_FONT,
        FontDescriptor::new(FontFamily::SYSTEM_UI).with_size(13.0),
    );
    env.set(
        UI_FONT_MEDIUM,
        FontDescriptor::new(FontFamily::SYSTEM_UI)
            .with_size(13.0)
            .with_weight(FontWeight::MEDIUM),
    );
    env.set(
        UI_FONT_MONO,
        FontDescriptor::new(FontFamily::MONOSPACE).with_size(13.0),
    );
    env.set(TEXT_SIZE_SMALL, 11.0);
    env.set(TEXT_SIZE_NORMAL, 13.0);
    env.set(TEXT_SIZE_LARGE, 16.0);

    env.set(BASIC_WIDGET_HEIGHT, 16.0);
    env.set(WIDE_WIDGET_WIDTH, grid(12.0));
    env.set(BORDERED_WIDGET_HEIGHT, grid(4.0));

    env.set(TEXTBOX_BORDER_RADIUS, 4.0);
    env.set(TEXTBOX_BORDER_WIDTH, 1.0);
    env.set(TEXTBOX_INSETS, Insets::uniform_xy(grid(1.2), grid(1.0)));

    env.set(SCROLLBAR_COLOR, env.get(GREY_300));
    env.set(SCROLLBAR_BORDER_COLOR, env.get(GREY_300));
    env.set(SCROLLBAR_MAX_OPACITY, 0.8);
    env.set(SCROLLBAR_FADE_DELAY, 1500u64);
    env.set(SCROLLBAR_WIDTH, 6.0);
    env.set(SCROLLBAR_PAD, 2.0);
    env.set(SCROLLBAR_RADIUS, 5.0);
    env.set(SCROLLBAR_EDGE_WIDTH, 1.0);

    env.set(WIDGET_PADDING_VERTICAL, grid(0.5));
    env.set(WIDGET_PADDING_HORIZONTAL, grid(1.0));
    env.set(WIDGET_CONTROL_COMPONENT_PADDING, grid(1.0));

    env.set(MENU_BUTTON_BG_ACTIVE, env.get(GREY_500));
    env.set(MENU_BUTTON_BG_INACTIVE, env.get(GREY_600));
    env.set(MENU_BUTTON_FG_ACTIVE, env.get(GREY_000));
    env.set(MENU_BUTTON_FG_INACTIVE, env.get(GREY_100));

    apply_semantic_overrides(env, &state.custom_theme);
}

fn set_if_some(env: &mut Env, key: Key<Color>, color: Option<ThemeColor>) {
    if let Some(color) = color {
        env.set(key, color.to_druid_color());
    }
}

fn apply_palette_overrides(env: &mut Env, custom: &CustomTheme) {
    set_if_some(env, GREY_000, custom.grey_000);
    set_if_some(env, GREY_100, custom.grey_100);
    set_if_some(env, GREY_200, custom.grey_200);
    set_if_some(env, GREY_300, custom.grey_300);
    set_if_some(env, GREY_400, custom.grey_400);
    set_if_some(env, GREY_500, custom.grey_500);
    set_if_some(env, GREY_600, custom.grey_600);
    set_if_some(env, GREY_700, custom.grey_700);
    set_if_some(env, BLUE_100, custom.blue_100);
    set_if_some(env, BLUE_200, custom.blue_200);
    set_if_some(env, RED, custom.red);
    set_if_some(env, LINK_HOT_COLOR, custom.link_hot_color);
    set_if_some(env, LINK_ACTIVE_COLOR, custom.link_active_color);
    set_if_some(env, LINK_COLD_COLOR, custom.link_cold_color);
}

fn apply_semantic_overrides(env: &mut Env, custom: &CustomTheme) {
    set_if_some(env, WINDOW_BACKGROUND_COLOR, custom.window_background_color);
    set_if_some(env, TEXT_COLOR, custom.text_color);
    set_if_some(env, ICON_COLOR, custom.icon_color);
    set_if_some(env, PLACEHOLDER_COLOR, custom.placeholder_color);
    set_if_some(env, PRIMARY_LIGHT, custom.primary_light);
    set_if_some(env, PRIMARY_DARK, custom.primary_dark);
    set_if_some(env, BACKGROUND_LIGHT, custom.background_light);
    set_if_some(env, BACKGROUND_DARK, custom.background_dark);
    set_if_some(env, FOREGROUND_LIGHT, custom.foreground_light);
    set_if_some(env, FOREGROUND_DARK, custom.foreground_dark);
    set_if_some(env, BUTTON_LIGHT, custom.button_light);
    set_if_some(env, BUTTON_DARK, custom.button_dark);
    set_if_some(env, BORDER_LIGHT, custom.border_light);
    set_if_some(env, BORDER_DARK, custom.border_dark);
    set_if_some(
        env,
        SELECTED_TEXT_BACKGROUND_COLOR,
        custom.selected_text_background_color,
    );
    set_if_some(env, SELECTION_TEXT_COLOR, custom.selection_text_color);
    set_if_some(env, CURSOR_COLOR, custom.cursor_color);
    set_if_some(env, SCROLLBAR_COLOR, custom.scrollbar_color);
    set_if_some(env, SCROLLBAR_BORDER_COLOR, custom.scrollbar_border_color);
    set_if_some(env, MENU_BUTTON_BG_ACTIVE, custom.menu_button_bg_active);
    set_if_some(env, MENU_BUTTON_BG_INACTIVE, custom.menu_button_bg_inactive);
    set_if_some(env, MENU_BUTTON_FG_ACTIVE, custom.menu_button_fg_active);
    set_if_some(env, MENU_BUTTON_FG_INACTIVE, custom.menu_button_fg_inactive);
}

fn setup_light_theme(env: &mut Env) {
    env.set(GREY_000, Color::grey8(0x00));
    env.set(GREY_100, Color::grey8(0x33));
    env.set(GREY_200, Color::grey8(0x4f));
    env.set(GREY_300, Color::grey8(0x82));
    env.set(GREY_400, Color::grey8(0xbd));
    env.set(GREY_500, Color::from_rgba32_u32(0xe5e6e7ff));
    env.set(GREY_600, Color::from_rgba32_u32(0xf5f6f7ff));
    env.set(GREY_700, Color::from_rgba32_u32(0xffffffff));
    env.set(BLUE_100, Color::rgb8(0x5c, 0xc4, 0xff));
    env.set(BLUE_200, Color::rgb8(0x00, 0x8d, 0xdd));

    env.set(RED, Color::rgba8(0xEB, 0x57, 0x57, 0xFF));

    env.set(LINK_HOT_COLOR, Color::rgba(0.0, 0.0, 0.0, 0.06));
    env.set(LINK_ACTIVE_COLOR, Color::rgba(0.0, 0.0, 0.0, 0.04));
    env.set(LINK_COLD_COLOR, Color::rgba(0.0, 0.0, 0.0, 0.0));
}

fn setup_dark_theme(env: &mut Env) {
    env.set(GREY_000, Color::grey8(0xff));
    env.set(GREY_100, Color::grey8(0xf2));
    env.set(GREY_200, Color::grey8(0xe0));
    env.set(GREY_300, Color::grey8(0xbd));
    env.set(GREY_400, Color::grey8(0x82));
    env.set(GREY_500, Color::grey8(0x4f));
    env.set(GREY_600, Color::grey8(0x33));
    env.set(GREY_700, Color::grey8(0x28));
    env.set(BLUE_100, Color::rgb8(0x00, 0x8d, 0xdd));
    env.set(BLUE_200, Color::rgb8(0x5c, 0xc4, 0xff));

    env.set(RED, Color::rgba8(0xEB, 0x57, 0x57, 0xFF));

    env.set(LINK_HOT_COLOR, Color::rgba(1.0, 1.0, 1.0, 0.05));
    env.set(LINK_ACTIVE_COLOR, Color::rgba(1.0, 1.0, 1.0, 0.025));
    env.set(LINK_COLD_COLOR, Color::rgba(1.0, 1.0, 1.0, 0.0));
}

pub fn spawn_watcher(event_sink: ExtEventSink) {
    let Some(theme_path) = Config::theme_path() else {
        log::warn!("Cannot watch custom theme: failed to resolve theme path");
        return;
    };

    if let Some(parent) = theme_path.parent() {
        if let Err(err) = mkdir_if_not_exists(parent) {
            log::warn!(
                "Failed to create directory {:?} for theme watcher: {err}",
                parent
            );
        }
    }

    thread::Builder::new()
        .name("theme_watcher".into())
        .spawn(move || watch_theme_loop(theme_path, event_sink))
        .expect("Failed to spawn theme watcher thread");
}

fn watch_theme_loop(theme_path: PathBuf, event_sink: ExtEventSink) {
    const DEBOUNCE_WINDOW: Duration = Duration::from_millis(50);

    let (tx, rx) = crossbeam_channel::unbounded();
    let mut watcher = match notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    }) {
        Ok(watcher) => watcher,
        Err(err) => {
            log::error!("Failed to initialize theme file watcher: {err}");
            return;
        }
    };

    let mut watched_dirs = HashSet::new();
    sync_watched_dirs(&mut watcher, &mut watched_dirs, &theme_path);
    log::info!("Watching custom theme file: {:?}", theme_path);

    let mut last_loaded: Option<CustomTheme> = None;

    while let Ok(res) = rx.recv() {
        let mut should_reload = event_matches_target(&res, &theme_path);

        // Drain bursty editor/atomic-save events within a short debounce window.
        while let Ok(next_res) = rx.recv_timeout(DEBOUNCE_WINDOW) {
            if event_matches_target(&next_res, &theme_path) {
                should_reload = true;
            }
        }

        if !should_reload {
            continue;
        }

        sync_watched_dirs(&mut watcher, &mut watched_dirs, &theme_path);

        if let Some(new_theme) = load_theme_with_retry(&theme_path) {
            if last_loaded.as_ref() != Some(&new_theme) {
                last_loaded = Some(new_theme.clone());
                if event_sink
                    .submit_command(cmd::RELOAD_CUSTOM_THEME, new_theme, Target::Global)
                    .is_err()
                {
                    break;
                }
            }
        }
    }
}

fn sync_watched_dirs(
    watcher: &mut dyn Watcher,
    watched_dirs: &mut HashSet<PathBuf>,
    theme_path: &Path,
) {
    let mut candidates = Vec::new();
    if let Some(parent) = theme_path.parent() {
        candidates.push(parent.to_path_buf());
    }
    if let Ok(canonical) = fs::canonicalize(theme_path) {
        if let Some(parent) = canonical.parent() {
            candidates.push(parent.to_path_buf());
        }
    }

    for dir in candidates {
        if !watched_dirs.contains(&dir) && dir.exists() {
            match watcher.watch(&dir, RecursiveMode::NonRecursive) {
                Ok(()) => {
                    watched_dirs.insert(dir);
                }
                Err(err) => {
                    log::warn!("Failed to watch directory {:?} for theme changes: {err}", dir);
                }
            }
        }
    }
}

fn event_matches_target(res: &notify::Result<Event>, theme_path: &Path) -> bool {
    let event = match res {
        Ok(event) => event,
        Err(err) => {
            log::warn!("Theme watcher error: {err}");
            return false;
        }
    };

    if matches!(event.kind, EventKind::Access(_)) {
        return false;
    }

    let primary_name = theme_path.file_name();
    let canonical_path = fs::canonicalize(theme_path).ok();
    let canonical_name = canonical_path.as_deref().and_then(Path::file_name);

    event_paths_match(&event.paths, primary_name, canonical_name)
}

fn event_paths_match(
    paths: &[PathBuf],
    primary_name: Option<&OsStr>,
    canonical_name: Option<&OsStr>,
) -> bool {
    if paths.is_empty() {
        return true;
    }

    paths.iter().any(|path| {
        let name = path.file_name();
        (primary_name.is_some() && name == primary_name)
            || (canonical_name.is_some() && name == canonical_name)
    })
}

fn load_theme_with_retry(theme_path: &Path) -> Option<CustomTheme> {
    if !theme_path.exists() {
        log::info!("Custom theme file removed ({:?}), reverting to base theme", theme_path);
        return Some(CustomTheme::default());
    }

    match CustomTheme::load_from_path(theme_path) {
        Ok(theme) => Some(theme),
        Err(first_err) => {
            // Non-atomic writers may temporarily leave an empty/partial file.
            thread::sleep(Duration::from_millis(80));
            if !theme_path.exists() {
                return Some(CustomTheme::default());
            }
            match CustomTheme::load_from_path(theme_path) {
                Ok(theme) => Some(theme),
                Err(_) => {
                    log::warn!("{first_err}");
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{AccessKind, CreateKind, ModifyKind, RemoveKind};
    use std::io::Write;

    #[test]
    fn setup_applies_palette_and_semantic_overrides_over_base_theme() {
        let mut state = AppState::default_with_config(Config::default());
        state.config.theme = Theme::Dark;
        state.custom_theme.grey_700 = Some(ThemeColor::from_rgba8(0x11, 0x22, 0x33, 0xff));
        state.custom_theme.window_background_color =
            Some(ThemeColor::from_rgba8(0x44, 0x55, 0x66, 0xff));

        let mut env = Env::empty();
        setup(&mut env, &state);

        // Derived key inherits overridden palette color grey_700.
        assert_eq!(
            env.get(BACKGROUND_LIGHT),
            Color::rgb8(0x11, 0x22, 0x33)
        );
        assert_eq!(
            env.get(BUTTON_DARK),
            Color::rgb8(0x11, 0x22, 0x33)
        );
        // Explicit semantic override takes precedence over derived window_background_color.
        assert_eq!(
            env.get(WINDOW_BACKGROUND_COLOR),
            Color::rgb8(0x44, 0x55, 0x66)
        );

        // Clearing overrides restores Dark theme defaults.
        state.custom_theme = CustomTheme::default();
        setup(&mut env, &state);
        assert_eq!(env.get(GREY_700), Color::grey8(0x28));
        assert_eq!(env.get(WINDOW_BACKGROUND_COLOR), Color::grey8(0x28));
        assert_eq!(env.get(BACKGROUND_LIGHT), Color::grey8(0x28));
    }

    #[test]
    fn event_filter_matches_theme_filename_and_ignores_access() {
        let target = PathBuf::from("/tmp/psst/theme.json");
        let other = PathBuf::from("/tmp/psst/config.json");

        let modify_target = Ok(Event::new(EventKind::Modify(ModifyKind::Any)).add_path(target.clone()));
        assert!(event_matches_target(&modify_target, &target));

        let modify_other = Ok(Event::new(EventKind::Modify(ModifyKind::Any)).add_path(other));
        assert!(!event_matches_target(&modify_other, &target));

        let access_target = Ok(Event::new(EventKind::Access(AccessKind::Any)).add_path(target.clone()));
        assert!(!event_matches_target(&access_target, &target));

        let create_target = Ok(Event::new(EventKind::Create(CreateKind::File)).add_path(target.clone()));
        assert!(event_matches_target(&create_target, &target));

        let remove_target = Ok(Event::new(EventKind::Remove(RemoveKind::File)).add_path(target.clone()));
        assert!(event_matches_target(&remove_target, &target));
    }

    #[test]
    fn load_theme_with_retry_handles_missing_valid_and_invalid_files() {
        let dir = std::env::temp_dir().join(format!(
            "psst-theme-watch-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("theme.json");

        // Missing file reverts to default (empty) overrides.
        assert_eq!(load_theme_with_retry(&path), Some(CustomTheme::default()));

        // Valid JSON loads overrides.
        {
            let mut file = fs::File::create(&path).unwrap();
            writeln!(file, r##"{{"blue_200": "#a6e3a1"}}"##).unwrap();
        }
        let loaded = load_theme_with_retry(&path).unwrap();
        assert_eq!(
            loaded.blue_200,
            Some(ThemeColor::from_rgba8(0xa6, 0xe3, 0xa1, 0xff))
        );

        // Invalid JSON preserves active theme by returning None.
        {
            let mut file = fs::File::create(&path).unwrap();
            writeln!(file, "{{invalid").unwrap();
        }
        assert_eq!(load_theme_with_retry(&path), None);

        let _ = fs::remove_dir_all(&dir);
    }
}
