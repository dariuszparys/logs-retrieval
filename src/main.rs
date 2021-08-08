use hyper::{Body, Client, Request, Response, body::HttpBody};
use hyperlocal::{UnixClientExt, Uri};
use std::error::Error;
use tokio::io::{self, AsyncWriteExt as _};
use serde::{Deserialize, Serialize};

const DOCKER_API_VERSION: &str = "v1.41";

#[derive(Serialize, Deserialize)]
struct ContainerCreate {
    image: String,
}

#[derive(Serialize, Deserialize)]
struct ContainerCreateWithCommand {
    image: String,
    cmd: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct ContainerCreateResponse {
    id: String
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut response = list_all_images().await?;
    while let Some(next) = response.data().await {
        let chunk = next?;
        io::stdout().write_all(&chunk).await?;
    }

    response = pull_image("alpine:latest").await?;
    while let Some(next) = response.data().await {
        let chunk = next?;
        io::stdout().write_all(&chunk).await?;
    }

    response = create_container_from_image_with_command("alpine:latest", vec!["echo".to_string(), "Hello, world.".to_string()]).await?;
    while let Some(next) = response.data().await {
        let chunk = next?;
        io::stdout().write_all(&chunk).await?;
    }

    Ok(())
}

fn build_any_hyper_url(path: &str) -> http::Uri {
    let versioned_path = format!("/{}{}", DOCKER_API_VERSION, path);
    let url: http::Uri = Uri::new("/var/run/docker.sock", versioned_path.as_str()).into();
    url
}

async fn list_all_images() -> Result<Response<Body>, hyper::Error> {
    let url = build_any_hyper_url("/images/json");
    let client = Client::unix();
    client.get(url).await
}

async fn pull_image(name: &str) -> Result<Response<Body>, hyper::Error> {
    let path = format!("/images/create?fromImage={}", name);
    let url = build_any_hyper_url(path.as_str());
    let client = Client::unix();
    
    let req = Request::post(url)
        .body(Body::empty())
        .unwrap();
    
    client.request(req).await
}

async fn create_container_from_image(name: &str) -> Result<Response<Body>, hyper::Error> {
    let url = build_any_hyper_url("/containers/create");
    let client = Client::unix();

    let image_to_create = serde_json::to_string(
        &ContainerCreate{ 
            image: name.to_string(),
        }).unwrap();

    let req = Request::post(url)
        .header("Content-Type", "application/json")
        .body(Body::from(image_to_create))
        .unwrap();

    client.request(req).await
}

async fn create_container_from_image_with_command(name: &str, cmd: Vec<String>) -> Result<Response<Body>, hyper::Error> {
    let url = build_any_hyper_url("/containers/create");
    let client = Client::unix();

    let image_to_create = serde_json::to_string(
        &ContainerCreateWithCommand{ 
            image: name.to_string(),
            cmd: cmd
        }).unwrap();

    let req = Request::post(url)
        .header("Content-Type", "application/json")
        .body(Body::from(image_to_create))
        .unwrap();

    client.request(req).await
}

async fn get_container_logs(container_id: &str) -> Result<Response<Body>, hyper::Error> {
    let path = format!("/containers/{}/logs?stdout=1", container_id);
    let url = build_any_hyper_url(path.as_str());
    let client = Client::unix();
    client.get(url).await
}
