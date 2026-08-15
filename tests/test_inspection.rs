use google_indexing_api::{GoogleIndexingApi, UrlNotificationsType};
use yup_oauth2::AccessToken;

async fn test_token() -> AccessToken {
    // 認証
    let secret = yup_oauth2::read_service_account_key("./test.json")
        .await
        .expect("test.json");
    let auth = yup_oauth2::ServiceAccountAuthenticator::builder(secret)
        .build()
        .await
        .unwrap();
    let scopes = &["https://www.googleapis.com/auth/indexing"];

    let token = auth.token(scopes).await;
    assert!(token.is_ok(), "{}", token.err().unwrap().to_string());
    token.unwrap()
}

// 実際の Google Indexing API を叩くライブテスト。
// リポジトリ直下にサービスアカウント鍵 test.json が必要なため、通常の cargo test では実行しない。
// 実行方法: cargo test -- --ignored
#[tokio::test]
#[ignore]
async fn test_sitemaps() {
    let token = test_token().await;
    let a = GoogleIndexingApi::url_notifications()
        .batch(
            token.token().unwrap(),
            vec![
                "http://example.com/widgets/1".to_string(),
                "http://example.com/widgets/2".to_string(),
            ],
            UrlNotificationsType::UPDATED,
        )
        .await;
    assert!(a.is_ok(), "{}", a.err().unwrap().to_string());

    for value in a.unwrap() {
        println!("{} {:?}", value.url(), value.json());
    }
}
