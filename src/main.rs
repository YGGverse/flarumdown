mod config;
mod database;

use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use clap::Parser;
use config::Config;
use database::Database;
use html_to_markdown_rs::convert;
use log::*;
use regex::{Captures, Regex};
use std::{
    collections::{HashMap, HashSet},
    env::var,
    fs::{File, copy, create_dir_all, remove_dir_all},
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

    if config.target.exists() {
        remove_dir_all(&config.target)?
    }
    create_dir_all(&config.target)?;

    let mut db = Database::connect(config.source)?;

    let mut users = HashMap::new();
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

    let mut tags = HashMap::new();
    for tag in db.tags()? {
        if !config.filter_tag.is_empty() && !config.filter_tag.contains(&tag.slug) {
            continue;
        }
        assert!(tags.insert(tag.id, tag.slug).is_none())
    }

    let mut discussions = Vec::new();
    for discussion in db.discussions()? {
        if !db
            .discussion_tag_ids(discussion.id)?
            .iter()
            .any(|id| tags.contains_key(id))
        {
            continue;
        }
        assert!(users.contains_key(&discussion.user_id));

        let mut posts = Vec::new();
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
            posts,
        })
    }

    if let Some(index) = config.index {
        let mut file = File::create_new({
            let mut path = PathBuf::from(&config.target);
            path.push(format!("{}.md", index.trim_end_matches(".md")));
            path
        })?;
        for discussion in &discussions {
            file.write_all(format!("* [{}]({}.md)\n", discussion.title, discussion.id).as_bytes())?;
        }
    }

    for discussion in discussions {
        let mut file = File::create_new({
            let mut path = PathBuf::from(&config.target);
            path.push(format!("{}.md", discussion.id));
            path
        })?;
        file.write_all(
            {
                let mut page = Vec::new();
                page.push(format!("# {}\n", discussion.title.trim()));
                page.push({
                    let mut content = Vec::new();
                    for post in discussion.posts {
                        content.push(format!(
                            "_@{} / {}{}_\n",
                            users.get(&post.user_id).unwrap().username,
                            post.created_at,
                            post.edited_at
                                .map(|edited_at| format!(" / {}", edited_at))
                                .unwrap_or_default()
                        ));
                        let mut uploads = HashSet::new();
                        content.push(post_format(&convert(
                            pre_format(&post.content, &mut uploads).trim(),
                            None,
                        )?));
                        for upload in &uploads {
                            let path_source = {
                                let mut p = PathBuf::from(&config.upload);
                                p.push(upload);
                                p
                            };
                            let path_target = {
                                let mut p = PathBuf::from(&config.target);
                                p.push(upload);
                                p
                            };
                            let path_parent = path_target.parent().unwrap();

                            create_dir_all(path_parent)?;
                            if !path_target.exists() {
                                if path_source.exists() {
                                    copy(path_source, path_target)?;
                                } else {
                                    warn!(
                                        "Source file does not exist: `{}`",
                                        path_source.to_string_lossy()
                                    )
                                }
                            }
                        }
                        content.push("---\n".into())
                    }
                    content.join("\n")
                });
                page.join("\n")
            }
            .as_bytes(),
        )?
    }

    Ok(())
}

fn pre_format(data: &str, uploads: &mut HashSet<PathBuf>) -> String {
    Regex::new(r"<e>[^<]+</e>")
        .unwrap()
        .replace_all(
            &Regex::new(r"<s>[^<]+</s>").unwrap().replace_all(
                &Regex::new(r"(?s)<UPL-IMAGE-PREVIEW([^>]+)>\[[^\]]+\]</UPL-IMAGE-PREVIEW>")
                    .unwrap()
                    .replace_all(data, |c: &Captures| {
                        uploads.insert(
                            Regex::new(r#"url="([^"]+)""#)
                                .unwrap()
                                .captures(&c[1])
                                .unwrap()[1]
                                .trim_start_matches("/")
                                .into(),
                        );
                        format!("<img{}>", c[1].replace(" url=", " src="))
                    }),
                "",
            ),
            "",
        )
        .replace("<C", "<code")
        .replace("</C>", "</code>")
        .replace("<LIST", "<ul")
        .replace("</LIST>", "</ul>")
        .replace("<URL", "<a")
        .replace("</URL>", "</a>")
        .replace(" url=", " href=")
        .replace("<r>", "")
        .replace("</r>", "")
}

fn post_format(data: &str) -> String {
    Regex::new(r"\n{3,}")
        .unwrap()
        .replace(data, "\n\n")
        .to_string()
}
