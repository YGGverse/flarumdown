mod config;
mod database;

use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use clap::Parser;
use config::Config;
use database::Database;
use log::*;
use regex::{Captures, Regex};
use std::{
    collections::{HashMap, HashSet},
    env::var,
    fs::{File, copy, create_dir_all, read_dir, remove_file},
    io::Write,
    path::PathBuf,
};

pub struct User {
    pub username: String,
}

pub struct Post {
    pub id: i64,
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub content: String,
}

pub struct Discussion {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub title: String,
    pub slug: String,
    pub posts: Vec<Post>,
}

fn main() -> Result<()> {
    if var("RUST_LOG").is_ok() {
        use tracing_subscriber::{EnvFilter, fmt::*};
        struct T;
        impl time::FormatTime for T {
            fn format_time(&self, w: &mut format::Writer<'_>) -> std::fmt::Result {
                write!(w, "{}", Local::now())
            }
        }
        fmt()
            .with_timer(T)
            .with_env_filter(EnvFilter::from_default_env())
            .init()
    }

    let config = Config::parse();

    if !config.target.exists() {
        create_dir_all(&config.target)?;
    }

    let mut db = Database::connect(config.source)?;
    let mut keep = HashSet::with_capacity(1000); // @TODO count entries expected from the DB
    let mut users = HashMap::with_capacity(100); // @TODO count entries expected from the DB
    for user in db.users()? {
        assert!(
            users
                .insert(
                    user.id,
                    User {
                        username: user.username,
                    }
                )
                .is_none()
        )
    }

    let mut tags = HashMap::with_capacity(100); // @TODO count entries expected from the DB
    for tag in db.tags()? {
        if !config.filter_tag.is_empty() && !config.filter_tag.contains(&tag.slug) {
            continue;
        }
        assert!(tags.insert(tag.id, tag.slug).is_none())
    }

    let mut discussions = Vec::with_capacity(1000); // @TODO count entries expected from the DB
    for discussion in db.discussions()? {
        if !db
            .discussion_tag_ids(discussion.id)?
            .iter()
            .any(|id| tags.contains_key(id))
        {
            continue;
        }
        assert!(users.contains_key(&discussion.user_id));

        let mut posts = Vec::with_capacity(1000); // @TODO count entries expected from the DB
        for post in db.posts(discussion.id)? {
            posts.push(Post {
                id: post.id,
                user_id: post.user_id,
                created_at: post.created_at,
                edited_at: post.edited_at,
                content: post.content,
            })
        }
        assert_eq!(discussion.first_post_id, posts.first().unwrap().id);
        discussions.push(Discussion {
            id: discussion.id,
            created_at: discussion.created_at,
            title: discussion.title,
            slug: discussion.slug,
            posts,
        })
    }

    if let Some(index) = config.index {
        let path = {
            let mut path = PathBuf::from(&config.target);
            path.push(format!("{}.md", index.trim_end_matches(".md")));
            path
        };
        let mut file = File::create(&path)?;
        keep.insert(path);
        for discussion in &discussions {
            file.write_all(
                format!(
                    "* [{}]({}.md)\n",
                    discussion
                        .title
                        .replace("[", "\\[")
                        .replace("]", "\\]")
                        .replace("(", "\\(")
                        .replace(")", "\\)"),
                    discussion.id
                )
                .as_bytes(),
            )?;
        }
        let mut footer = Vec::new();
        footer.push("\n---\n".into());
        footer.push(format!("Generated at {}\n", Utc::now()));
        for refer in &config.refer {
            footer.push(format!("* {refer}"));
        }
        file.write_all(footer.join("\n").as_bytes())?
    }

    for discussion in &discussions {
        let path = {
            let mut path = PathBuf::from(&config.target);
            path.push(format!("{}.md", discussion.id));
            path
        };
        let mut file = File::create(&path)?;
        keep.insert(path);
        file.write_all(
            {
                let mut page = Vec::new();
                page.push(format!("# {}\n", discussion.title.trim()));
                page.push({
                    let mut content = Vec::new();
                    for post in &discussion.posts {
                        content.push(format!(
                            "_@{} / {}{}_\n",
                            users.get(&post.user_id).unwrap().username,
                            post.created_at,
                            post.edited_at
                                .map(|edited_at| format!(" / {}", edited_at))
                                .unwrap_or_default()
                        ));
                        let mut uploads = HashSet::new();
                        content.push({
                            let mut post = post_format(
                                pre_format(&post.content, &mut uploads).trim()
                            );
                            for d in &discussions {
                                post = post
                                    .replace(
                                        &format!("](/d/{}-{})", d.id, d.slug),
                                        &format!("]({}.md)", d.id),
                                    )
                                    .replace(
                                        &format!("](d/{}-{})", d.id, d.slug),
                                        &format!("]({}.md)", d.id),
                                    )
                                    .replace(
                                        &format!("]({}-{})", d.id, d.slug),
                                        &format!("]({}.md)", d.id),
                                    )
                                    .replace(&format!("]({})", d.id), &format!("]({}.md)", d.id))
                            }
                            post
                        });
                        for upload in &uploads {
                            let t = {
                                let mut p = PathBuf::from(&config.target);
                                p.push(upload);
                                p
                            };
                            let mut p = PathBuf::from(&config.public);
                            p.push(upload);
                            match p.canonicalize() {
                                Ok(src) => {
                                    if src.starts_with(&config.public) {
                                        if t.exists() {
                                            debug!(
                                                "Copied file `{}` for `{}` exists, skip overwrite",
                                                t.to_string_lossy(),
                                                src.to_string_lossy(),
                                            );
                                        } else {
                                            create_dir_all(t.parent().unwrap())?;
                                            copy(&src, &t)?;
                                            debug!(
                                                "Copied file from `{}` to `{}`",
                                                src.to_string_lossy(),
                                                t.to_string_lossy(),
                                            );
                                            keep.insert(t);
                                        }
                                    } else {
                                        warn!(
                                            "Possible traversal injection: `{}` (post #{}, user #{})",
                                            src.to_string_lossy(),
                                            post.id,
                                            post.user_id
                                        )
                                    }
                                }
                                Err(e) => error!("{e}: `{}` (post #{})", p.to_string_lossy(), post.id)
                            }
                        }
                        content.push("\n---\n".into())
                    }
                    content.push(format!("Generated at {}\n", Utc::now()));
                    for refer in &config.refer {
                        content.push(format!(
                            "* {}/d/{}",
                            refer.trim_end_matches("/"),
                            discussion.id
                        ));
                    }
                    content.join("\n")
                });
                page.join("\n")
            }
            .as_bytes(),
        )?
    }
    cleanup(&config.target, &keep)
}

/// Recursively removes entries that not exists in the `keep` registry
/// * empty directories cleanup yet not implemented @TODO
fn cleanup(target: &PathBuf, keep: &HashSet<PathBuf>) -> Result<()> {
    for entry in read_dir(target)? {
        let p = entry?.path();
        if p.is_file() && !keep.contains(&p) {
            remove_file(&p)?;
            debug!("Cleanup file `{}`", p.to_string_lossy());
        }
    }
    Ok(())
}

fn pre_format(data: &str, uploads: &mut HashSet<PathBuf>) -> String {
    // * keep leading `\s+url` to skip the `thumbnail_url` match
    const R: &str = r#"(?s)<UPL-IMAGE-PREVIEW\s+alt="([^"]*)"\s+.*?\s+url="([^"]*)"\s+[^>]*>[^<]*</UPL-IMAGE-PREVIEW>"#;
    html_escape::decode_html_entities(&strip_tags::strip_tags(
        &Regex::new(R).unwrap().replace_all(data, |c: &Captures| {
            let rel = c[2].trim_start_matches("/").trim_start_matches("d/");
            uploads.insert(rel.into());
            format!(
                "![{}]({rel})",
                c.get(1).map(|s| s.as_str()).unwrap_or_default()
            )
        }),
    ))
    .into()
}

fn post_format(data: &str) -> String {
    Regex::new(r"\n{3,}")
        .unwrap()
        .replace(data, "\n\n")
        .to_string()
}
