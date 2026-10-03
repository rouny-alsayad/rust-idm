#[tokio::main]
async fn main() {
    let client = reqwest::Client::builder().user_agent("Mozilla/5.0").timeout(std::time::Duration::from_secs(3)).build().unwrap();
    let resp = client.head("https://files2.freedownloadmanager.org/6/latest/fdm_x64_setup.exe").send().await;
    println!("Resp: {:?}", resp);
}