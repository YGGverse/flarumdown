mod config;
mod database;

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Parser;
use config::Config;
use database::Database;
use html_to_markdown_rs::convert;
use std::{
    collections::HashMap,
    fs::{File, create_dir_all, remove_dir_all},
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

    for discussion in discussions {
        let mut file = File::create_new({
            let mut path = PathBuf::from(&config.target);
            path.push(format!("{}.md", discussion.id));
            path
        })?;
        file.write_all(
            {
                let mut page = Vec::new();
                page.push(format!("# {}", discussion.title));
                page.push({
                    let mut content = Vec::new();
                    for post in discussion.posts {
                        content.push(format!(
                            "@{} / {}{}",
                            users.get(&post.user_id).unwrap().username,
                            post.created_at,
                            post.edited_at
                                .map(|edited_at| format!(" / {}", edited_at))
                                .unwrap_or_default()
                        ));
                        content.push("---".into());
                        content.push(convert(&post.content, None)?)
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
