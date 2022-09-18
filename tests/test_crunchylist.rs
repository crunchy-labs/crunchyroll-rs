use crate::utils::{Store, SESSION};
use crunchyroll_rs::list::Crunchylists;
use crunchyroll_rs::Series;

mod utils;

static CRUNCHYLISTS: Store<Crunchylists> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let crunchylists = crunchy.crunchylists().await?;
        Ok(crunchylists)
    })
});

#[tokio::test]
async fn crunchylists() {
    assert_result!(CRUNCHYLISTS.get().await)
}

#[tokio::test]
async fn modify_crunchylist() {
    let crunchylists = CRUNCHYLISTS.get().await.unwrap();

    let new_crunchylist_preview = crunchylists.create("test".to_string()).await;
    assert_result!(new_crunchylist_preview);

    let new_crunchylist = new_crunchylist_preview.unwrap().crunchylist().await;
    assert_result!(new_crunchylist);

    let crunchylist = new_crunchylist.unwrap();

    let series = Series::from_id(SESSION.get().await.unwrap(), "GY8VEQ95Y".to_string())
        .await
        .unwrap();
    let crunchylist_add_result = crunchylist.add(series.into()).await;
    assert_result!(crunchylist_add_result);

    assert_result!(crunchylist.rename("test1".to_string()).await);

    let crunchylist_delete_result = crunchylist.delete().await;
    assert_result!(crunchylist_delete_result);
}
