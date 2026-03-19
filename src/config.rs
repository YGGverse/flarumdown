use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// Path to database
    #[arg(short, long)]
    pub source: PathBuf,

    /// Path to export FoF/upload tags from
    /// * tip: root should be the path to `flarum/public` dir
    #[arg(short, long)]
    pub upload: PathBuf,

    /// Path to export markdown
    #[arg(short, long)]
    pub target: PathBuf,

    /// Collect discussions with given tag slug only
    /// * keep empty to export all
    #[arg(short, long)]
    pub filter_tag: Vec<String>,

    /// Generate index file with given name
    #[arg(short, long)]
    pub index: Option<String>,

    /// Append reference to original source (mirrors)
    #[arg(short, long)]
    pub refer: Vec<String>,
}
