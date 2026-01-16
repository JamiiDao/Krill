use dioxus::prelude::*;

#[post("/api/data")]
pub async fn post_server_data(data: String) -> ServerFnResult {
    println!("Server received: {}", data);

    Ok(())
}

#[get("/api/data")]
pub async fn get_server_data() -> ServerFnResult<String> {
    Ok("Hello from the server!".to_string())
}
