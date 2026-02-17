use dioxus::prelude::*;

#[post("/api/data")]
pub async fn post_server_data(data: String) -> ServerFnResult {
    println!("Server received: {}", data);

    Ok(())
}

#[get("/api/data")]
pub async fn get_server_data() -> ServerFnResult<String> {
    // let bg = crate::BRANDING_STORE.read().background_dark().to_string();

    // Ok("Hello from the server!".to_string())

    Ok("BG: ".to_string() + &"bg")
}
