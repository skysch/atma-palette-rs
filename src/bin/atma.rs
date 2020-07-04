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
use atma::AtmaOptions;
use atma::Config;
use atma::DEFAULT_CONFIG_PATH;
use anyhow::Context;
use anyhow::Error;
use atma::logger::Logger;

// External library imports.
use structopt::StructOpt;

use log::*;
use log::LevelFilter;


////////////////////////////////////////////////////////////////////////////////
// main
////////////////////////////////////////////////////////////////////////////////
/// The application entry point.
pub fn main() {
    if let Err(err) = main_facade() {
        // Print errors to stderr and exit with error code.
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

////////////////////////////////////////////////////////////////////////////////
// main_facade
////////////////////////////////////////////////////////////////////////////////
/// The application facade for propagating user errors.
pub fn main_facade() -> Result<(), Error> {
    // Parse command line options.
    let opts = AtmaOptions::from_args();

    // Find the path for the config file.
    let cur_dir = std::env::current_dir()?;
    let config_path = match &opts.common.use_config {
        Some(path) => path.clone(),
        None       => cur_dir.join(DEFAULT_CONFIG_PATH),
    };

    // Load the config file.
    let mut config_load_status = Ok(());
    let mut config = Config::from_path(&config_path)
        .with_context(|| format!("Unable to load config file: {:?}", 
            config_path))
        .unwrap_or_else(|e| {
            // Store the error for output until after the logger is configured.
            config_load_status = Err(e);
            Config::default()
        });
    config.normalize_paths(&cur_dir);

    // Setup and start the global logger.
    let mut logger =  Logger::from_config(config.logger_config.clone());
    for (context, level) in &config.log_levels {
        logger = logger.level_for(context.clone(), *level);
    }
    match (opts.common.verbose, opts.common.quiet, opts.common.trace) {
        (_, _, true) => logger.level_for("atma", LevelFilter::Trace).start(),
        (_, true, _) => (),
        (true, _, _) => logger.level_for("atma", LevelFilter::Debug).start(),
        _            => logger.level_for("atma", LevelFilter::Info).start(),
    }

    // Print version information.
    debug!("Atma version: {}", env!("CARGO_PKG_VERSION"));
    let rustc_meta = rustc_version_runtime::version_meta();
    trace!("Rustc version: {} {:?}", rustc_meta.semver, rustc_meta.channel);
    if let Some(hash) = rustc_meta.commit_hash {
        trace!("Rustc git commit: {}", hash);
    }
    trace!("Options: {:?}", opts);
    trace!("Config: {:?}", config); 


    // Log any config loading errors.
    if let Err(e) = config_load_status { 
        error!("{}", e);
        warn!("Using default config due to previous error.");
    };

    // Dispatch to appropriate commands.
    // use CommandOptions::*;
    // match opts {
    //     Edit { common, in_path } => unimplemented!(),
    // }

    let pal = atma::Palette::default();
    
    pal.write_to_path(&cur_dir.join("test.atma"))?;
    Ok(())
}
