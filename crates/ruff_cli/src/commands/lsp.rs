use std::fmt::{Display, Formatter};
use std::io;
use std::num::NonZeroU16;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Result;
use colored::Colorize;
use rayon::iter::Either::{Left, Right};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use thiserror::Error;
use tracing::metadata::Level;
use tracing::{debug, warn};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, Registry};
use tracing_tree::time::Uptime;

use ruff::fs;
use ruff::logging::LogLevel;
use ruff::settings::types::PreviewMode;
use ruff::warn_user_once;
use ruff_formatter::LineWidth;
use ruff_python_ast::{PySourceType, SourceType};
use ruff_python_formatter::{format_module, FormatModuleError, PyFormatOptions};
use ruff_source_file::{find_newline, LineEnding};
use ruff_workspace::resolver::python_files_in_path;

use crate::args::{FormatArguments, LspArguments, LspCommand, Overrides};
use crate::resolve::resolve;
use crate::ExitStatus;

/// Format a set of files, and return the exit status.
pub(crate) fn lsp(arguments: LspCommand, log_level: LogLevel) -> Result<ExitStatus> {
    let subscriber = Registry::default().with(
        tracing_tree::HierarchicalLayer::default()
            .with_indent_lines(true)
            .with_indent_amount(2)
            .with_bracketed_fields(true)
            .with_targets(true)
            .with_writer(|| Box::new(std::io::stderr()))
            .with_timer(Uptime::default())
            .with_filter(LevelFilter::from(Some(Level::DEBUG))),
    );
    tracing::subscriber::with_default(subscriber, || {
        ruff_lsp::stdio();
    });

    Ok(ExitStatus::Success)
}
