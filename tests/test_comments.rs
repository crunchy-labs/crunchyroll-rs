use crate::utils::{Store, SESSION};
use crunchyroll_rs::rating::{Comment, CommentFlag, CommentsOptions};
use crunchyroll_rs::Episode;
use futures_util::StreamExt;

mod utils;

static COMMENT: Store<Comment> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let episode: Episode = crunchy.media_from_id("GRDKJZ81Y").await.unwrap();
        Ok(episode
            .comments(CommentsOptions::default())
            .next()
            .await
            .unwrap()?)
    })
});

#[tokio::test]
async fn comments() {
    assert_result!(COMMENT.get().await)
}

#[tokio::test]
async fn comment_flag() {
    let mut comment = COMMENT.get().await.unwrap().clone();

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
