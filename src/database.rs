use chrono::{DateTime, Utc};
use rusqlite::{Connection, Error};
use std::path::PathBuf;

pub struct User {
    pub id: i64,
    pub username: String,
}

pub struct Tag {
    pub id: i64,
    //pub name: String,
    pub slug: String,
}

pub struct Discussion {
    pub id: i64,
    pub user_id: i64,
    pub first_post_id: i64,
    pub created_at: DateTime<Utc>,
    pub title: String,
    pub slug: String,
}

pub struct Post {
    pub id: i64,
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub content: String,
}

pub struct Database(Connection);

impl Database {
    pub fn connect(path: PathBuf) -> Result<Self, Error> {
        Ok(Self(Connection::open(path)?))
    }

    pub fn users(&mut self) -> Result<Vec<User>, Error> {
        self.0
            .prepare("SELECT `id`, `username` FROM `users`")?
            .query_map([], |row| {
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                })
            })?
            .collect()
    }

    pub fn tags(&mut self) -> Result<Vec<Tag>, Error> {
        self.0
            .prepare("SELECT `id`, `name`, `slug` FROM `tags`")?
            .query_map([], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    //name: row.get(1)?,
                    slug: row.get(2)?,
                })
            })?
            .collect()
    }

    pub fn discussions(&mut self) -> Result<Vec<Discussion>, Error> {
        self.0.prepare(
            "SELECT `id`, `user_id`, `first_post_id`, `created_at`, `title`, `slug`
                FROM `discussions` WHERE `is_private` <> 1 AND `is_approved` <> 0 AND `hidden_at` IS NULL",
        )?.query_map([], |row| {
            Ok(Discussion {
                id: row.get(0)?,
                user_id: row.get(1)?,
                first_post_id: row.get(2)?,
                created_at: row.get(3)?,
                title: row.get(4)?,
                slug: row.get(5)?,
            })
        })?
        .collect()
    }

    pub fn discussion_tag_ids(&mut self, discussion_id: i64) -> Result<Vec<i64>, Error> {
        self.0
            .prepare("SELECT `tag_id` FROM `discussion_tag` WHERE `discussion_id` = ?")?
            .query_map([discussion_id], |row| row.get(0))?
            .collect()
    }

    pub fn posts(&mut self, discussion_id: i64) -> Result<Vec<Post>, Error> {
        self.0.prepare(
            "SELECT `id`, `user_id`, `created_at`, `edited_at`, `content`
                FROM `posts` WHERE `discussion_id` = ? AND `type` = 'comment' AND `is_private` <> 1 AND `is_approved` <> 0 AND `hidden_at` IS NULL
                ORDER BY `number` ASC",
        )?.query_map([discussion_id], |row| {
            Ok(Post {
                id: row.get(0)?,
                user_id: row.get(1)?,
                created_at: row.get(2)?,
                edited_at: row.get(3)?,
                content: row.get(4)?,
            })
        })?.collect()
    }
}
