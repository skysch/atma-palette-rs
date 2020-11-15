////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Application entry point.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use atma::command::AtmaOptions;
use atma::Config;
use atma::Palette;
use atma::Settings;
use atma::setup::DEFAULT_CONFIG_PATH;
use atma::utility::normalize_path;

// Standard library imports.
use anyhow::Context;
use anyhow::Error;
use tracing_appender::non_blocking::WorkerGuard;

// External library imports.
use structopt::StructOpt;
use tracing::event;
use tracing::span;
use tracing::Level;


////////////////////////////////////////////////////////////////////////////////
// main
////////////////////////////////////////////////////////////////////////////////
/// The application entry point.
pub fn main() {
    // The worker_guard holds the worker thread handle for the nonblocking
    // trace writer. It should be held until all tracing is complete, as any
    // trace spans or events after it is dropped will be ignored.
    let mut worker_guard: Option<WorkerGuard> = None;

    if let Err(err) = main_facade(&mut worker_guard) {
        // Trace errors without coloring.
        colored::control::set_override(false);
        event!(Level::ERROR, "{:?}", err);

        // Print errors to stderr and exit with error code.
        colored::control::unset_override();
        eprintln!("{:?}", err);
        std::process::exit(1);
    }
}


////////////////////////////////////////////////////////////////////////////////
// main_facade
////////////////////////////////////////////////////////////////////////////////
/// The application facade for propagating user errors.
pub fn main_facade(worker_guard: &mut Option<WorkerGuard>)
    -> Result<(), Error>
{
    // Parse command line options.
    let AtmaOptions { common, command } = AtmaOptions::from_args();

    // Find the path for the config file.
    let cur_dir = std::env::current_dir()?;
    let config_path = match &common.config {
        Some(path) => path.clone(),
        None       => cur_dir.join(DEFAULT_CONFIG_PATH),
    };

    // Load the config file.
    let mut config_load_status = Ok(());
    let mut config = Config::read_from_path(&config_path)
        .with_context(|| format!("Unable to load config file: {:?}", 
            config_path))
        .unwrap_or_else(|e| {
            // Store the error for output until after the logger is configured.
            config_load_status = dbg!(Err(e));
            Config::new().with_load_path(&config_path)
        });
    config.normalize_paths(&cur_dir);

    // Initialize the global tracing subscriber.
    
    *worker_guard = config.trace_config.init_global_default(
        common.verbose,
        common.quiet,
        common.trace)?;
    let span = span!(Level::INFO, "main");
    let _enter = span.enter();

    // Print version information.
    event!(Level::INFO, "Atma version: {}", env!("CARGO_PKG_VERSION"));
    #[cfg(feature = "png")]
    event!(Level::INFO, "PNG support enabled.");
    #[cfg(feature = "termsize")]
    event!(Level::INFO, "Terminal size detection support enabled.");
    let rustc_meta = rustc_version_runtime::version_meta();
    event!(Level::DEBUG, "Rustc version: {} {:?}", rustc_meta.semver, rustc_meta.channel);
    if let Some(hash) = rustc_meta.commit_hash {
        event!(Level::DEBUG, "Rustc git commit: {}", hash);
    }
    event!(Level::DEBUG, "{:#?}", common);
    event!(Level::DEBUG, "{:#?}", command);
    event!(Level::DEBUG, "{:#?}", config);

    // Log any config loading errors.
    match config_load_status {
        Err(e) if common.config.is_some() => {
            // Path is user-specified, so it is an error to now load it.
            return Err(Error::from(e)).with_context(|| format!(
                "Unable to load config file: {:?}",
                config_path));
        },
        Err(_) => {
            // Path is default, so it is ok to use default.
            event!(Level::DEBUG, "Using default config.");
        },

        Ok(_) => (),
    }

    // Find the path for the settings file.
    let cur_dir = std::env::current_dir()?;
    let settings_path = match &common.settings {
        Some(path) => path.clone(),
        None       => cur_dir.join(&config.default_settings_path),
    };

    // Load the settings file.
    let mut settings = match Settings::read_from_path(&settings_path) {
        Err(e) if common.settings.is_some() => {
            // Path is user-specified, so it is an error to now load it.
            return Err(Error::from(e)).with_context(|| format!(
                "Unable to load settings file: {:?}", 
                settings_path));
        },
        Err(_) => {
            // Path is default, so it is ok to use default settings.
            event!(Level::DEBUG, "Using default settings.");
            Settings::new().with_load_path(settings_path)
        },

        Ok(mut settings) => {
            settings.normalize_paths(&cur_dir);
            event!(Level::TRACE, "{:#?}", settings); 
            settings
        },
    };

    // Load the palette.
    let mut palette = if command.requires_palette() {
        match &common.palette {
            Some(pal_path) => {
                let path = normalize_path(cur_dir.clone(), pal_path);
                Some(Palette::read_from_path(&path)
                    .unwrap_or_else(|_| Palette::new().with_load_path(path)))
            },

            None => match &settings.active_palette {
                Some(pal_path) => Some(Palette::read_from_path(&pal_path)?),
                None => if config.load_default_palette {
                    event!(Level::DEBUG, "No specified active palette, loading from default \
                        location.");
                    let default_path = cur_dir.clone()
                        .join(&config.default_palette_path);
                    Palette::read_from_path(&default_path).ok()
                } else {
                    event!(Level::DEBUG, "No active palette.");
                    None
                },
            },
        }
    } else {
        None
    };
    event!(Level::TRACE, "Palette: {:#?}", palette);

    // Dispatch to appropriate commands.
    atma::command::dispatch(
        palette.as_mut(),
        command,
        &common,
        &config,
        &mut settings,
        Some(&cur_dir))?;

    if let Some(pal) = palette {
        if pal.modified() {
            event!(Level::DEBUG, "Palette modified, saving to load path.");
            pal.write_to_load_path()
                .map(|_| ())
                .context("Failed to write palette pile")?;
        }
    }

    if config.modified() {
        event!(Level::DEBUG, "Config modified, saving to load path.");
        config.write_to_load_path()
            .map(|_| ())
            .context("Failed to write config file")?;
    }

    if settings.modified() {
        event!(Level::DEBUG, "Settings modified, saving to load path.");
        settings.write_to_load_path()
            .map(|_| ())
            .context("Failed to write settings file")?;
    }


    Ok(())
}
