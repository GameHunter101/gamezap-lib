use gamezap::Gamezap;
use winit::window::WindowAttributes;

#[tokio::main]
async fn main() {
    let engine = Gamezap::new(
        WindowAttributes::default()
            .with_title("Example Gamezap Project")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
    );

    engine.main_loop();
}
