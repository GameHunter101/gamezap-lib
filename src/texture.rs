use anyhow::*;

#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn blank_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        label: Option<&str>,
        is_normal: bool,
    ) -> Result<Self> {
        Self::from_rgba(
            device,
            queue,
            &image::RgbaImage::new(width, height),
            label,
            is_normal,
            true,
        )
    }

    pub fn load_ui_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &mut imgui_wgpu::Renderer,
        path: String,
    ) -> (imgui::TextureId, [f32; 2]) {
        let bytes = std::fs::read(&path).unwrap();
        let image = image::load_from_memory(&bytes).expect("Invalid image");
        let image = image.to_rgba8();
        let (width, height) = image.dimensions();
        let raw_data = image.into_raw();

        let texture_config = imgui_wgpu::TextureConfig {
            size: wgpu::Extent3d {
                width,
                height,
                ..Default::default()
            },
            label: Some(&path),
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            ..Default::default()
        };

        let texture = imgui_wgpu::Texture::new(device, renderer, texture_config);

        texture.write(queue, &raw_data, width, height);

        (
            renderer.textures.insert(texture),
            [width as f32, height as f32],
        )
    }

    pub async fn load_binary(file_name: &str, absolute_path: bool) -> anyhow::Result<Vec<u8>> {
        let path = if absolute_path {
            std::path::Path::new(file_name).to_path_buf()
        } else {
            std::path::Path::new(&std::env::current_dir().unwrap()).join(file_name)
        };
        let data = std::fs::read(path)?;

        Ok(data)
    }

    pub async fn load_texture(
        file_name: &str,
        absolute_path: bool,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        is_normal_map: bool,
    ) -> anyhow::Result<Texture> {
        let data = Self::load_binary(file_name, absolute_path).await?;
        Texture::from_bytes(device, queue, &data, file_name, is_normal_map)
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
        is_normal_map: bool,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, Some(label), is_normal_map)
    }

    pub fn from_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::RgbaImage,
        label: Option<&str>,
        is_normal_map: bool,
        is_storage_texture: bool,
    ) -> Result<Self> {
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let format = if is_normal_map {
            wgpu::TextureFormat::Rgba8Unorm
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: if is_storage_texture {
                wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
            } else {
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST
            },
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTextureBase {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            img,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
        is_normal_map: bool,
    ) -> Result<Self> {
        let rgba = img.to_rgba8();
        Self::from_rgba(device, queue, &rgba, label, is_normal_map, false)
    }

    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}
