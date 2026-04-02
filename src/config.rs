use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(ValueEnum, Debug, Default, Clone)]
pub enum Order {
    #[default]
    Asc,
    Desc,
}

impl std::fmt::Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Asc => write!(f, "ASC"),
            Self::Desc => write!(f, "DESC"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// Path to database
    #[arg(short, long)]
    pub source: PathBuf,

    /// Path to export FoF/upload files from
    /// * the root is path to the public dir (e.g. `/var/www/flarum/public`)
    #[arg(short, long)]
    pub public: PathBuf,

    /// Path to export markdown
    /// * e.g. `/var/www/flarum/public/flarumdown/dump`
    #[arg(short, long)]
    pub target: PathBuf,

    /// Collect discussions with given tag slug only
    /// * keep empty to export all
    #[arg(short, long)]
    pub filter_tag: Vec<String>,

    /// Generate index file with given name
    #[arg(short, long)]
    pub index: Option<String>,

    /// Append time (created) to `index` entries in given format
    /// * tip: escape with `%%d/%%m/%%Y %%H:%%M` when using CLI/bash argument
    #[arg(long)]
    pub index_time_created: Option<String>,

    /// Order entries by ID
    /// * useful as the `index` new / old records first
    #[arg(short, long)]
    pub order: Order,

    /// Append reference to original source (mirrors)
    #[arg(short, long)]
    pub refer: Vec<String>,
}
