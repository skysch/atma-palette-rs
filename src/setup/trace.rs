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
use anyhow::Context;
use anyhow::Error;
use serde::Deserialize;
use serde::Serialize;
use tracing::subscriber::set_global_default;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Registry;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::filter::LevelFilter;
// use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::FmtSubscriber;
use tracing_appender::non_blocking::WorkerGuard;

// Standard library imports.
use std::borrow::Cow;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::path::Path;


////////////////////////////////////////////////////////////////////////////////
// Constants
////////////////////////////////////////////////////////////////////////////////
/// Default value for the tracing environment variable.
const DEFAULT_TRACE_ENV_VAR: &'static str = "ATMA_LOG";

/// Default value for ansi_colors.
const DEFAULT_ANSI_COLORS: bool = true;



////////////////////////////////////////////////////////////////////////////////
// TraceConfig
////////////////////////////////////////////////////////////////////////////////
/// Tracing configuration parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraceConfig {
    /// Trace level filters.
    #[serde(default = "TraceConfig::default_filters")]
    pub filters: Vec<Cow<'static, str>>,
    
    /// The trace output path. If None, the trace will not be written to file.
    #[serde(default = "TraceConfig::default_output_path")]
    pub output_path: Option<PathBuf>,

    /// Whether to write trace output to stdout.
    pub output_stdout: bool,

    /// Whether to use ANSI coloring in the output.
    #[serde(default = "TraceConfig::default_ansi_colors")]
    pub ansi_colors: bool,

}


impl TraceConfig {
    /// Initializes the global default tracing subscriber for the using this
    /// configuration.

    pub fn init_global_default<L>(
        &self,
        default_level_filter: L)
        -> Result<Option<WorkerGuard>, Error>
        where L: Into<LevelFilter>
    {
        if self.output_path.is_none() && !self.output_stdout {
            return Ok(None);
        }

        let mut env_filter = EnvFilter::from_env(DEFAULT_TRACE_ENV_VAR)
            .add_directive(default_level_filter.into().into());
        
        for filter in &self.filters[..] {
            let directive = filter
                .parse()
                .with_context(|| format!(
                    "failed to parse trace filter directive \"{:?}\"",
                    filter))?;
            env_filter = env_filter.add_directive(directive);
        }

        let fmt_layer = Layer::new()
            .without_time()
            .with_ansi(self.ansi_colors);

        match (self.output_stdout, &self.output_path) {
            (true, Some(output_path)) => {
                let path: &Path = output_path.as_ref();
                let file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(path)
                    .with_context(|| format!(
                        "Failed to create/open log file for writing: {}",
                        path.display()))?;
                let (writer, guard) = tracing_appender::non_blocking(file);

                let subscriber = Registry::default()
                    .with(env_filter)
                    .with(fmt_layer)
                    .with(Layer::new()
                        .without_time()
                        .with_ansi(false)
                        .with_writer(writer));

                set_global_default(subscriber)
                    .with_context(|| format!(
                        "failed to set global tracing subscriber"))?;

                Ok(Some(guard))
            },

            (false, Some(output_path)) => {
                let path: &Path = output_path.as_ref();
                let file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(path)
                    .with_context(|| format!(
                        "Failed to create/open log file for writing: {}",
                        path.display()))?;
                let (writer, guard) = tracing_appender::non_blocking(file);

                let subscriber = Registry::default()
                    .with(env_filter)
                    .with(Layer::new()
                        .without_time()
                        .with_ansi(false)
                        .with_writer(writer));

                set_global_default(subscriber)
                    .with_context(|| format!(
                        "failed to set global tracing subscriber"))?;

                Ok(Some(guard))
            },

            (true, None) => {
                let subscriber = Registry::default()
                    .with(env_filter)
                    .with(fmt_layer);

                set_global_default(subscriber)
                    .with_context(|| format!(
                        "failed to set global tracing subscriber"))?;
                Ok(None)
            },

            _ => unreachable!(),
        }
    }


    /// Returns the default value for filters.
    #[inline(always)]
    fn default_filters() -> Vec<Cow<'static, str>> {
        vec![
            "atma=WARN".into(),
        ]
    }

    /// Returns the default value for output_path.
    #[inline(always)]
    fn default_output_path() -> Option<PathBuf> {
        None
    }

    /// Returns the default value for ansi_colors.
    #[inline(always)]
    fn default_ansi_colors() -> bool {
        DEFAULT_ANSI_COLORS
    }
}


impl Default for TraceConfig {
    fn default() -> Self {
        TraceConfig {
            filters: TraceConfig::default_filters(),
            output_path: TraceConfig::default_output_path(),
            output_stdout: false,
            ansi_colors: TraceConfig::default_ansi_colors(),
        }
    }
}
