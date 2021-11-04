use crate::{engine, palette, state, util};

use serde::{Deserialize, Serialize};

use std::{
    error::Error,
    fmt,
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
    string::ToString,
};

use toml_edit::Document as TomlDocument;

pub const MIN_WINDOW_WIDTH: u32 = 480;
pub const MAX_WINDOW_WIDTH: u32 = 5000;

pub const MIN_WINDOW_HEIGHT: u32 = 320;
pub const MAX_WINDOW_HEIGHT: u32 = 5000;

pub const DEFAULT_WINDOW_WIDTH: u32 = 1280;
pub const DEFAULT_WINDOW_HEIGHT: u32 = 800;

pub const DISPLAY: &str = "display";
pub const FULLSCREEN: &str = "fullscreen";
pub const WINDOW: &str = "window";

pub const VISUAL_STYLE: &str = "visual_style";
pub const PALETTE: &str = "palette";
pub const TILE_SIZE: &str = "tile_size";
pub const TEXT_SIZE: &str = "text_size";
pub const WINDOW_WIDTH: &str = "window_width";
pub const WINDOW_HEIGHT: &str = "window_height";
pub const BACKEND: &str = "backend";
pub const HIDE_UNSEEN_TILES: &str = "hide_unseen_tiles";
pub const FAST_DEPRESSION: &str = "fast_depression";
pub const PERMADEATH: &str = "permadeath";
pub const BACKGROUND_VOLUME: &str = "background_volume";
pub const SOUND_VOLUME: &str = "sound_volume";

pub const FIRST_EVER_STARTUP: &str = "first_ever_startup";

/// The colour palette that the user can select
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Palette {
    Classic,
    Accessible,
    Greyscale,
}

impl fmt::Display for Palette {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use self::Palette::*;
        let s = match *self {
            Classic => PALETTE_CLASSIC_STR,
            Accessible => PALETTE_ACCESSIBLE_STR,
            Greyscale => PALETTE_GREYSCALE_STR,
        };
        f.write_str(s)
    }
}

const PALETTE_CLASSIC_STR: &str = "classic";
const PALETTE_ACCESSIBLE_STR: &str = "accessible";
const PALETTE_GREYSCALE_STR: &str = "greyscale";

/// Settings the engine needs to carry.
///
/// Things such as the fullscreen/windowed display, font size, font
/// type, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub fullscreen: bool,
    pub visual_style: engine::VisualStyle,
    pub palette: Palette,
    pub text_size: i32,
    pub tile_size: i32,
    pub window_width: u32,
    pub window_height: u32,
    pub backend: String,
    pub hide_unseen_tiles: bool,
    pub fast_depression: bool,
    pub permadeath: bool,
    pub background_volume: f32,
    pub sound_volume: f32,
    pub first_ever_startup: bool,
}

impl Default for Settings {
    fn default() -> Self {
        // TODO: make backend an enum generated in the build script?
        let backend = if crate::engine::AVAILABLE_BACKENDS.contains(&"glutin") {
            "glutin"
        } else {
            crate::engine::AVAILABLE_BACKENDS.get(0).unwrap_or(&"none")
        };

        let settings = Self {
            fullscreen: false,
            visual_style: engine::VisualStyle::Graphical,
            palette: Palette::Classic,
            text_size: crate::engine::DEFAULT_TEXT_SIZE,
            tile_size: crate::engine::DEFAULT_TILE_SIZE,
            window_width: DEFAULT_WINDOW_WIDTH,
            window_height: DEFAULT_WINDOW_HEIGHT,
            backend: backend.into(),
            hide_unseen_tiles: true,
            fast_depression: true,
            permadeath: true,
            background_volume: 1.0,
            sound_volume: 1.0,
            first_ever_startup: true,
        };

        debug_assert!(settings.valid());
        settings
    }
}

#[allow(dead_code)]
impl Settings {
    pub fn valid(&self) -> bool {
        self.valid_tile_sizes() && self.valid_backend()
    }

    pub fn valid_tile_sizes(&self) -> bool {
        crate::engine::AVAILABLE_TEXT_SIZES.contains(&self.text_size)
            && crate::engine::AVAILABLE_TILE_SIZES.contains(&self.tile_size)
    }

    pub fn valid_backend(&self) -> bool {
        crate::engine::AVAILABLE_BACKENDS.contains(&self.backend.as_str())
    }

    pub fn challenge(&self) -> state::Challenge {
        state::Challenge {
            hide_unseen_tiles: self.hide_unseen_tiles,
            fast_depression: self.fast_depression,
            one_chance: self.permadeath,
        }
    }

    pub fn palette(&self) -> palette::Palette {
        match self.palette {
            Palette::Classic => palette::Palette::classic(),
            Palette::Accessible => palette::Palette::accessible(),
            Palette::Greyscale => palette::Palette::greyscale(),
        }
    }

    pub fn as_toml(&self) -> String {
        let mut out = String::with_capacity(1000);
        out.push_str(&format!(
            "# Options: \"{}\" or \"{}\"\n",
            FULLSCREEN, WINDOW
        ));
        out.push_str(&format!("{} = \"{}\"\n\n", DISPLAY, WINDOW));

        out.push_str(&format!(
            "# Options: \"{}\" or \"{}\"\n",
            engine::VisualStyle::Graphical,
            engine::VisualStyle::Textual
        ));
        out.push_str(&format!("{} = \"{}\"\n\n", VISUAL_STYLE, self.visual_style));

        out.push_str(&format!(
            "# Options: \"{}\", \"{}\" or \"{}\"\n",
            Palette::Classic,
            Palette::Accessible,
            Palette::Greyscale
        ));
        out.push_str(&format!("{} = \"{}\"\n\n", PALETTE, self.palette));

        let tile_sizes_str = crate::engine::AVAILABLE_TILE_SIZES
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("# Options: {}\n", tile_sizes_str));
        out.push_str(&format!("{} = {}\n\n", TILE_SIZE, self.tile_size));

        let text_sizes_str = crate::engine::AVAILABLE_TEXT_SIZES
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("# Options: {}\n", text_sizes_str));
        out.push_str(&format!("{} = {}\n\n", TEXT_SIZE, self.text_size));

        out.push_str(&format!("{} = {}\n", WINDOW_WIDTH, self.window_width));
        out.push_str(&format!("{} = {}\n\n", WINDOW_HEIGHT, self.window_height));

        let backends_str = crate::engine::AVAILABLE_BACKENDS
            .iter()
            .map(|b| format!("\"{}\"", b))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("# Options: {}\n", backends_str));

        out.push_str(&format!("{} = \"{}\"\n", BACKEND, self.backend));

        out.push_str(&format!(
            "{} = \"{}\"\n",
            HIDE_UNSEEN_TILES, self.hide_unseen_tiles
        ));

        out.push_str(&format!(
            "{} = \"{}\"\n",
            FAST_DEPRESSION, self.fast_depression
        ));

        out.push_str(&format!("{} = \"{}\"\n", PERMADEATH, self.permadeath));

        out.push_str(&"# Options: <0.0, 1.0>\n".to_string());
        out.push_str(&format!(
            "{} = \"{}\"\n",
            BACKGROUND_VOLUME, self.background_volume
        ));
        out.push_str(&"# Options: <0.0, 1.0>\n".to_string());
        out.push_str(&format!("{} = \"{}\"\n", SOUND_VOLUME, self.sound_volume));

        out.push_str(&format!(
            "{} = \"{}\"\n",
            FIRST_EVER_STARTUP, self.first_ever_startup
        ));

        out
    }
}

/// Trait that handles saving and loading the `Settings` to whatever
/// underlying storage solution. Could be a TOML file on the drive in
/// the current directory, browser's local storage, Windows Registy or
/// whatever else.
pub trait Store {
    fn load(&self) -> Settings;
    fn save(&mut self, settings: &Settings);
}

pub struct FileSystemStore {
    path: PathBuf,
    toml: TomlDocument,
}

#[allow(dead_code)]
impl FileSystemStore {
    /// Create a new `Settings` store backed by a TOML document on the
    /// filesystem. If the file does not exist, it will be created.
    pub fn new() -> Self {
        let filename = "settings.toml";
        let mut path = std::env::current_exe()
            .or_else(|_| std::env::current_dir())
            .unwrap_or_default();
        path.set_file_name(filename);
        log::info!("Settings will be stored at: '{}'", path.display());

        let toml = Self::read_settings_toml(&path).unwrap_or_else(|err| {
            log::error!("Could not open settings: {:?}", err);
            log::info!("Falling back to default settings.");
            let toml = Settings::default().as_toml().parse().unwrap_or_else(|err| {
                log::error!("Could not parse the default hardcoded settings: {}", err);
                log::info!("Generating an empty TOML document.");
                TomlDocument::new()
            });

            log::info!("Creating settings file at: {}", path.display());
            if let Err(err) = Self::write_settings_toml(&path, &toml) {
                log::error!("Could not write settings: {:?}.", err);
            }

            toml
        });

        Self { path, toml }
    }

    fn read_settings_toml(path: &Path) -> Result<TomlDocument, Box<dyn Error>> {
        let mut f = File::open(path)?;
        let mut buffer = String::with_capacity(1000);
        f.read_to_string(&mut buffer)?;
        let toml = buffer.parse::<TomlDocument>()?;

        Ok(toml)
    }

    fn write_settings_toml(path: &Path, toml: &TomlDocument) -> Result<(), Box<dyn Error>> {
        std::fs::write(path, toml.to_string_in_original_order())?;
        Ok(())
    }
}

impl Default for FileSystemStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for FileSystemStore {
    fn load(&self) -> Settings {
        let mut settings = Settings::default();

        match self.toml[DISPLAY].as_str() {
            Some(FULLSCREEN) => settings.fullscreen = true,
            Some(WINDOW) => settings.fullscreen = false,
            Some(unexpected) => {
                log::error!("Settings: unknown `{}` entry: {}", DISPLAY, unexpected);
                log::info!(
                    "Valid `{}` entries: \"{}\" or \"{}\"",
                    DISPLAY,
                    FULLSCREEN,
                    WINDOW
                );
            }
            None => log::error!("Settings: missing `{}` entry.", DISPLAY),
        }

        match self.toml[VISUAL_STYLE].as_str() {
            Some(engine::VISUAL_STYLE_GRAPHICAL_STR) => {
                settings.visual_style = engine::VisualStyle::Graphical
            }
            Some(engine::VISUAL_STYLE_TEXTUAL_STR) => {
                settings.visual_style = engine::VisualStyle::Textual
            }
            Some(unexpected) => {
                log::error!(
                    "Settings: unknown `{}` entry: \"{}\"",
                    VISUAL_STYLE,
                    unexpected
                );
                log::info!(
                    "Valid `{}` entries: \"{}\" or \"{}\"",
                    VISUAL_STYLE,
                    engine::VisualStyle::Graphical,
                    engine::VisualStyle::Textual
                );
            }
            None => log::info!(
                "Setting: missing `{}`, falling back to: \"{}\"",
                VISUAL_STYLE,
                settings.visual_style
            ),
        }

        match self.toml[PALETTE].as_str() {
            Some(PALETTE_CLASSIC_STR) => {
                settings.palette = Palette::Classic;
            }
            Some(PALETTE_ACCESSIBLE_STR) => {
                settings.palette = Palette::Accessible;
            }
            Some(PALETTE_GREYSCALE_STR) => {
                settings.palette = Palette::Greyscale;
            }
            Some(unexpected) => {
                log::error!("Settings: unknown `{}` entry: \"{}\"", PALETTE, unexpected);
                log::info!(
                    "Valid `{}` entries: \"{}\", \"{}\" or \"{}\"",
                    PALETTE,
                    Palette::Classic,
                    Palette::Accessible,
                    Palette::Greyscale
                );
            }
            None => log::info!(
                "Settings: missing `{}`, falling back to: \"{}\"",
                PALETTE,
                settings.palette
            ),
        }

        match self.toml[TILE_SIZE].as_integer() {
            Some(tile_size) => {
                let tile_size = tile_size as i32;
                if crate::engine::AVAILABLE_TILE_SIZES.contains(&tile_size) {
                    settings.tile_size = tile_size;
                } else {
                    log::error!("Settings: unsupported `{}`: {}", TILE_SIZE, tile_size);
                    log::info!(
                        "Available tile sizes: {:?}",
                        crate::engine::AVAILABLE_TILE_SIZES
                    );
                }
            }
            None => log::error!("Settings: missing `{}` entry.", TILE_SIZE),
        }

        match self.toml[TEXT_SIZE].as_integer() {
            Some(text_size) => {
                let text_size = text_size as i32;
                if crate::engine::AVAILABLE_TEXT_SIZES.contains(&text_size) {
                    settings.text_size = text_size;
                } else {
                    log::error!("Settings: unsupported `{}`: {}", TEXT_SIZE, text_size);
                    log::info!(
                        "Available text sizes: {:?}",
                        crate::engine::AVAILABLE_TEXT_SIZES
                    );
                }
            }
            None => log::error!("Settings: missing `{}` entry.", TEXT_SIZE),
        }

        match self.toml[WINDOW_WIDTH].as_integer() {
            Some(window_width) => {
                if window_width < i64::from(MIN_WINDOW_WIDTH) {
                    log::error!(
                        "Settings error: `{}` must be at least {}.",
                        WINDOW_WIDTH,
                        MIN_WINDOW_WIDTH
                    )
                } else if window_width > i64::from(MAX_WINDOW_WIDTH) {
                    log::error!(
                        "Settings error: `{}` cannot be greater than {}.",
                        WINDOW_WIDTH,
                        MAX_WINDOW_WIDTH
                    );
                } else {
                    settings.window_width = window_width as u32;
                }
            }
            None => log::error!("Settings: missing `{}` entry.", WINDOW_WIDTH),
        }

        match self.toml[WINDOW_HEIGHT].as_integer() {
            Some(window_height) => {
                if window_height < i64::from(MIN_WINDOW_HEIGHT) {
                    log::error!(
                        "Settings error: `{}` must be at least {}.",
                        WINDOW_HEIGHT,
                        MIN_WINDOW_HEIGHT
                    )
                } else if window_height > i64::from(MAX_WINDOW_HEIGHT) {
                    log::error!(
                        "Settings error: `{}` cannot be greater than {}.",
                        WINDOW_HEIGHT,
                        MAX_WINDOW_HEIGHT
                    );
                } else {
                    settings.window_height = window_height as u32;
                }
            }
            None => log::error!("Settings: missing `{}` entry.", WINDOW_HEIGHT),
        }

        match self.toml[BACKEND].as_str() {
            Some(backend) => {
                if crate::engine::AVAILABLE_BACKENDS.contains(&backend) {
                    settings.backend = backend.into();
                } else {
                    log::error!("Settings: unknown `{}`: {}", BACKEND, backend);
                    log::info!(
                        "Available backends: {:?}",
                        crate::engine::AVAILABLE_BACKENDS
                    );
                }
            }
            None => log::error!("Settings: missing `{}` entry.", BACKEND),
        }

        match self.toml[HIDE_UNSEEN_TILES].as_bool() {
            Some(hide_unseen_tiles) => {
                settings.hide_unseen_tiles = hide_unseen_tiles;
            }
            None => log::error!("Settings: missing `{}` entry.", HIDE_UNSEEN_TILES),
        }

        match self.toml[FAST_DEPRESSION].as_bool() {
            Some(fast_depression) => {
                settings.fast_depression = fast_depression;
            }
            None => log::error!("Settings: missing `{}` entry.", FAST_DEPRESSION),
        }

        match self.toml[PERMADEATH].as_bool() {
            Some(permadeath) => {
                settings.permadeath = permadeath;
            }
            None => log::error!("Settings: missing `{}` entry.", PERMADEATH),
        }

        match self.toml[BACKGROUND_VOLUME].as_float() {
            Some(volume) => {
                settings.background_volume = util::clampf(0.0, volume as f32, 1.0);
            }
            // toml_edit uses `f64.as_string()` which outputs e.g.
            // `1.0` as `1`. This then gets parsed here as integer not
            // float so we need to handle that case separately.
            None => match self.toml[BACKGROUND_VOLUME].as_integer() {
                Some(volume) => {
                    settings.background_volume = util::clampf(0.0, volume as f32, 1.0);
                }
                None => log::error!("Settings: missing `{}` entry.", BACKGROUND_VOLUME),
            },
        }

        match self.toml[SOUND_VOLUME].as_float() {
            Some(volume) => {
                settings.sound_volume = util::clampf(0.0, volume as f32, 1.0);
            }
            None => match self.toml[SOUND_VOLUME].as_integer() {
                Some(volume) => {
                    settings.sound_volume = util::clampf(0.0, volume as f32, 1.0);
                }
                None => log::error!("Settings: missing `{}` entry.", SOUND_VOLUME),
            },
        }

        match self.toml[FIRST_EVER_STARTUP].as_bool() {
            Some(first_ever_startup) => {
                settings.first_ever_startup = first_ever_startup;
            }
            None => log::error!("Settings: missing `{}` entry.", FIRST_EVER_STARTUP),
        }

        debug_assert!(settings.valid());

        log::info!("Loaded settings: {:?}", settings);

        settings
    }

    fn save(&mut self, settings: &Settings) {
        log::info!("Saving new settings to file {}", self.path.display());
        let display = match settings.fullscreen {
            true => FULLSCREEN,
            false => WINDOW,
        };
        self.toml[DISPLAY] = toml_edit::value(display);

        self.toml[VISUAL_STYLE] = toml_edit::value(settings.visual_style.to_string());

        self.toml[PALETTE] = toml_edit::value(settings.palette.to_string());

        self.toml[TILE_SIZE] = toml_edit::value(i64::from(settings.tile_size));

        self.toml[TEXT_SIZE] = toml_edit::value(i64::from(settings.text_size));

        if settings.fullscreen {
            // NOTE: don't save the window width/height when we're in
            // the full screen mode. Because that means once we
            // switched to windowed, the window would keep the fullscreen size.
            //
            // Instead, we want to return to the size the window had
            // before going full screen.
        } else {
            self.toml[WINDOW_WIDTH] = toml_edit::value(i64::from(settings.window_width));
            self.toml[WINDOW_HEIGHT] = toml_edit::value(i64::from(settings.window_height));
        }

        self.toml[BACKEND] = toml_edit::value(settings.backend.clone());

        self.toml[HIDE_UNSEEN_TILES] = toml_edit::value(settings.hide_unseen_tiles);

        self.toml[FAST_DEPRESSION] = toml_edit::value(settings.fast_depression);

        self.toml[PERMADEATH] = toml_edit::value(settings.permadeath);

        self.toml[BACKGROUND_VOLUME] = toml_edit::value(f64::from(settings.background_volume));

        self.toml[SOUND_VOLUME] = toml_edit::value(f64::from(settings.sound_volume));

        self.toml[FIRST_EVER_STARTUP] = toml_edit::value(settings.first_ever_startup);

        if let Err(err) = Self::write_settings_toml(&self.path, &self.toml) {
            log::error!("Could not write settings to the storage: {:?}", err);
        }
    }
}

#[allow(dead_code, missing_copy_implementations)]
pub struct NoOpStore;

impl Store for NoOpStore {
    fn load(&self) -> Settings {
        Default::default()
    }

    fn save(&mut self, _settings: &Settings) {}
}
