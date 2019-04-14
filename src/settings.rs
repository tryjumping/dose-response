use serde::{Deserialize, Serialize};

use std::{
    error::Error,
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
};

use toml_edit::Document as TomlDocument;

/// Settings the engine needs to carry.
///
/// Things such as the fullscreen/windowed display, font size, font
/// type, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub fullscreen: bool,
    pub tile_size: i32,
    pub backend: String,
}

impl Default for Settings {
    fn default() -> Self {
        // TODO: make backend an enum generated in the build script?
        let backend = if crate::engine::AVAILABLE_BACKENDS.contains(&"glutin") {
            "glutin"
        } else {
            crate::engine::AVAILABLE_BACKENDS[0]
        };

        let settings = Self {
            fullscreen: false,
            tile_size: crate::engine::DEFAULT_TILESIZE,
            backend: backend.into(),
        };

        debug_assert!(settings.valid());
        settings
    }
}

impl Settings {
    pub fn valid(&self) -> bool {
        self.valid_tile_size() && self.valid_backend()
    }

    pub fn valid_tile_size(&self) -> bool {
        crate::engine::AVAILABLE_FONT_SIZES.contains(&self.tile_size)
    }

    pub fn valid_backend(&self) -> bool {
        crate::engine::AVAILABLE_BACKENDS.contains(&self.backend.as_str())
    }

    pub fn as_toml(&self) -> String {
        let mut out = String::with_capacity(1000);
        out.push_str("# Options: \"fullscreen\" or \"window\"\n");
        out.push_str("display = \"window\"\n\n");

        let tile_sizes_str = crate::engine::AVAILABLE_FONT_SIZES
            .iter()
            .map(|num| num.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("# Options: {}\n", tile_sizes_str));
        out.push_str(&format!("tile_size = {}\n\n", self.tile_size));

        let backends_str = crate::engine::AVAILABLE_BACKENDS
            .iter()
            .map(|b| format!("\"{}\"", b))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("# Options: {}\n", backends_str));

        out.push_str(&format!("backend = \"{}\"\n", self.backend));

        out
    }
}

/// Backend that handles saving and loading the `Settings` to whatever
/// underlying storage solution. Right now, that's a TOML file on the
/// drive in the current directory, but it could be the browser's
/// local storage, registy or whatnot later.
pub struct Store {
    path: PathBuf,
    toml: TomlDocument,
}

impl Store {
    /// Create a new `Settings` store. It is mapped to a storage
    /// backend that will save the settings. If the backing storage (a
    /// settings TOML file) does not exist, it will be created.
    pub fn new() -> Self {
        let filename = "settings.toml";
        let mut path = std::env::current_exe()
            .or(std::env::current_dir())
            .unwrap_or(PathBuf::new());
        path.set_file_name(filename);
        log::info!("Settings will be stored at: '{}'", path.display());

        let toml = Store::read_settings_toml(&path).unwrap_or_else(|err| {
            log::error!("Could not open settings: {:?}", err);
            log::info!("Falling back to default settings.");
            let toml = Settings::default().as_toml().parse().unwrap();

            log::info!("Creating settings file at: {}", path.display());
            if let Err(err) = Store::write_settings_toml(&path, &toml) {
                log::error!("Could not write settings: {:?}.", err);
            }

            toml
        });

        Self { path, toml }
    }

    pub fn load(&self) -> Settings {
        let mut settings = Settings::default();

        match self.toml["display"].as_str() {
            Some("fullscreen") => settings.fullscreen = true,
            Some("window") => settings.fullscreen = false,
            Some(unexpected) => {
                log::error!("Unknown `display` entry: {}", unexpected);
                log::info!("Valid display entries: \"fullscreen\" or \"window\"");
            }
            None => log::error!("Missing `display` entry."),
        }

        match self.toml["tile_size"].as_integer() {
            Some(tile_size) => {
                let tile_size = tile_size as i32;
                if crate::engine::AVAILABLE_FONT_SIZES.contains(&tile_size) {
                    settings.tile_size = tile_size;
                } else {
                    log::error!("Unsupported `tile_size`: {}", tile_size);
                    log::info!(
                        "Available tile sizes: {:?}",
                        crate::engine::AVAILABLE_FONT_SIZES
                    );
                }
            }
            None => log::error!("Missing `tile_size` entry."),
        }

        match self.toml["backend"].as_str() {
            Some(backend) => {
                if crate::engine::AVAILABLE_BACKENDS.contains(&backend) {
                    settings.backend = backend.into();
                } else {
                    log::error!("Unknown `backend`: {}", backend);
                    log::info!(
                        "Available backends: {:?}",
                        crate::engine::AVAILABLE_BACKENDS
                    );
                }
            }
            None => log::error!("Missing `backend` entry."),
        }

        debug_assert!(settings.valid());

        settings
    }

    fn read_settings_toml(path: &Path) -> Result<TomlDocument, Box<Error>> {
        let mut f = File::open(path)?;
        let mut buffer = String::with_capacity(1000);
        f.read_to_string(&mut buffer)?;
        let toml = buffer.parse::<TomlDocument>()?;

        Ok(toml)
    }

    fn write_settings_toml(path: &Path, toml: &TomlDocument) -> Result<(), Box<Error>> {
        let contents = format!("{}", toml);
        std::fs::write(path, contents)?;
        Ok(())
    }

    pub fn persist(&mut self, settings: &Settings) {
        log::info!("Saving new settings to file {}", self.path.display());
        let display = match settings.fullscreen {
            true => "fullscreen",
            false => "window",
        };
        self.toml["display"] = toml_edit::value(display);

        self.toml["tile_size"] = toml_edit::value(settings.tile_size as i64);

        self.toml["backend"] = toml_edit::value(settings.backend.clone());

        if let Err(err) = Store::write_settings_toml(&self.path, &self.toml) {
            log::error!("Could not write settings to the storage: {:?}", err);
        }
    }
}
