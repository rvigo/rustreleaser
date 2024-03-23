use super::github_client::GITHUB_TOKEN;
use reqwest::{
    header::{ACCEPT, USER_AGENT},
    RequestBuilder,
};

pub trait Headers {
    fn default_headers(self) -> RequestBuilder;
}

impl Headers for RequestBuilder {
    fn default_headers(self) -> RequestBuilder {
        self.bearer_auth(GITHUB_TOKEN.to_string())
            .header(ACCEPT, "application/vnd.github.VERSION.sha")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(USER_AGENT, "rustreleaser")
    }
}

#[macro_export]
macro_rules! put {
    ($url:expr, $body:expr) => {{
        use $crate::{github::macros::Headers, http::ResponseHandler};

        $crate::http::HttpClient::new()
            .put($url)
            .default_headers()
            .body($body)
            .send()
            .await
            .handle()
            .await
    }};
}

#[macro_export]
macro_rules! get {
    ($url:expr) => {{
        use $crate::{github::macros::Headers, http::ResponseHandler};

        $crate::http::HttpClient::new()
            .get($url)
            .default_headers()
            .send()
            .await
            .handle()
            .await
    }};
}

#[macro_export]
macro_rules! post {
    ($url:expr, $body:expr) => {{
        use $crate::{github::macros::Headers, http::ResponseHandler};

        $crate::http::HttpClient::new()
            .post($url)
            .default_headers()
            .body($body)
            .send()
            .await
            .handle()
            .await
    }};
}

#[macro_export]
macro_rules! upload_file {
    ($url:expr, $content:expr) => {{
        use reqwest::header::CONTENT_TYPE;
        use $crate::{github::macros::Headers, http::ResponseHandler};

        $crate::http::HttpClient::new()
            .post($url)
            .default_headers()
            .header(CONTENT_TYPE, "application/octet-stream")
            .body($content)
            .send()
            .await
            .handle()
            .await
    }};
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use mockito::Server;
    use std::env;
    use tokio::join;

    #[tokio::test]
    async fn put_macro() -> Result<()> {
        env::set_var("GITHUB_TOKEN", "token");
        let mut server = Server::new_async().await;
        let url = server.url();

        let expected_body = "test_body";
        let mock_future = server
            .mock("PUT", "/")
            .with_header("authorization", "Bearer test_token")
            .with_header("accept", "application/vnd.github.VERSION.sha")
            .with_header("x-github-api-version", "2022-11-28")
            .with_header("user-agent", "rustreleaser")
            .with_body(expected_body)
            .create_async();

        let (m, ..) = join!(mock_future);

        let response = put!(url, expected_body)?;

        m.assert_async().await;
        assert_eq!(response, expected_body);

        Ok(())
    }

    #[tokio::test]
    async fn get_macro() -> Result<()> {
        env::set_var("GITHUB_TOKEN", "token");
        let mut server = Server::new_async().await;
        let url = server.url();

        let expected_body = "test_body";

        let mock_future = server
            .mock("GET", "/")
            .with_header("authorization", "Bearer test_token")
            .with_header("accept", "application/vnd.github.VERSION.sha")
            .with_header("x-github-api-version", "2022-11-28")
            .with_header("user-agent", "rustreleaser")
            .with_body(expected_body)
            .create_async();

        let (m, ..) = join!(mock_future);

        let response = get!(url)?;

        m.assert_async().await;

        assert_eq!(response, expected_body);

        Ok(())
    }

    #[tokio::test]
    async fn post_macro() -> Result<()> {
        env::set_var("GITHUB_TOKEN", "token");
        let mut server = Server::new_async().await;
        let url = server.url();

        let expected_body = "test_body";
        let mock_future = server
            .mock("POST", "/")
            .with_header("authorization", "Bearer test_token")
            .with_header("accept", "application/vnd.github.VERSION.sha")
            .with_header("x-github-api-version", "2022-11-28")
            .with_header("user-agent", "rustreleaser")
            .with_body(expected_body)
            .create_async();

        let (m, ..) = join!(mock_future);

        let response = post!(url, expected_body)?;

        m.assert_async().await;
        assert_eq!(response, expected_body);

        Ok(())
    }

    // #[tokio::test]
    // async fn form_macro() -> Result<()> {
    // env::set_var("GITHUB_TOKEN", "token");
    // let mut server = Server::new_async().await;
    // let url = server.url();
    //
    // let expected_body = "test_body";
    // let mock_future = server
    // .mock("POST", "/")
    // .with_header("authorization", "Bearer test_token")
    // .with_header("accept", "application/vnd.github.VERSION.sha")
    // .with_header("x-github-api-version", "2022-11-28")
    // .with_header("user-agent", "rustreleaser")
    // .with_header("content-type", "application/octet-stream")
    // .with_body(expected_body)
    // .create_async();
    //
    // let (m, ..) = join!(mock_future);
    //
    // let form = reqwest::multipart::Form::new().text("key", "value");
    // let response = upload_file!(url, form)?;
    //
    // m.assert_async().await;
    // assert_eq!(response, expected_body);
    //
    // Ok(())
    // }
}
