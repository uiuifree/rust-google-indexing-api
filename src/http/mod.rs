use std::collections::HashMap;
use std::fmt::{Debug};
use hyper::{Body, Client, Method, Request};
use hyper::header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE};
use hyper::http::HeaderValue;
use hyper_tls::HttpsConnector;
use serde_json::{json};
use crate::error::GoogleApiError;
use crate::{GoogleIndexingBatch, UrlNotificationsType};


#[derive(Default, Debug)]
pub(crate) struct HttpClient {}


impl HttpClient {
    pub async fn get<T>(token: &str, url: &str) -> Result<T, GoogleApiError>
        where
            T: for<'de> serde::Deserialize<'de>,
    {
        let mut response = reqwest::Client::new()
            .get(format!("{}", url))
            .header("Authorization", format!("Bearer {}", token));

        response = response.header("Accept", "application/json");

        let response = response.send().await;


        if response.is_err() {
            return Err(GoogleApiError::Connection(response.err().unwrap().to_string()));
        }
        let response = response.unwrap();
        let status = response.status();
        let value = response.text().await;
        if !(200 <= status.as_u16() && status.as_u16() < 300) {
            return Err(GoogleApiError::JsonParse(value.unwrap().to_string()));
        }
        if value.is_err() {
            return Err(GoogleApiError::JsonParse(value.unwrap().to_string()));
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
            U: serde::Serialize + std::fmt::Debug
    {
        let mut response = reqwest::Client::new()
            .post(format!("{}", url));
        if !token.is_empty() {
            response = response.header("Authorization", format!("Bearer {}", token))
        }
        let response = response
            .json(&json!(params))
            .send()
            .await;


        if response.is_err() {
            return Err(GoogleApiError::Connection(response.err().unwrap().to_string()));
        }
        let response = response.unwrap();
        let status = response.status();
        let value = response.text().await;
        if status != 200 {
            return Err(GoogleApiError::JsonParse(value.unwrap().to_string()));
        }
        if value.is_err() {
            return Err(GoogleApiError::JsonParse(value.unwrap().to_string()));
        }
        let value = value.unwrap();
        let parse = serde_json::from_str(value.as_str());
        if parse.is_err() {
            return Err(GoogleApiError::JsonParse(value));
        }

        Ok(parse.unwrap())
    }

    pub async fn execute(token: &str, urls: Vec<String>,url_type: UrlNotificationsType) -> Result<Vec<GoogleIndexingBatch> ,GoogleApiError>{
        // マルチパートフォームデータのバウンダリー
        let boundary = "===============7330845974216740156==";
        let boundary2 = "--===============7330845974216740156==";
        fn make_row(index: isize, url: &str, url_type: &str) -> (String, String, String) {
            let body = json!({
            "url":url,
            "type":url_type,
        }).to_string();
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
                ].join("\r\n")
            )
        }

        let send_data1 = urls.iter().enumerate().map(|(i, url)| {
            make_row(i as isize, url, url_type.to_string().as_str())
        }).collect::<Vec<(String, String, String)>>();

        let key_values = send_data1.clone().iter().map(|q| (q.0.to_string(), q.1.to_string())).collect::<Vec<(String, String)>>();
        let send_body = send_data1.iter().map(|q| q.2.to_string()).collect::<Vec<String>>()
            .join(format!("\r\n{}\r\n", boundary2).as_str());
        // マルチパートフォームデータのテキスト部分
        let text_parts = [
            boundary2.to_string(),
            send_body,
            boundary2.to_string(),
            "".to_string(),
        ];
        // リクエストの作成
        let mut request = Request::builder()
            .method(Method::POST)
            .uri("https://indexing.googleapis.com/batch")
            .body(Body::empty())
            .unwrap();

        let headers = request.headers_mut();
        let a = format!("multipart/mixed; boundary={}", boundary);
        let b = format!("Bearer {}", token);
        headers.insert(CONTENT_TYPE, HeaderValue::from_str(a.as_str()).unwrap());
        headers.insert(AUTHORIZATION, HeaderValue::from_str(b.as_str()).unwrap());
        headers.insert(CONTENT_LENGTH, HeaderValue::from_static("content_length"));

        // マルチパートフォームデータをリクエストボディに設定
        *request.body_mut() = Body::from(text_parts.join("\r\n"));


        let c = HttpsConnector::new();
        let client = Client::builder().build(c);

        // リクエストの送信とレスポンスの取得
        let response = client.request(request).await.unwrap();

        let status = response.status();
        if status != 200 {
            return Err(GoogleApiError::JsonParse("".to_string()));
        }
        // println!("status {}", status);
        let headers = response.headers();
        // println!("headers {:?}", headers);

        let mut content_type = "".to_string();
        if headers.get("Content-Type").is_some() {
            content_type = headers.get("Content-Type").unwrap().to_str().unwrap().to_string();
        }

        // let b = response.into_parts();
        // println!("b {:?}", b);
        // レスポンスのボディの読み取り
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap().to_vec();
        let boundary = get_boundary(content_type.as_str());
        let body = String::from_utf8(body).unwrap();


        let mut batch_response = vec![];


        let boundary_bodies = body_boundary_split(body.as_str(), boundary.as_str());
        for boundary_body in boundary_bodies {
            let http = plane_http_to_response(boundary_body.as_str());
            let mut http_url = "".to_string();
            for (id, url) in &key_values {
                if id == &http.content_id {
                    http_url = url.to_string();
                    break;
                }
            }
            batch_response.push(GoogleIndexingBatch {
                url: http_url,
                status_code: http.status_code,
                value: http.content,
            });
        }


        Ok(batch_response)
    }
}fn get_boundary(value: &str) -> String {
    if !value.contains("multipart/mixed") {
        return "".to_string();
    }
    if !value.contains("boundary=") {
        return "".to_string();
    }
    let d = value.split("boundary=").collect::<Vec<&str>>();
    let boundary = d.get(1).unwrap().to_string();
    return boundary;
}

fn plane_http_to_response(content: &str) -> HttpResponse {
    let delimiter = "\r\n\r\n";
    // HeaderとBodyに分割
    let (header, body) = split_one(content, delimiter);
    let mut http = header_from_plane_text(header.as_str());
    let content_type = http.content_type();
    if content_type.contains("application/http") {
        let mut content_id = http.content_id.to_string();
        if http.header.get("Content-ID").is_some() {
            content_id = http.header.get("Content-ID").unwrap().trim_start_matches("<response-").trim_end_matches(">").to_string();
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
            let tmp_status = row.to_string().split(" ").map(|q| q.to_string()).collect::<Vec<String>>();
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
    return response;
}


fn body_boundary_split(content: &str, boundary: &str) -> Vec<String> {
    let end_boundary = format!("--{}--", boundary);
    if !content.contains(end_boundary.as_str()) {
        return vec![];
    }
    let content = content.split(end_boundary.as_str()).collect::<Vec<&str>>().get(0).unwrap().to_string();

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
        let mut content_type = "".to_string();
        if self.header.get("Content-Type").is_some() {
            content_type = self.header.get("Content-Type").unwrap().to_string();
        }

        content_type
    }
}
