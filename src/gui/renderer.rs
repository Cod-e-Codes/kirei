use crate::gui::core::{DrawPass, Painter, Rect};
use glam::Vec2;
use glyphon::{
    Attrs, Buffer, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea,
    TextAtlas, TextBounds, TextRenderer, Wrap,
};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

/// Map a widget content size to cosmic-text `Buffer::set_size` options. Non-finite `y` means
/// unbounded vertical layout (all lines are shaped).
fn cosmic_text_buffer_size(layout_size: Vec2) -> (Option<f32>, Option<f32>) {
    let w = if layout_size.x.is_finite() {
        Some(layout_size.x.max(1.0))
    } else {
        Some(1.0)
    };
    let h = if !layout_size.y.is_finite() {
        None
    } else if layout_size.y <= 0.0 {
        Some(1.0)
    } else {
        Some(layout_size.y)
    };
    (w, h)
}

#[derive(Debug)]
pub enum GuiError {
    TextureLoadFailed(String),
    TextRenderFailed(String),
    BufferCreationFailed,
    InvalidTextureId,
}

impl std::fmt::Display for GuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuiError::TextureLoadFailed(msg) => write!(f, "Texture load failed: {}", msg),
            GuiError::TextRenderFailed(msg) => write!(f, "Text render failed: {}", msg),
            GuiError::BufferCreationFailed => write!(f, "Buffer creation failed"),
            GuiError::InvalidTextureId => write!(f, "Invalid texture ID"),
        }
    }
}

impl std::error::Error for GuiError {}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GuiVertex {
    position: [f32; 2],
    color: [f32; 4],
    tex_coords: [f32; 2],
    rect_pos: [f32; 2],  // Top-left corner of the rect
    rect_size: [f32; 2], // Width and height of the rect
    radius: f32,         // Corner radius (0 = no rounding)
    _padding: [f32; 3],  // Padding to align to 16 bytes
}

impl GuiVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GuiVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // tex_coords
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 2]>() + std::mem::size_of::<[f32; 4]>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // rect_pos
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 2]>()
                        + std::mem::size_of::<[f32; 4]>()
                        + std::mem::size_of::<[f32; 2]>())
                        as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // rect_size
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 2]>()
                        + std::mem::size_of::<[f32; 4]>()
                        + std::mem::size_of::<[f32; 2]>()
                        + std::mem::size_of::<[f32; 2]>())
                        as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // radius
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 2]>()
                        + std::mem::size_of::<[f32; 4]>()
                        + std::mem::size_of::<[f32; 2]>() * 3)
                        as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

struct Batch {
    texture_id: usize,
    vertices: Vec<GuiVertex>,
    scissor: Option<Rect>,
}

/// Rendering context containing GPU resources and viewport dimensions
pub struct RenderContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub view: &'a wgpu::TextureView,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub width: u32,
    pub height: u32,
}

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_buffer_size: u64,
    batches: Vec<Batch>,
    uniform_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group_0: wgpu::BindGroup,

    // Texture management
    white_texture_id: usize,
    next_texture_id: usize,
    textures: HashMap<usize, wgpu::BindGroup>,
    sampler: wgpu::Sampler,

    // Scissor state
    current_scissor: Option<Rect>,

    // Text
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: glyphon::Viewport,
    pub atlas: TextAtlas,
    text_renderer: TextRenderer,
    normal_text_areas: Vec<OwnedTextArea>,
    overlay_text_areas: Vec<OwnedTextArea>,
    current_pass: DrawPass,

    vertex_pool: Vec<Vec<GuiVertex>>, // Reusable vertex buffers
    text_size_cache: HashMap<(String, u32), Vec2>, // Cache for text measurements: (text, font_size_bits) -> size
    wrapped_text_size_cache: HashMap<(String, u32, u32), Vec2>, // (text, font_size_bits, max_width_bits)
}

// Helper struct to hold text data until render
struct OwnedTextArea {
    text: String,
    pos: Vec2,
    color: [f32; 4],
    font_size: f32,
    /// Screen-space clip (usually the active scissor).
    clip_origin: Vec2,
    clip_size: Vec2,
    /// Size passed to `Buffer::set_size` for wrapping (width from viewport; height unbounded).
    layout_size: Vec2,
    wrap: Wrap,
    buffer: Option<Buffer>,
    hash: u64, // For change detection
}

impl OwnedTextArea {
    /// Calculate hash including all factors that affect text rendering
    fn calculate_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.text.hash(&mut hasher);
        self.font_size.to_bits().hash(&mut hasher);
        self.clip_origin.x.to_bits().hash(&mut hasher);
        self.clip_origin.y.to_bits().hash(&mut hasher);
        self.clip_size.x.to_bits().hash(&mut hasher);
        self.clip_size.y.to_bits().hash(&mut hasher);
        self.layout_size.x.to_bits().hash(&mut hasher);
        self.layout_size.y.to_bits().hash(&mut hasher);
        (self.wrap as u32).hash(&mut hasher);
        hasher.finish()
    }
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        // Shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GUI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
        });

        // Uniforms (Screen Size)
        let screen_size = [width as f32, height as f32];
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("GUI Uniform Buffer"),
            contents: bytemuck::cast_slice(&screen_size),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout_0 =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("GUI Bind Group Layout 0"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let bind_group_0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GUI Bind Group 0"),
            layout: &bind_group_layout_0,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let bind_group_layout_1 =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("GUI Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("GUI Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout_0, &bind_group_layout_1],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("GUI Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[GuiVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Initial empty vertex buffer
        let vertex_buffer_size = 1024;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GUI Vertex Buffer"),
            size: vertex_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create white texture
        let white_texture_size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let white_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("White Texture"),
            size: white_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &white_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255, 255, 255, 255],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            white_texture_size,
        );
        let white_view = white_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let white_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("White Texture Bind Group"),
            layout: &bind_group_layout_1,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&white_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let mut textures = HashMap::new();
        textures.insert(0, white_bind_group);

        // Text setup
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = glyphon::Cache::new(device);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);

        let viewport = glyphon::Viewport::new(device, &cache);

        Self {
            pipeline,
            vertex_buffer,
            vertex_buffer_size,
            batches: Vec::new(),
            uniform_buffer,
            bind_group_layout: bind_group_layout_1,
            bind_group_0,
            white_texture_id: 0,
            next_texture_id: 1,
            textures,
            sampler,
            current_scissor: None,
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            normal_text_areas: Vec::new(),
            overlay_text_areas: Vec::new(),
            current_pass: DrawPass::Normal,
            vertex_pool: Vec::new(),
            text_size_cache: HashMap::new(),
            wrapped_text_size_cache: HashMap::new(),
        }
    }

    pub fn load_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &str,
    ) -> Result<usize, GuiError> {
        let img = image::open(path)
            .map_err(|e| GuiError::TextureLoadFailed(e.to_string()))?
            .to_rgba8();
        let dimensions = img.dimensions();
        let rgba = img.into_raw();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        let id = self.next_texture_id;
        self.textures.insert(id, bind_group);
        self.next_texture_id += 1;
        Ok(id)
    }

    pub fn resize(&mut self, width: u32, height: u32, queue: &wgpu::Queue) {
        let screen_size = [width as f32, height as f32];
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&screen_size));
        self.viewport.update(queue, Resolution { width, height });
    }

    pub fn prepare(&mut self) {
        // Return vertex buffers to pool
        // Collect batches first to avoid borrow checker issues
        let batches = std::mem::take(&mut self.batches);
        for batch in batches {
            self.return_vertex_buffer(batch.vertices);
        }
        self.normal_text_areas.clear();
        self.overlay_text_areas.clear();
        // Clear text size cache to prevent unbounded growth
        self.text_size_cache.clear();
    }

    /// Clear batches and text areas without returning vertices to pool
    /// Useful for clearing between render passes (e.g., before drawing overlays)
    /// Also resets scissor state so overlays aren't clipped by previous scissor
    pub fn clear_for_overlay(&mut self) {
        self.batches.clear();
        self.normal_text_areas.clear();
        self.overlay_text_areas.clear();
        self.current_scissor = None;
    }

    /// Get the current counts of batches and text areas
    /// Used to split rendering between normal widgets and overlays
    pub fn get_content_counts(&self) -> (usize, usize) {
        (
            self.batches.len(),
            self.normal_text_areas.len() + self.overlay_text_areas.len(),
        )
    }

    /// Get a vertex buffer from the pool or create a new one
    fn get_vertex_buffer(&mut self, capacity: usize) -> Vec<GuiVertex> {
        self.vertex_pool
            .pop()
            .map(|mut v| {
                v.clear();
                if v.capacity() < capacity {
                    v.reserve(capacity - v.capacity());
                }
                v
            })
            .unwrap_or_else(|| Vec::with_capacity(capacity))
    }

    /// Return a vertex buffer to the pool
    fn return_vertex_buffer(&mut self, buffer: Vec<GuiVertex>) {
        if buffer.capacity() <= 4096 {
            self.vertex_pool.push(buffer);
        }
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        width: u32,
        height: u32,
    ) {
        // Render all content (no split)
        let mut ctx = RenderContext {
            device,
            queue,
            view,
            encoder,
            width,
            height,
        };
        self.render_split(&mut ctx, None)
    }

    /// Render with optional split point for overlay rendering
    /// If split is Some((batch_count, text_count)), renders normal content first,
    /// then overlay content. Otherwise renders everything together.
    pub fn render_split(&mut self, ctx: &mut RenderContext, split: Option<(usize, usize)>) {
        if let Some((normal_batch_count, _normal_text_count)) = split {
            // Phase 1: Render normal widget batches (shapes)
            if normal_batch_count > 0 {
                self.render_batches(ctx, 0, normal_batch_count);
            }

            // Phase 2: Render overlay batches (modal backdrop, etc.)
            if self.batches.len() > normal_batch_count {
                self.render_batches(ctx, normal_batch_count, self.batches.len());
            }

            // Phase 3: Render ALL text in correct order (normal first, then overlay)
            // We must combine them into a single prepare/render call because glyphon's
            // text_renderer.prepare() replaces the previous batch
            let mut normal_text = std::mem::take(&mut self.normal_text_areas);
            let mut overlay_text = std::mem::take(&mut self.overlay_text_areas);

            // Combine: normal first, then overlay
            normal_text.append(&mut overlay_text);

            if !normal_text.is_empty() {
                self.render_text_areas(ctx, &mut normal_text);
            }

            // Return empty vectors (text was consumed)
            self.normal_text_areas = Vec::new();
            self.overlay_text_areas = Vec::new();
        } else {
            // Render everything together (original behavior)
            self.render_batches(ctx, 0, self.batches.len());

            // Combine and render all text
            let mut normal_text = std::mem::take(&mut self.normal_text_areas);
            let mut overlay_text = std::mem::take(&mut self.overlay_text_areas);

            // Combine: normal first, then overlay
            normal_text.append(&mut overlay_text);

            if !normal_text.is_empty() {
                self.render_text_areas(ctx, &mut normal_text);
            }

            // Return empty vectors (text was consumed)
            self.normal_text_areas = Vec::new();
            self.overlay_text_areas = Vec::new();
        }
    }

    fn render_batches(&mut self, ctx: &mut RenderContext, start: usize, end: usize) {
        if start >= end || end > self.batches.len() {
            return;
        }

        let batches_to_render = &self.batches[start..end];
        if batches_to_render.is_empty() {
            return;
        }

        // Calculate total vertices needed for this range
        let total_vertices: usize = batches_to_render.iter().map(|b| b.vertices.len()).sum();
        let needed_size = (total_vertices * std::mem::size_of::<GuiVertex>()) as u64;

        // Only resize if we need more space
        if needed_size > self.vertex_buffer_size {
            let new_size = (needed_size * 3 / 2).max(1024);
            self.vertex_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("GUI Vertex Buffer"),
                size: new_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.vertex_buffer_size = new_size;
        }

        // Calculate vertex offset for batches before start
        let offset_before_start: usize =
            self.batches[..start].iter().map(|b| b.vertices.len()).sum();

        // Flatten and upload vertices for this range
        let mut all_vertices = Vec::with_capacity(total_vertices);
        for batch in batches_to_render {
            all_vertices.extend_from_slice(&batch.vertices);
        }
        let start_byte = (offset_before_start * std::mem::size_of::<GuiVertex>()) as u64;
        ctx.queue.write_buffer(
            &self.vertex_buffer,
            start_byte,
            bytemuck::cast_slice(&all_vertices),
        );

        let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("GUI Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: ctx.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group_0, &[]);

        let mut vertex_offset = offset_before_start;
        for batch in batches_to_render {
            if let Some(bg) = self.textures.get(&batch.texture_id) {
                // Apply Scissor
                if let Some(rect) = batch.scissor {
                    // Clamp scissor to render target bounds
                    let x = (rect.pos.x as u32).min(ctx.width);
                    let y = (rect.pos.y as u32).min(ctx.height);
                    let w = (rect.size.x as u32).min(ctx.width.saturating_sub(x));
                    let h = (rect.size.y as u32).min(ctx.height.saturating_sub(y));
                    pass.set_scissor_rect(x, y, w, h);
                } else {
                    // No scissor specified, use full render target dimensions
                    pass.set_scissor_rect(0, 0, ctx.width, ctx.height);
                }

                pass.set_bind_group(1, bg, &[]);
                let vertex_count = batch.vertices.len() as u32;
                let start_byte = (vertex_offset * std::mem::size_of::<GuiVertex>()) as u64;
                let end_byte =
                    start_byte + (vertex_count as usize * std::mem::size_of::<GuiVertex>()) as u64;

                pass.set_vertex_buffer(0, self.vertex_buffer.slice(start_byte..end_byte));
                pass.draw(0..vertex_count, 0..1);

                vertex_offset += vertex_count as usize;
            }
        }
    }

    fn render_text_areas(&mut self, ctx: &mut RenderContext, text_areas: &mut [OwnedTextArea]) {
        if text_areas.is_empty() {
            return;
        }

        // Reuse or create buffers based on hash
        for area in text_areas.iter_mut() {
            // Calculate current hash
            let current_hash = area.calculate_hash();

            // Only create new buffer if hash changed or buffer doesn't exist
            if area.buffer.is_none() || area.hash != current_hash {
                let mut buffer = Buffer::new(
                    &mut self.font_system,
                    Metrics::new(area.font_size, area.font_size * 1.2),
                );
                buffer.set_wrap(&mut self.font_system, area.wrap);
                let (lw, lh) = cosmic_text_buffer_size(area.layout_size);
                buffer.set_size(&mut self.font_system, lw, lh);
                buffer.set_text(
                    &mut self.font_system,
                    &area.text,
                    &Attrs::new().family(Family::SansSerif),
                    Shaping::Advanced,
                );
                area.hash = current_hash;
                area.buffer = Some(buffer);
            }
        }

        let text_areas_refs: Vec<TextArea> = text_areas
            .iter()
            .map(|area| TextArea {
                buffer: area.buffer.as_ref().unwrap(),
                left: area.pos.x,
                top: area.pos.y,
                scale: 1.0,
                bounds: TextBounds {
                    left: (area.clip_origin.x as i32),
                    top: (area.clip_origin.y as i32),
                    right: (area.clip_origin.x + area.clip_size.x) as i32,
                    bottom: (area.clip_origin.y + area.clip_size.y) as i32,
                },
                default_color: Color::rgba(
                    (area.color[0] * 255.0) as u8,
                    (area.color[1] * 255.0) as u8,
                    (area.color[2] * 255.0) as u8,
                    (area.color[3] * 255.0) as u8,
                ),
                custom_glyphs: &[],
            })
            .collect();

        self.text_renderer
            .prepare(
                ctx.device,
                ctx.queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                text_areas_refs,
                &mut self.swash_cache,
            )
            .unwrap();

        let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Text Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: ctx.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.text_renderer
            .render(&self.atlas, &self.viewport, &mut pass)
            .unwrap();
    }

    fn push_vertices(&mut self, vertices: Vec<GuiVertex>, texture_id: usize) {
        // Check if we can merge without holding a mutable reference
        let can_merge = if let Some(last) = self.batches.last() {
            last.texture_id == texture_id
                && last.scissor == self.current_scissor
                && last.vertices.len() + vertices.len() <= 65536
        } else {
            false
        };

        if can_merge {
            // Safe to unwrap because we just checked it exists
            self.batches.last_mut().unwrap().vertices.extend(vertices);
        } else {
            self.batches.push(Batch {
                texture_id,
                vertices,
                scissor: self.current_scissor,
            });
        }
    }
}

impl Renderer {
    fn draw_rect_internal(&mut self, rect: Rect, color: [f32; 4], radius: f32) {
        let x = rect.pos.x;
        let y = rect.pos.y;
        let w = rect.size.x;
        let h = rect.size.y;

        let mut vertices = self.get_vertex_buffer(6);
        let tex = [0.0, 0.0]; // Dummy tex coords
        let rect_pos = [x, y];
        let rect_size = [w, h];

        // Top-left triangle
        vertices.push(GuiVertex {
            position: [x, y],
            color,
            tex_coords: tex,
            rect_pos,
            rect_size,
            radius,
            _padding: [0.0; 3],
        });
        vertices.push(GuiVertex {
            position: [x, y + h],
            color,
            tex_coords: tex,
            rect_pos,
            rect_size,
            radius,
            _padding: [0.0; 3],
        });
        vertices.push(GuiVertex {
            position: [x + w, y],
            color,
            tex_coords: tex,
            rect_pos,
            rect_size,
            radius,
            _padding: [0.0; 3],
        });

        // Bottom-right triangle
        vertices.push(GuiVertex {
            position: [x + w, y],
            color,
            tex_coords: tex,
            rect_pos,
            rect_size,
            radius,
            _padding: [0.0; 3],
        });
        vertices.push(GuiVertex {
            position: [x, y + h],
            color,
            tex_coords: tex,
            rect_pos,
            rect_size,
            radius,
            _padding: [0.0; 3],
        });
        vertices.push(GuiVertex {
            position: [x + w, y + h],
            color,
            tex_coords: tex,
            rect_pos,
            rect_size,
            radius,
            _padding: [0.0; 3],
        });

        self.push_vertices(vertices, self.white_texture_id);
    }
}

impl Painter for Renderer {
    fn draw_rect(&mut self, rect: Rect, color: [f32; 4]) {
        self.draw_rect_internal(rect, color, 0.0);
    }

    fn draw_rounded_rect(&mut self, rect: Rect, color: [f32; 4], radius: f32) {
        self.draw_rect_internal(rect, color, radius);
    }

    fn draw_image(&mut self, rect: Rect, texture_id: usize) {
        let x = rect.pos.x;
        let y = rect.pos.y;
        let w = rect.size.x;
        let h = rect.size.y;
        let color = [1.0, 1.0, 1.0, 1.0];
        let rect_pos = [x, y];
        let rect_size = [w, h];

        let mut vertices = self.get_vertex_buffer(6);
        vertices.extend_from_slice(&[
            // Top-left triangle
            GuiVertex {
                position: [x, y],
                color,
                tex_coords: [0.0, 0.0],
                rect_pos,
                rect_size,
                radius: 0.0,
                _padding: [0.0; 3],
            },
            GuiVertex {
                position: [x, y + h],
                color,
                tex_coords: [0.0, 1.0],
                rect_pos,
                rect_size,
                radius: 0.0,
                _padding: [0.0; 3],
            },
            GuiVertex {
                position: [x + w, y],
                color,
                tex_coords: [1.0, 0.0],
                rect_pos,
                rect_size,
                radius: 0.0,
                _padding: [0.0; 3],
            },
            // Bottom-right triangle
            GuiVertex {
                position: [x + w, y],
                color,
                tex_coords: [1.0, 0.0],
                rect_pos,
                rect_size,
                radius: 0.0,
                _padding: [0.0; 3],
            },
            GuiVertex {
                position: [x, y + h],
                color,
                tex_coords: [0.0, 1.0],
                rect_pos,
                rect_size,
                radius: 0.0,
                _padding: [0.0; 3],
            },
            GuiVertex {
                position: [x + w, y + h],
                color,
                tex_coords: [1.0, 1.0],
                rect_pos,
                rect_size,
                radius: 0.0,
                _padding: [0.0; 3],
            },
        ]);

        self.push_vertices(vertices, texture_id);
    }

    fn draw_text(
        &mut self,
        text: &str,
        pos: Vec2,
        color: [f32; 4],
        font_size: f32,
        wrap: Wrap,
        layout_size: Vec2,
    ) {
        let (clip_origin, clip_size) = if let Some(rect) = self.current_scissor {
            (rect.pos, rect.size)
        } else {
            // When scissor is None (overlay mode), use very large bounds to prevent clipping
            // Use (0,0) as origin and very large size to cover entire screen
            (Vec2::new(0.0, 0.0), Vec2::new(10000.0, 10000.0))
        };

        // Create temporary OwnedTextArea to calculate hash
        let area = OwnedTextArea {
            text: text.to_string(),
            pos,
            color,
            font_size,
            clip_origin,
            clip_size,
            layout_size,
            wrap,
            buffer: None,
            hash: 0, // Will be calculated
        };

        let hash = area.calculate_hash();

        let owned_area = OwnedTextArea {
            text: text.to_string(),
            pos,
            color,
            font_size,
            clip_origin,
            clip_size,
            layout_size,
            wrap,
            buffer: None,
            hash,
        };

        // Route to the appropriate text list based on current pass
        match self.current_pass {
            DrawPass::Normal => self.normal_text_areas.push(owned_area),
            DrawPass::Overlay => self.overlay_text_areas.push(owned_area),
        }
    }

    fn get_text_size(&mut self, text: &str, font_size: f32) -> Vec2 {
        // Create cache key
        let key = (text.to_string(), font_size.to_bits());

        // Check cache first
        if let Some(&size) = self.text_size_cache.get(&key) {
            return size;
        }

        // Cache miss: measure text
        let mut buffer = Buffer::new(
            &mut self.font_system,
            Metrics::new(font_size, font_size * 1.2),
        );
        buffer.set_text(
            &mut self.font_system,
            text,
            &Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );

        let mut w: f32 = 0.0;
        let mut h: f32 = 0.0;

        for run in buffer.layout_runs() {
            w = w.max(run.line_w);
            h += run.line_height;
        }

        let size = Vec2::new(w, h);

        // Store in cache
        self.text_size_cache.insert(key, size);

        size
    }

    fn get_wrapped_text_size(&mut self, text: &str, font_size: f32, max_width: f32) -> Vec2 {
        let mw = max_width.max(1.0);
        let key = (text.to_string(), font_size.to_bits(), mw.to_bits());
        if let Some(&size) = self.wrapped_text_size_cache.get(&key) {
            return size;
        }

        let mut buffer = Buffer::new(
            &mut self.font_system,
            Metrics::new(font_size, font_size * 1.2),
        );
        buffer.set_wrap(&mut self.font_system, Wrap::Word);
        let (lw, lh) = cosmic_text_buffer_size(Vec2::new(mw, f32::INFINITY));
        buffer.set_size(&mut self.font_system, lw, lh);
        buffer.set_text(
            &mut self.font_system,
            text,
            &Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );

        let mut w: f32 = 0.0;
        let mut h: f32 = 0.0;
        for run in buffer.layout_runs() {
            w = w.max(run.line_w);
            h += run.line_height;
        }

        let size = Vec2::new(w.min(mw), h.max(font_size));
        self.wrapped_text_size_cache.insert(key, size);
        size
    }

    fn set_scissor(&mut self, rect: Option<Rect>) -> Option<Rect> {
        let previous = self.current_scissor;
        self.current_scissor = rect;
        previous
    }

    fn set_draw_pass(&mut self, pass: DrawPass) {
        self.current_pass = pass;
    }
}
