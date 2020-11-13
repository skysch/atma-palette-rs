////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Crate-wide tracing infrastructure.
////////////////////////////////////////////////////////////////////////////////
#![warn(missing_docs)]

// External library imports.
use anyhow::Error;
use anyhow::Context;
use serde::Deserialize;
use serde::Serialize;
use tracing::subscriber::set_global_default;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::FmtSubscriber;

// Standard library imports.
use std::borrow::Cow;


////////////////////////////////////////////////////////////////////////////////
// Constants
////////////////////////////////////////////////////////////////////////////////
/// Default value for default_settings_path.
const DEFAULT_TRACE_ENV_VAR: &'static str = "ATMA_LOG";


////////////////////////////////////////////////////////////////////////////////
// TraceConfig
////////////////////////////////////////////////////////////////////////////////
/// Tracing configuration parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraceConfig {
    /// Trace level filters.
    pub filters: Vec<Cow<'static, str>>,
}


impl TraceConfig {
    /// Initializes the global default tracing subscriber for the using this
    /// configuration.
    pub fn init_global_default(
        &self,
        verbose: bool,
        quiet: bool,
        trace: bool)
        -> Result<(), Error>
    {
        let mut env_filter = EnvFilter::from_env(DEFAULT_TRACE_ENV_VAR);

        for filter in &self.filters[..] {
            let directive = filter
                .parse()
                .with_context(|| format!(
                    "failed to parse trace filter directive \"{:?}\"",
                    filter))?;
            env_filter = env_filter.add_directive(directive);
        }

        let atma_level_filter = match (verbose, quiet, trace) {
            (_, _, true) => LevelFilter::TRACE,
            (_, true, _) => LevelFilter::WARN,
            (true, _, _) => LevelFilter::INFO,
            _            => LevelFilter::WARN,
        };
        env_filter = env_filter.add_directive(atma_level_filter.into());
        

        let subscriber = FmtSubscriber::builder()
            .with_env_filter(env_filter);

        set_global_default(subscriber.finish())
            .with_context(|| format!(
                "failed to set global tracing subscriber"))
    }
}


impl Default for TraceConfig {
    fn default() -> Self {
        TraceConfig {
            filters: vec![
                "atma=INFO".into(),
            ],
        }
    }
}
