use hyper::{Body, Client, Request, Response, body::HttpBody};
use hyperlocal::{UnixClientExt, Uri};
use std::error::Error;
use tokio::io::{self, AsyncWriteExt as _};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut response = get_all_images("/v1.41/images/json").await?;
    while let Some(next) = response.data().await {
        let chunk = next?;
        io::stdout().write_all(&chunk).await?;
    }

    response = pull_image("alpine:latest").await?;
    while let Some(next) = response.data().await {
        let chunk = next?;
        io::stdout().write_all(&chunk).await?;
    }

    Ok(())
}

fn build_any_hyper_url(path: &str) -> http::Uri {
    let url: http::Uri = Uri::new("/var/run/docker.sock", path).into();
    url
}

async fn get_all_images(path: &str) -> Result<Response<Body>, hyper::Error> {
    let url = build_any_hyper_url(path);
    println!("{:?}", url );
    let client = Client::unix();
    client.get(url).await
}

async fn pull_image(name: &str) -> Result<Response<Body>, hyper::Error> {
    let path = format!("/v1.41/images/create?fromImage={}", name);

    let url = build_any_hyper_url(path.as_str());
    println!("{:?}", url );
    let client = Client::unix();
    
    let req = Request::post(url)
        .body(Body::empty())
        .unwrap();
    
    client.request(req).await
}