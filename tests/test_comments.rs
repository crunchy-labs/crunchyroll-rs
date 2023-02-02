use crate::utils::{Store, SESSION};
use crunchyroll_rs::common::BulkResult;
use crunchyroll_rs::rating::{Comment, CommentFlag, CommentsOptions};
use crunchyroll_rs::Episode;

mod utils;

static COMMENTS: Store<BulkResult<Comment>> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let episode: Episode = crunchy.media_from_id("GRDKJZ81Y", None).await.unwrap();
        Ok(episode.comments(CommentsOptions::default()).await?)
    })
});

#[tokio::test]
async fn comments() {
    assert_result!(COMMENTS.get().await)
}

#[tokio::test]
async fn comment_flag() {
    let mut comments = COMMENTS.get().await.unwrap().clone();
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
