use gamezap::Gamezap;

#[tokio::main]
async fn main() {
    let engine = Gamezap::builder()
        .window_settings(
            600,
            600,
            "Example Gamezap Project",
            glfw::WindowMode::Windowed,
        )
        .clear_color(wgpu::Color { r: 0.8, g: 0.15, b: 0.2, a: 1.0 })
        .build()
        .await;

    engine.main_loop().await;
}
