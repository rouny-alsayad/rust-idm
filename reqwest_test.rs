#[tokio::main]
async fn main() {
    println!("Testing reqwest");
    let client = reqwest::Client::builder().build().unwrap();
    println!("Success");
}