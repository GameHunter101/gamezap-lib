use gamezap::Gamezap;

#[tokio::main]
async fn main() {
    let engine = Gamezap::builder().build().await;

    engine.main_loop().await;
}
