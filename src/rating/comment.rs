use crate::common::{BulkResult, Image, Pagination};
use crate::{enum_values, options, EmptyJsonProxy, Executor, Locale, Request, Result};
use chrono::{DateTime, Utc};
use futures_util::FutureExt;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use serde_json::json;
use std::sync::Arc;

#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CommentUserAttributesAvatar {
    pub locked: Vec<Image>,
    pub unlocked: Vec<Image>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CommentUserAttributes {
    pub username: String,
    pub avatar: CommentUserAttributesAvatar,
}

/// Information about a user which wrote a [`Comment`].
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CommentUser {
    pub user_key: String,
    pub user_attributes: CommentUserAttributes,

    pub user_flags: Vec<String>,
}

/// Number of votes users gave a [`Comment`].
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CommentVotes {
    #[serde(default)]
    pub like: u32,
    #[serde(default)]
    pub spoiler: u32,
    #[serde(default)]
    pub inappropriate: u32,
}

enum_values! {
    pub enum CommentFlag {
        Like = "like"
        Spoiler = "spoiler"
        Inappropriate = "inappropriate"
    }
}

/// Comment about a episode or movie.
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Comment {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub comment_id: String,
    pub parent_comment_id: Option<String>,
    pub domain_id: String,
    pub guestbook_key: String,

    pub user: CommentUser,
    pub locale: Locale,

    pub message: String,
    #[serde(rename = "flags")]
    #[serde(deserialize_with = "deserialize_flags_to_spoiler")]
    pub is_spoiler: bool,
    /// If populated, this comment is deleted. Crunchyroll still stores deletes comments but without
    /// information what the content was (which means that [`Comment::message`] is not populated if
    /// it's flagged as deleted).
    pub delete_reason: Option<String>,

    pub votes: CommentVotes,
    pub replies_count: u32,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub created: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub modified: DateTime<Utc>,

    pub is_owner: bool,
    #[serde(rename = "user_votes")]
    pub user_flags: Vec<CommentFlag>,
}

impl Comment {
    /// Return all replies to this comment.
    pub fn replies(&self) -> Pagination<Comment> {
        Pagination::new(
            |options| {
                async move {
                    let endpoint = format!(
                        "https://www.crunchyroll.com/talkbox/guestbooks/{}/comments/{}/replies",
                        options.extra.get("guestbook_key").unwrap(),
                        options.extra.get("comment_id").unwrap()
                    );
                    let result: BulkResult<Comment> = options
                        .executor
                        .get(endpoint)
                        .query(&[("page", options.page), ("page_size", options.page_size)])
                        .apply_locale_query()
                        .request()
                        .await?;
                    Ok((result.items, result.total))
                }
                .boxed()
            },
            self.executor.clone(),
            None,
            Some(vec![
                ("guestbook_key", self.guestbook_key.clone()),
                ("comment_id", self.comment_id.clone()),
            ]),
        )
    }

    /// Reply to this comment.
    pub async fn reply<S: AsRef<str>>(&self, message: S, is_spoiler: bool) -> Result<Comment> {
        create_comment(
            &self.executor,
            &self.guestbook_key,
            message.as_ref().to_string(),
            is_spoiler,
            &self.locale,
            Some(&self.comment_id),
        )
        .await
    }

    /// Flag the comment as one of [`CommentFlag`]. The second arguments states if you want to _add_
    /// (`true`) or _remove_ (`false`) a flag. See [`Comment::user_flags`] if you already voted with
    /// the flag you want to use.
    pub async fn flag(&mut self, flag: CommentFlag, add: bool) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/talkbox/guestbooks/{}/comments/{}/votes",
            self.guestbook_key, self.comment_id
        );
        if add {
            self.executor
                .post(endpoint)
                .json(&json!({ "vote_type": &flag }))
                .apply_locale_query()
                .request::<serde_json::Value>()
                .await?;

            match flag {
                CommentFlag::Like => self.votes.like += 1,
                CommentFlag::Spoiler => self.votes.spoiler += 1,
                CommentFlag::Inappropriate => self.votes.inappropriate += 1,
                _ => (),
            }
            self.user_flags.push(flag);
        } else {
            self.executor
                .delete(endpoint)
                .query(&[("vote_type", &flag)])
                .apply_locale_query()
                .request::<EmptyJsonProxy>()
                .await?;

            match flag {
                CommentFlag::Like => self.votes.like -= 1,
                CommentFlag::Spoiler => self.votes.spoiler -= 1,
                CommentFlag::Inappropriate => self.votes.inappropriate -= 1,
                _ => (),
            }
            // `.unwrap()` should be save to call here but if something goes wrong this `Some(...)`
            // check is an extra layer of security
            if let Some(i) = self.user_flags.iter().position(|f| f == &flag) {
                self.user_flags.remove(i);
            }
        }

        Ok(())
    }

    /// Edit this comment. You **must** be the author of it so perform this request. See
    /// [`Comment::is_owner`] if the comment is written by you. If you use this function, its
    /// argument has always be the opposite of [`Comment::is_spoiler`], else a error will occur.
    pub async fn edit(&mut self, spoiler: bool) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/talkbox/guestbooks/{}/comments/{}/flags",
            self.guestbook_key, self.comment_id
        );
        let comment: Comment = self
            .executor
            .patch(endpoint)
            .json(&if spoiler {
                json!({"add": ["spoiler"]})
            } else {
                json!({"remove": ["spoiler"]})
            })
            .apply_locale_query()
            .request()
            .await?;

        *self = comment;

        Ok(())
    }

    /// Delete this comment. You **must** be the author of it so perform this request. See
    /// [`Comment::is_owner`] if the comment is written by you. The comment does not disappear after
    /// this action, its content simply gets blanked out and [`Comment::delete_reason`] is
    /// populated.
    pub async fn delete(&mut self) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/talkbox/guestbooks/{}/comments/{}",
            self.guestbook_key, self.comment_id
        );
        let comment: Comment = self
            .executor
            .delete(endpoint)
            .apply_locale_query()
            .request()
            .await?;

        *self = comment;

        Ok(())
    }
}

enum_values! {
    pub enum CommentSortType {
        Popularity = "popular"
        Newest = "date"
    }
}

options! {
    CommentsOptions;
    sort(CommentSortType, "sort") = Some(CommentSortType::Popularity)
}

macro_rules! impl_comment {
    ($($s:path)*) => {
        $(
            impl $s {
                pub fn comments(&self, options: CommentsOptions) -> Pagination<Comment> {
                    Pagination::new(|options| {
                        async move {
                            let endpoint = format!("https://www.crunchyroll.com/talkbox/guestbooks/{}/comments", options.extra.get("id").unwrap());
                            let result: BulkResult<Comment> = options
                                .executor
                                .get(endpoint)
                                .query(&options.query)
                                .query(&[("page", options.page), ("page_size", options.page_size)])
                                .apply_locale_query()
                                .request()
                                .await?;
                            Ok((result.items, result.total))
                        }
                        .boxed()
                    }, self.executor.clone(), Some(options.into_query()), Some(vec![("id", self.id.clone())]))
                }

                pub async fn comment<S: AsRef<str>>(&self, message: S, is_spoiler: bool) -> Result<Comment> {
                    create_comment(&self.executor, &self.id, message.as_ref().to_string(), is_spoiler, &self.executor.details.locale, None).await
                }
            }
        )*
    }
}

impl_comment! {
    crate::media::Episode crate::media::Movie
}

async fn create_comment<S: AsRef<str>>(
    executor: &Arc<Executor>,
    video_id: &String,
    message: S,
    is_spoiler: bool,
    locale: &Locale,
    parent_id: Option<&String>,
) -> Result<Comment> {
    let endpoint = format!("https://www.crunchyroll.com/talkbox/guestbooks/{video_id}/comments");
    let flags = if is_spoiler { vec!["spoiler"] } else { vec![] };
    executor
        .post(endpoint)
        .json(&if let Some(p_id) = parent_id {
            json!({"message": message.as_ref(), "flags": flags, "locale": locale, "parent_id": p_id})
        } else {
            json!({"message": message.as_ref(), "flags": flags, "locale": locale})
        })
        .apply_locale_query()
        .request()
        .await
}

fn deserialize_flags_to_spoiler<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let mut flags: Vec<String> = Vec::deserialize(deserializer)?;

    for (i, flag) in flags.iter().enumerate() {
        // remove the 'deleted' flag. we already have `Comment::delete_reason` to see if a message
        // is deleted or not
        if flag == "deleted" {
            flags.remove(i);
            break;
        }
    }

    match flags.len() {
        0 => Ok(false),
        1 => {
            if flags.get(0).unwrap() == "spoiler" {
                Ok(true)
            } else {
                Err(Error::custom(format!(
                    "flags has unexpected fields: '{}'",
                    flags.join(", ")
                )))
            }
        }
        _ => Err(Error::custom(format!(
            "flags has too many fields: '{}'",
            flags.join(", ")
        ))),
    }
}
