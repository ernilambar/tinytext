mod app;
mod cli;
mod file_icons;
mod language;
mod markdown;
mod paths;
mod session;
mod settings;
mod update;

use std::path::PathBuf;

use gpui_kit::component::{Theme, ThemeMode, WindowExt as _, input, notification::Notification};
use gpui_kit::*;

use app::{TinytextApp, WINDOW_TITLE};
use cli::{CliCommand, parse_args, print_help};
use paths::path_from_file_url;
use session::load_session;
use settings::{DEFAULT_FONT_WEIGHT, load_settings};

gpui_kit::actions!(
    tinytext,
    [
        NewFile,
        OpenFile,
        OpenFolder,
        CloseFolder,
        SaveFile,
        SaveFileAs,
        SaveAll,
        CloseTab,
        Quit,
        Hide,
        HideOthers,
        ShowAll,
        ToggleSidebar,
        InstallCli,
        CheckForUpdates,
        About,
        OpenSettings,
        RevealInFinder,
        CopyFilePath,
        CopyRelativePath,
    ]
);

fn main() {
    match parse_args(std::env::args().skip(1)) {
        CliCommand::Run => {}
        CliCommand::Help => {
            print_help();
            return;
        }
        CliCommand::Version => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            return;
        }
        CliCommand::Unknown(arg) => {
            eprintln!(
                "{name}: unknown option '{arg}'",
                name = env!("CARGO_PKG_NAME")
            );
            eprintln!(
                "Run '{name} --help' for usage.",
                name = env!("CARGO_PKG_NAME")
            );
            std::process::exit(2);
        }
    }

    let (open_tx, open_rx) = async_channel::unbounded::<Vec<PathBuf>>();

    let application = gpui_kit::application().with_assets(gpui_kit::assets::Assets);
    application.on_open_urls(move |urls| {
        let paths: Vec<PathBuf> = urls
            .iter()
            .filter_map(|url| path_from_file_url(url))
            .collect();
        if !paths.is_empty() {
            let _ = open_tx.try_send(paths);
        }
    });

    application.run(move |cx| {
        gpui_kit::init(cx);
        Theme::change(ThemeMode::Dark, None, cx);
        // Lift the tab strip and mute inactive labels so the active tab, which
        // merges with the editor background, is the only bright thing on the bar.
        Theme::update(cx, |theme| {
            theme.tab_bar = theme.secondary;
            theme.tab_foreground = theme.muted_foreground;
            theme.tab_active = theme.background;
            theme.tab_active_foreground = theme.foreground;
        });

        let settings = load_settings();
        let base_weight = settings
            .as_ref()
            .map(|settings| settings.editor.font_weight().unwrap_or(DEFAULT_FONT_WEIGHT))
            .unwrap_or(DEFAULT_FONT_WEIGHT);
        markdown::apply_emphasis(cx, base_weight);

        let argument = std::env::args().nth(1);
        let initial_folder = argument
            .as_ref()
            .map(PathBuf::from)
            .filter(|path| path.is_dir());
        let session = if argument.is_none() {
            load_session()
        } else {
            None
        };

        cx.bind_keys([
            KeyBinding::new("cmd-n", NewFile, None),
            KeyBinding::new("cmd-o", OpenFile, None),
            KeyBinding::new("cmd-shift-o", OpenFolder, None),
            KeyBinding::new("cmd-s", SaveFile, None),
            KeyBinding::new("cmd-shift-s", SaveFileAs, None),
            KeyBinding::new("alt-cmd-s", SaveAll, None),
            KeyBinding::new("cmd-w", CloseTab, None),
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-h", Hide, None),
            KeyBinding::new("alt-cmd-h", HideOthers, None),
            KeyBinding::new("cmd-b", ToggleSidebar, None),
            KeyBinding::new("cmd-,", OpenSettings, None),
        ]);
        cx.on_action(|_: &Hide, cx| cx.hide());
        cx.on_action(|_: &HideOthers, cx| cx.hide_other_apps());
        cx.on_action(|_: &ShowAll, cx| cx.unhide_other_apps());
        cx.set_menus(app_menus());

        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some(WINDOW_TITLE.into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        gpui_kit::open_window(options, cx, move |window, cx| {
            let (settings, settings_error) = match settings {
                Ok(settings) => (settings, None),
                Err(message) => (Default::default(), Some(message)),
            };
            let app = cx.new(|cx| TinytextApp::new(initial_folder.clone(), settings, cx));
            // The notification layer is attached after this closure returns.
            if let Some(message) = settings_error {
                window.defer(cx, move |window, cx| {
                    window.push_notification(Notification::error(message), cx);
                });
            }
            if let Some(session) = session {
                app.update(cx, |this, cx| this.restore_session(session, window, cx));
            }
            app.update(cx, |this, cx| this.check_font_family(window, cx));
            // Re-render whenever the window is (de)activated so a file deleted
            // while the app was in the background is reflected in its tab.
            app.update(cx, |_this, cx| {
                cx.observe_window_activation(window, |_this, _window, cx| cx.notify())
                    .detach();
            });
            let focus_handle = app.read(cx).focus_handle.clone();
            focus_handle.focus(window, cx);

            app.update(cx, |_this, cx| {
                cx.spawn_in(window, async move |this, cx| {
                    while let Ok(paths) = open_rx.recv().await {
                        this.update_in(cx, |this, window, cx| {
                            for path in paths {
                                this.request_open(path, window, cx);
                            }
                        })
                        .ok();
                    }
                })
                .detach();
            });

            app
        })
        .expect("failed to open window");
    });
}

fn app_menus() -> Vec<Menu> {
    vec![
        Menu::new("Tinytext").items([
            MenuItem::action("About Tinytext", About),
            MenuItem::action("Check for Updates…", CheckForUpdates),
            MenuItem::separator(),
            MenuItem::action("Settings…", OpenSettings),
            MenuItem::separator(),
            MenuItem::action("Install Command Line Tool…", InstallCli),
            MenuItem::separator(),
            MenuItem::os_submenu("Services", SystemMenuType::Services),
            MenuItem::separator(),
            MenuItem::action("Hide Tinytext", Hide),
            MenuItem::action("Hide Others", HideOthers),
            MenuItem::action("Show All", ShowAll),
            MenuItem::separator(),
            MenuItem::action("Quit Tinytext", Quit),
        ]),
        Menu::new("File").items([
            MenuItem::action("New File", NewFile),
            MenuItem::action("Open…", OpenFile),
            MenuItem::action("Open Folder…", OpenFolder),
            MenuItem::action("Close Folder", CloseFolder),
            MenuItem::separator(),
            MenuItem::action("Save", SaveFile),
            MenuItem::action("Save As…", SaveFileAs),
            MenuItem::action("Save All", SaveAll),
            MenuItem::separator(),
            MenuItem::action("Close Tab", CloseTab),
        ]),
        Menu::new("Edit").items([
            MenuItem::os_action("Undo", input::Undo, OsAction::Undo),
            MenuItem::os_action("Redo", input::Redo, OsAction::Redo),
            MenuItem::separator(),
            MenuItem::os_action("Cut", input::Cut, OsAction::Cut),
            MenuItem::os_action("Copy", input::Copy, OsAction::Copy),
            MenuItem::os_action("Paste", input::Paste, OsAction::Paste),
            MenuItem::separator(),
            MenuItem::os_action("Select All", input::SelectAll, OsAction::SelectAll),
        ]),
        Menu::new("View").items([MenuItem::action("Toggle Sidebar", ToggleSidebar)]),
    ]
}
