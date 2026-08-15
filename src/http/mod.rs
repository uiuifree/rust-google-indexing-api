use crate::error::GoogleApiError;
use crate::{ResponseGoogleIndexingBatch, UrlNotificationsType};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

#[derive(Default, Debug)]
pub(crate) struct HttpClient {}

impl HttpClient {
    pub async fn get<T>(token: &str, url: &str) -> Result<T, GoogleApiError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let mut response = reqwest::Client::new()
            .get(url.to_string())
            .header("Authorization", format!("Bearer {}", token));

        response = response.header("Accept", "application/json");

        let response = response.send().await;

        if response.is_err() {
            return Err(GoogleApiError::Connection(
                response.err().unwrap().to_string(),
            ));
        }
        let response = response.unwrap();
        let status = response.status();
        let value = response.text().await;
        if !status.is_success() {
            return Err(GoogleApiError::HttpStatus(
                status.as_u16(),
                value.unwrap_or_default(),
            ));
        }
        if value.is_err() {
            return Err(GoogleApiError::JsonParse(value.err().unwrap().to_string()));
        }
        let value = value.unwrap();
        let parse = serde_json::from_str(value.as_str());
        if parse.is_err() {
            return Err(GoogleApiError::JsonParse(value));
        }

        Ok(parse.unwrap())
    }
    pub async fn post<T, U>(token: &str, url: &str, params: U) -> Result<T, GoogleApiError>
    where
        T: for<'de> serde::Deserialize<'de>,
        U: serde::Serialize + std::fmt::Debug,
    {
        let mut response = reqwest::Client::new().post(url.to_string());
        if !token.is_empty() {
            response = response.header("Authorization", format!("Bearer {}", token))
        }
        let response = response.json(&json!(params)).send().await;

        if response.is_err() {
            return Err(GoogleApiError::Connection(
                response.err().unwrap().to_string(),
            ));
        }
        let response = response.unwrap();
        let status = response.status();
        let value = response.text().await;
        if !status.is_success() {
            return Err(GoogleApiError::HttpStatus(
                status.as_u16(),
                value.unwrap_or_default(),
            ));
        }
        if value.is_err() {
            return Err(GoogleApiError::JsonParse(value.err().unwrap().to_string()));
        }
        let value = value.unwrap();
        let parse = serde_json::from_str(value.as_str());
        if parse.is_err() {
            return Err(GoogleApiError::JsonParse(value));
        }

        Ok(parse.unwrap())
    }

    pub async fn execute(
        token: &str,
        urls: Vec<String>,
        url_type: UrlNotificationsType,
    ) -> Result<Vec<ResponseGoogleIndexingBatch>, GoogleApiError> {
        Self::execute_url(
            "https://indexing.googleapis.com/batch",
            token,
            urls,
            url_type,
        )
        .await
    }

    // endpoint を差し替え可能にしているのはモックサーバでテストするため
    async fn execute_url(
        endpoint: &str,
        token: &str,
        urls: Vec<String>,
        url_type: UrlNotificationsType,
    ) -> Result<Vec<ResponseGoogleIndexingBatch>, GoogleApiError> {
        // バッチAPIの上限は100件
        if urls.is_empty() || urls.len() > 100 {
            return Err(GoogleApiError::InvalidArgument(format!(
                "batch accepts 1 to 100 urls, got {}",
                urls.len()
            )));
        }
        let (key_values, send_body) = build_batch_request_body(&urls, &url_type);
        // リクエストの送信とレスポンスの取得
        let response = reqwest::Client::new()
            .post(endpoint)
            .header(
                CONTENT_TYPE,
                format!("multipart/mixed; boundary={}", BATCH_BOUNDARY),
            )
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .body(send_body)
            .send()
            .await;

        if response.is_err() {
            return Err(GoogleApiError::Connection(
                response.err().unwrap().to_string(),
            ));
        }
        let response = response.unwrap();

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(GoogleApiError::HttpStatus(status.as_u16(), body));
        }
        let headers = response.headers();

        let mut content_type = "".to_string();
        if let Some(value) = headers.get("Content-Type") {
            content_type = match value.to_str() {
                Ok(v) => v.to_string(),
                Err(e) => {
                    return Err(GoogleApiError::JsonParse(format!(
                        "batch response Content-Type header is not valid text: {}",
                        e
                    )))
                }
            };
        }

        // レスポンスのボディの読み取り
        let body = response.text().await;
        if body.is_err() {
            return Err(GoogleApiError::Connection(body.err().unwrap().to_string()));
        }
        let body = body.unwrap();
        let boundary = get_boundary(content_type.as_str());
        if boundary.is_empty() {
            return Err(GoogleApiError::JsonParse(format!(
                "batch response is not multipart/mixed with a boundary: Content-Type=\"{}\"",
                content_type
            )));
        }

        let mut batch_response = vec![];

        let boundary_bodies = body_boundary_split(body.as_str(), boundary.as_str());
        if boundary_bodies.is_empty() {
            return Err(GoogleApiError::JsonParse(format!(
                "batch response body has no closing boundary: {}",
                body
            )));
        }
        let mut seen_ids = HashSet::new();
        for boundary_body in boundary_bodies {
            let http = plane_http_to_response(boundary_body.as_str());
            let mut http_url = None;
            for (id, url) in &key_values {
                if id == &http.content_id {
                    http_url = Some(url.to_string());
                    break;
                }
            }
            // リクエストのどの Content-ID とも一致しなければ、結果と URL の対応が取れない
            let Some(http_url) = http_url else {
                return Err(GoogleApiError::JsonParse(format!(
                    "batch response has an unknown Content-ID \"{}\": {}",
                    http.content_id, body
                )));
            };
            if !seen_ids.insert(http.content_id.to_string()) {
                return Err(GoogleApiError::JsonParse(format!(
                    "batch response has a duplicated Content-ID \"{}\": {}",
                    http.content_id, body
                )));
            }
            if http.status_code == 0 {
                return Err(GoogleApiError::JsonParse(format!(
                    "batch response part has no parsable HTTP status line: {}",
                    boundary_body
                )));
            }
            batch_response.push(ResponseGoogleIndexingBatch {
                url: http_url,
                status_code: http.status_code,
                value: http.content,
            });
        }
        if batch_response.len() != urls.len() {
            return Err(GoogleApiError::JsonParse(format!(
                "batch response has {} parts but {} urls were requested: {}",
                batch_response.len(),
                urls.len(),
                body
            )));
        }

        Ok(batch_response)
    }
}
// マルチパートフォームデータのバウンダリー
const BATCH_BOUNDARY: &str = "===============7330845974216740156==";

// バッチ用の multipart/mixed リクエストボディを組み立てる。
// 戻り値は (Content-ID と URL の対応表, ボディ文字列)
fn build_batch_request_body(
    urls: &[String],
    url_type: &UrlNotificationsType,
) -> (Vec<(String, String)>, String) {
    let boundary2 = format!("--{}", BATCH_BOUNDARY);
    fn make_row(index: isize, url: &str, url_type: &str) -> (String, String, String) {
        let body = json!({
            "url":url,
            "type":url_type,
        })
        .to_string();
        let id = format!("b29c5de2-0db4-490b-b421-6a51b598bd23+{}", index + 1);
        (
            id.to_string(),
            url.to_string(),
            [
                "Content-Type: application/http",
                "Content-Transfer-Encoding: binary",
                format!("Content-ID: <{}>", id).as_str(),
                "",
                "POST /v3/urlNotifications:publish HTTP/1.1",
                "Content-Type: application/json",
                "accept: application/json",
                format!("content-length: {}", body.len()).as_str(),
                "",
                body.as_str(),
            ]
            .join("\r\n"),
        )
    }

    let send_data1 = urls
        .iter()
        .enumerate()
        .map(|(i, url)| make_row(i as isize, url, url_type.to_string().as_str()))
        .collect::<Vec<(String, String, String)>>();

    let key_values = send_data1
        .clone()
        .iter()
        .map(|q| (q.0.to_string(), q.1.to_string()))
        .collect::<Vec<(String, String)>>();
    let send_body = send_data1
        .iter()
        .map(|q| q.2.to_string())
        .collect::<Vec<String>>()
        .join(format!("\r\n{}\r\n", boundary2).as_str());
    // マルチパートフォームデータのテキスト部分。終了バウンダリーは末尾に "--" が付く
    let text_parts = [
        boundary2.to_string(),
        send_body,
        format!("{}--", boundary2),
        "".to_string(),
    ];
    (key_values, text_parts.join("\r\n"))
}

fn get_boundary(value: &str) -> String {
    if !value.contains("multipart/mixed") {
        return "".to_string();
    }
    if !value.contains("boundary=") {
        return "".to_string();
    }
    let d = value.split("boundary=").collect::<Vec<&str>>();
    // 後続のパラメータと引用符を取り除く
    let boundary = d.get(1).unwrap().split(';').next().unwrap_or_default();
    boundary.trim().trim_matches('"').to_string()
}

fn plane_http_to_response(content: &str) -> HttpResponse {
    let delimiter = "\r\n\r\n";
    // HeaderとBodyに分割
    let (header, body) = split_one(content, delimiter);
    let mut http = header_from_plane_text(header.as_str());
    let content_type = http.content_type();
    if content_type.contains("application/http") {
        let mut content_id = http.content_id.to_string();
        if let Some(value) = http.header.get("Content-ID") {
            content_id = value
                .trim_start_matches("<response-")
                .trim_end_matches(">")
                .to_string();
        }
        let mut http = plane_http_to_response(body.as_str());
        http.content_id = content_id;
        return http;
    } else {
        http.content = body;
    }
    http
}

fn split_one(value: &str, delimiter: &str) -> (String, String) {
    let mut header = "".to_string();
    let mut body_vec = vec![];
    let mut is_header = true;

    for row in value.split(delimiter).collect::<Vec<&str>>() {
        if is_header {
            header = row.to_string();
            is_header = false;
            continue;
        }
        body_vec.push(row.to_string());
    }
    (header, body_vec.join(delimiter).to_string())
}

fn header_from_plane_text(value: &str) -> HttpResponse {
    let mut header = HashMap::new();
    let rows = value.split("\n");

    let mut response = HttpResponse::default();
    for row in rows {
        if row.starts_with("HTTP/1.1") {
            let tmp_status = row
                .split(" ")
                .map(|q| q.to_string())
                .collect::<Vec<String>>();
            if tmp_status.get(2).is_some() {
                response.status_code = tmp_status.get(1).unwrap().parse().unwrap_or_default();
                response.status_name = tmp_status.get(2).unwrap().parse().unwrap_or_default();
            }
            continue;
        }
        let (key, value) = split_one(row, ":");
        header.entry(key).or_insert(value.trim().to_string());
    }
    response.header = header;
    response
}

fn body_boundary_split(content: &str, boundary: &str) -> Vec<String> {
    let end_boundary = format!("--{}--", boundary);
    if !content.contains(end_boundary.as_str()) {
        return vec![];
    }
    let content = content
        .split(end_boundary.as_str())
        .collect::<Vec<&str>>()
        .first()
        .unwrap()
        .to_string();

    content
        .split(format!("--{}", boundary).as_str())
        .map(|q| q.trim().to_string())
        .filter(|q| !q.is_empty())
        .collect::<Vec<String>>()
}

#[derive(Debug, Default)]
struct HttpResponse {
    content_id: String,
    status_code: u16,
    status_name: String,
    header: HashMap<String, String>,
    content: String,
}

impl HttpResponse {
    fn content_type(&self) -> String {
        match self.header.get("Content-Type") {
            Some(value) => value.to_string(),
            None => "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    const BOUNDARY: &str = "batch_abc123";
    // make_row が生成する Content-ID の接頭辞と同じ値
    const REQUEST_ID_PREFIX: &str = "b29c5de2-0db4-490b-b421-6a51b598bd23";

    fn batch_part_with_id(content_id: &str, status_line: &str, json: &str) -> String {
        format!(
            "--{}\r\nContent-Type: application/http\r\nContent-ID: <response-{}>\r\n\r\nHTTP/1.1 {}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{}\r\n",
            BOUNDARY, content_id, status_line, json
        )
    }

    fn batch_part(index: usize, status_line: &str, json: &str) -> String {
        batch_part_with_id(
            format!("{}+{}", REQUEST_ID_PREFIX, index).as_str(),
            status_line,
            json,
        )
    }

    fn batch_response_body() -> String {
        format!(
            "{}{}--{}--\r\n",
            batch_part(
                1,
                "200 OK",
                r#"{"urlNotificationMetadata":{"url":"http://example.com/widgets/1"}}"#
            ),
            batch_part(2, "403 Forbidden", r#"{"error":{"code":403}}"#),
            BOUNDARY
        )
    }

    #[test]
    fn test_get_boundary() {
        assert_eq!(
            get_boundary("multipart/mixed; boundary=batch_abc123"),
            "batch_abc123"
        );
        // 引用符付き・後続パラメータ付きでも取り出せる
        assert_eq!(
            get_boundary(r#"multipart/mixed; boundary="batch_abc123""#),
            "batch_abc123"
        );
        assert_eq!(
            get_boundary("multipart/mixed; boundary=batch_abc123; charset=UTF-8"),
            "batch_abc123"
        );
        assert_eq!(get_boundary("application/json"), "");
        assert_eq!(get_boundary("multipart/mixed"), "");
    }

    #[test]
    fn test_batch_request_body_ends_with_closing_boundary() {
        let (key_values, body) = build_batch_request_body(
            &["http://example.com/widgets/1".to_string()],
            &UrlNotificationsType::UPDATED,
        );
        assert_eq!(key_values.len(), 1);
        assert!(
            body.trim_end()
                .ends_with("--===============7330845974216740156==--"),
            "終了バウンダリーは --boundary-- で終わるべき: {}",
            body
        );
    }

    #[test]
    fn test_split_one() {
        let (head, rest) = split_one("Content-ID: <response-1>: x", ":");
        assert_eq!(head, "Content-ID");
        assert_eq!(rest, " <response-1>: x");
    }

    #[test]
    fn test_header_from_plane_text() {
        let response =
            header_from_plane_text("HTTP/1.1 404 Not Found\r\nContent-Type: application/json");
        assert_eq!(response.status_code, 404);
        assert_eq!(
            response.header.get("Content-Type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_body_boundary_split() {
        let chunks = body_boundary_split(batch_response_body().as_str(), BOUNDARY);
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].contains("HTTP/1.1 200 OK"));
        assert!(chunks[1].contains("HTTP/1.1 403 Forbidden"));

        // 終端バウンダリーが無ければ何も返さない
        assert!(body_boundary_split("--batch_abc123\r\nfoo", BOUNDARY).is_empty());
    }

    #[test]
    fn test_plane_http_to_response() {
        let chunks = body_boundary_split(batch_response_body().as_str(), BOUNDARY);
        let http = plane_http_to_response(chunks[0].as_str());
        assert_eq!(http.content_id, format!("{}+1", REQUEST_ID_PREFIX));
        assert_eq!(http.status_code, 200);
        assert_eq!(
            http.content,
            r#"{"urlNotificationMetadata":{"url":"http://example.com/widgets/1"}}"#
        );
    }

    #[tokio::test]
    async fn test_get_success() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET)
                    .path("/metadata")
                    .header("Authorization", "Bearer test-token");
                then.status(200).body(r#"{"url":"http://example.com/"}"#);
            })
            .await;

        let value: serde_json::Value =
            HttpClient::get("test-token", server.url("/metadata").as_str())
                .await
                .unwrap();
        mock.assert_async().await;
        assert_eq!(value["url"], "http://example.com/");
    }

    #[tokio::test]
    async fn test_get_http_error_returns_status_and_body() {
        let server = MockServer::start_async().await;
        server
            .mock_async(|when, then| {
                when.method(GET).path("/metadata");
                then.status(403).body(r#"{"error":{"code":403}}"#);
            })
            .await;

        let result: Result<serde_json::Value, GoogleApiError> =
            HttpClient::get("test-token", server.url("/metadata").as_str()).await;
        match result {
            Err(GoogleApiError::HttpStatus(status, body)) => {
                assert_eq!(status, 403);
                assert!(body.contains("403"));
            }
            other => panic!("HttpStatus であるべき: {:?}", other.err()),
        }
    }

    #[tokio::test]
    async fn test_get_broken_json() {
        let server = MockServer::start_async().await;
        server
            .mock_async(|when, then| {
                when.method(GET).path("/metadata");
                then.status(200).body("not-json");
            })
            .await;

        let result: Result<serde_json::Value, GoogleApiError> =
            HttpClient::get("test-token", server.url("/metadata").as_str()).await;
        assert!(matches!(result, Err(GoogleApiError::JsonParse(_))));
    }

    #[tokio::test]
    async fn test_get_connection_error() {
        // 不正な URL はタイムアウトを待たず即時に Connection エラーになる
        let result: Result<serde_json::Value, GoogleApiError> =
            HttpClient::get("test-token", "not-a-url").await;
        assert!(matches!(result, Err(GoogleApiError::Connection(_))));
    }

    #[tokio::test]
    async fn test_post_success() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/publish")
                    .header("Authorization", "Bearer test-token")
                    .body_contains("http://example.com/");
                then.status(200).body(r#"{"ok":true}"#);
            })
            .await;

        let value: serde_json::Value = HttpClient::post(
            "test-token",
            server.url("/publish").as_str(),
            json!({"url": "http://example.com/"}),
        )
        .await
        .unwrap();
        mock.assert_async().await;
        assert_eq!(value["ok"], true);
    }

    #[tokio::test]
    async fn test_post_http_error_returns_status_and_body() {
        let server = MockServer::start_async().await;
        server
            .mock_async(|when, then| {
                when.method(POST).path("/publish");
                then.status(429).body("rate limit");
            })
            .await;

        let result: Result<serde_json::Value, GoogleApiError> = HttpClient::post(
            "test-token",
            server.url("/publish").as_str(),
            json!({"url": "http://example.com/"}),
        )
        .await;
        match result {
            Err(GoogleApiError::HttpStatus(status, body)) => {
                assert_eq!(status, 429);
                assert_eq!(body, "rate limit");
            }
            other => panic!("HttpStatus であるべき: {:?}", other.err()),
        }
    }

    #[tokio::test]
    async fn test_execute_batch() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/batch")
                    .header("Authorization", "Bearer test-token")
                    .body_contains("http://example.com/widgets/1")
                    .body_contains("http://example.com/widgets/2")
                    .body_contains("--===============7330845974216740156==--");
                then.status(200)
                    .header(
                        "Content-Type",
                        format!("multipart/mixed; boundary={}", BOUNDARY).as_str(),
                    )
                    .body(batch_response_body());
            })
            .await;

        let result = HttpClient::execute_url(
            server.url("/batch").as_str(),
            "test-token",
            vec![
                "http://example.com/widgets/1".to_string(),
                "http://example.com/widgets/2".to_string(),
            ],
            UrlNotificationsType::UPDATED,
        )
        .await
        .unwrap();
        mock.assert_async().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].url(), "http://example.com/widgets/1");
        assert_eq!(result[0].status_code(), 200);
        assert_eq!(
            result[0].json()["urlNotificationMetadata"]["url"],
            "http://example.com/widgets/1"
        );
        assert_eq!(result[1].url(), "http://example.com/widgets/2");
        assert_eq!(result[1].status_code(), 403);
    }

    #[tokio::test]
    async fn test_execute_batch_with_quoted_boundary() {
        let server = MockServer::start_async().await;
        server
            .mock_async(|when, then| {
                when.method(POST).path("/batch");
                then.status(200)
                    .header(
                        "Content-Type",
                        format!(r#"multipart/mixed; boundary="{}""#, BOUNDARY).as_str(),
                    )
                    .body(batch_response_body());
            })
            .await;

        let result = HttpClient::execute_url(
            server.url("/batch").as_str(),
            "test-token",
            vec![
                "http://example.com/widgets/1".to_string(),
                "http://example.com/widgets/2".to_string(),
            ],
            UrlNotificationsType::UPDATED,
        )
        .await
        .unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_execute_error_when_content_type_has_no_boundary() {
        let server = MockServer::start_async().await;
        server
            .mock_async(|when, then| {
                when.method(POST).path("/batch");
                then.status(200)
                    .header("Content-Type", "multipart/mixed")
                    .body(batch_response_body());
            })
            .await;

        let result = HttpClient::execute_url(
            server.url("/batch").as_str(),
            "test-token",
            vec![
                "http://example.com/widgets/1".to_string(),
                "http://example.com/widgets/2".to_string(),
            ],
            UrlNotificationsType::UPDATED,
        )
        .await;
        assert!(matches!(result, Err(GoogleApiError::JsonParse(_))));
    }

    #[tokio::test]
    async fn test_execute_error_when_no_closing_boundary() {
        let server = MockServer::start_async().await;
        server
            .mock_async(|when, then| {
                when.method(POST).path("/batch");
                then.status(200)
                    .header(
                        "Content-Type",
                        format!("multipart/mixed; boundary={}", BOUNDARY).as_str(),
                    )
                    // 終了バウンダリーの無い壊れたレスポンス
                    .body(batch_part(1, "200 OK", r#"{"ok":true}"#));
            })
            .await;

        let result = HttpClient::execute_url(
            server.url("/batch").as_str(),
            "test-token",
            vec!["http://example.com/widgets/1".to_string()],
            UrlNotificationsType::UPDATED,
        )
        .await;
        assert!(matches!(result, Err(GoogleApiError::JsonParse(_))));
    }

    #[tokio::test]
    async fn test_execute_error_when_response_count_mismatch() {
        let server = MockServer::start_async().await;
        server
            .mock_async(|when, then| {
                when.method(POST).path("/batch");
                then.status(200)
                    .header(
                        "Content-Type",
                        format!("multipart/mixed; boundary={}", BOUNDARY).as_str(),
                    )
                    // 2件のリクエストに対して1件しか返ってこないレスポンス
                    .body(format!(
                        "{}--{}--\r\n",
                        batch_part(1, "200 OK", r#"{"ok":true}"#),
                        BOUNDARY
                    ));
            })
            .await;

        let result = HttpClient::execute_url(
            server.url("/batch").as_str(),
            "test-token",
            vec![
                "http://example.com/widgets/1".to_string(),
                "http://example.com/widgets/2".to_string(),
            ],
            UrlNotificationsType::UPDATED,
        )
        .await;
        assert!(matches!(result, Err(GoogleApiError::JsonParse(_))));
    }

    async fn execute_one_url_against(
        server: &MockServer,
    ) -> Result<Vec<ResponseGoogleIndexingBatch>, GoogleApiError> {
        HttpClient::execute_url(
            server.url("/batch").as_str(),
            "test-token",
            vec!["http://example.com/widgets/1".to_string()],
            UrlNotificationsType::UPDATED,
        )
        .await
    }

    async fn mock_batch_response(server: &MockServer, body: String) {
        server
            .mock_async(|when, then| {
                when.method(POST).path("/batch");
                then.status(200)
                    .header(
                        "Content-Type",
                        format!("multipart/mixed; boundary={}", BOUNDARY).as_str(),
                    )
                    .body(body);
            })
            .await;
    }

    #[tokio::test]
    async fn test_execute_error_when_unknown_content_id() {
        let server = MockServer::start_async().await;
        mock_batch_response(
            &server,
            format!(
                "{}--{}--\r\n",
                batch_part_with_id("deadbeef+1", "200 OK", r#"{"ok":true}"#),
                BOUNDARY
            ),
        )
        .await;

        match execute_one_url_against(&server).await {
            Err(GoogleApiError::JsonParse(msg)) => {
                assert!(msg.contains("unknown Content-ID"), "{}", msg)
            }
            other => panic!("unknown Content-ID はエラーであるべき: {:?}", other.err()),
        }
    }

    #[tokio::test]
    async fn test_execute_error_when_duplicated_content_id() {
        let server = MockServer::start_async().await;
        mock_batch_response(
            &server,
            format!(
                "{}{}--{}--\r\n",
                batch_part(1, "200 OK", r#"{"ok":true}"#),
                batch_part(1, "200 OK", r#"{"ok":true}"#),
                BOUNDARY
            ),
        )
        .await;

        let result = HttpClient::execute_url(
            server.url("/batch").as_str(),
            "test-token",
            vec![
                "http://example.com/widgets/1".to_string(),
                "http://example.com/widgets/2".to_string(),
            ],
            UrlNotificationsType::UPDATED,
        )
        .await;
        match result {
            Err(GoogleApiError::JsonParse(msg)) => {
                assert!(msg.contains("duplicated Content-ID"), "{}", msg)
            }
            other => panic!("重複した Content-ID はエラーであるべき: {:?}", other.err()),
        }
    }

    #[tokio::test]
    async fn test_execute_error_when_status_line_unparsable() {
        let server = MockServer::start_async().await;
        mock_batch_response(
            &server,
            format!(
                "{}--{}--\r\n",
                batch_part(1, "XXX Broken", r#"{"ok":true}"#),
                BOUNDARY
            ),
        )
        .await;

        match execute_one_url_against(&server).await {
            Err(GoogleApiError::JsonParse(msg)) => {
                assert!(msg.contains("HTTP status line"), "{}", msg)
            }
            other => panic!(
                "status を解析できない part はエラーであるべき: {:?}",
                other.err()
            ),
        }
    }

    #[tokio::test]
    async fn test_execute_rejects_invalid_url_count() {
        // バリデーションは送信前に行われるので、送信不能な endpoint でも検証できる
        // (もし送信されてしまったら Connection エラーになりテストは即時に落ちる)
        let result = HttpClient::execute_url(
            "not-a-url",
            "test-token",
            vec![],
            UrlNotificationsType::UPDATED,
        )
        .await;
        assert!(matches!(result, Err(GoogleApiError::InvalidArgument(_))));

        let too_many = (0..101)
            .map(|i| format!("http://example.com/{}", i))
            .collect::<Vec<String>>();
        let result = HttpClient::execute_url(
            "not-a-url",
            "test-token",
            too_many,
            UrlNotificationsType::UPDATED,
        )
        .await;
        assert!(matches!(result, Err(GoogleApiError::InvalidArgument(_))));
    }

    #[tokio::test]
    async fn test_execute_http_error_returns_status_and_body() {
        let server = MockServer::start_async().await;
        server
            .mock_async(|when, then| {
                when.method(POST).path("/batch");
                then.status(500).body("internal error");
            })
            .await;

        let result = HttpClient::execute_url(
            server.url("/batch").as_str(),
            "test-token",
            vec!["http://example.com/widgets/1".to_string()],
            UrlNotificationsType::UPDATED,
        )
        .await;
        match result {
            Err(GoogleApiError::HttpStatus(status, body)) => {
                assert_eq!(status, 500);
                assert_eq!(body, "internal error");
            }
            other => panic!("HttpStatus であるべき: {:?}", other.err()),
        }
    }
}
