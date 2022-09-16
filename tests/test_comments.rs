use crate::utils::{Store, SESSION};
use crunchyroll_rs::common::FromId;
use crunchyroll_rs::rating::{Comment, CommentFlag, CommentsOptions};
use crunchyroll_rs::{BulkResult, Episode};
use tokio::sync::Mutex;

mod utils;

static COMMENTS: Store<Mutex<BulkResult<Comment>>> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let episode: Episode = Episode::from_id(crunchy, "GRDKJZ81Y".to_string())
            .await
            .unwrap();
        let comments = episode.comments(CommentsOptions::default()).await?;
        Ok(Mutex::new(comments))
    })
});

#[tokio::test]
async fn comments() {
    assert_result!(COMMENTS.get().await)
}

#[tokio::test]
async fn comment_flag() {
    let mut comments = COMMENTS.get().await.unwrap().lock().await;
    let comment = comments.items.get_mut(0).unwrap();

    assert_result!(
        comment
            .flag(
                CommentFlag::Like,
                !comment.user_flags.contains(&CommentFlag::Like)
            )
            .await
    );
    assert_result!(
        comment
            .flag(
                CommentFlag::Spoiler,
                !comment.user_flags.contains(&CommentFlag::Spoiler)
            )
            .await
    );
    assert_result!(
        comment
            .flag(
                CommentFlag::Inappropriate,
                !comment.user_flags.contains(&CommentFlag::Inappropriate)
            )
            .await
    );
}
