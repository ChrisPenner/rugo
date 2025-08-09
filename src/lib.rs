use wasm_bindgen::prelude::*;
use web_sys::{console, HtmlCanvasElement};
use wgpu::util::DeviceExt;

// Import the `console.log` function from the browser console
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to make logging easier
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Go game constants
const BOARD_SIZE: usize = 19; // Standard Go board is 19x19
const STONE_RADIUS: f32 = 0.4;

// Game state
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StoneState {
    Empty,
    Black,
    White,
}

#[wasm_bindgen]
pub struct GoGame {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    board: [[StoneState; BOARD_SIZE]; BOARD_SIZE],
    current_player: StoneState,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

#[wasm_bindgen]
impl GoGame {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> Result<GoGame, JsValue> {
        console_log!("Initializing Go game with WebGPU...");
        
        // Initialize logging
        console_error_panic_hook::set_once();
        console_log::init_with_level(log::Level::Debug).expect("Failed to initialize logger");
        
        // Create the WGPU instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        // Create surface from canvas
        let surface = instance.create_surface_from_canvas(&canvas)?;

        // Initialize the rest asynchronously
        wasm_bindgen_futures::spawn_local(async move {
            // This will be completed in init_async
        });

        // For now, return a placeholder - we'll need to restructure this
        // to handle async initialization properly
        Err("Async initialization needed".into())
    }

    pub async fn init_async(canvas: HtmlCanvasElement) -> Result<GoGame, JsValue> {
        console_log!("Starting async WebGPU initialization...");

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface_from_canvas(&canvas)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or("Failed to find an appropriate adapter")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    label: None,
                },
                None,
            )
            .await
            .map_err(|e| format!("Failed to create device: {:?}", e))?;

        let canvas_width = canvas.width();
        let canvas_height = canvas.height();

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: canvas_width,
            height: canvas_height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        // Create render pipeline
        let render_pipeline = Self::create_render_pipeline(&device, &surface_config);

        // Create vertex and index buffers for the board
        let (vertex_buffer, index_buffer) = Self::create_board_geometry(&device);

        console_log!("WebGPU initialization complete!");

        Ok(GoGame {
            device,
            queue,
            surface,
            surface_config,
            render_pipeline,
            board: [[StoneState::Empty; BOARD_SIZE]; BOARD_SIZE],
            current_player: StoneState::Black,
            vertex_buffer,
            index_buffer,
        })
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Go Board Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    }

    fn create_board_geometry(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
        // Create vertices for the Go board grid lines
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Generate grid lines
        for i in 0..BOARD_SIZE {
            let pos = (i as f32 / (BOARD_SIZE - 1) as f32) * 2.0 - 1.0;
            
            // Horizontal lines
            vertices.push(Vertex { position: [-1.0, pos, 0.0], color: [0.0, 0.0, 0.0] });
            vertices.push(Vertex { position: [1.0, pos, 0.0], color: [0.0, 0.0, 0.0] });
            
            // Vertical lines  
            vertices.push(Vertex { position: [pos, -1.0, 0.0], color: [0.0, 0.0, 0.0] });
            vertices.push(Vertex { position: [pos, 1.0, 0.0], color: [0.0, 0.0, 0.0] });
        }

        // Generate indices for lines
        for i in 0..(BOARD_SIZE * 2) {
            indices.push((i * 2) as u16);
            indices.push((i * 2 + 1) as u16);
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        (vertex_buffer, index_buffer)
    }

    pub fn render(&mut self) -> Result<(), JsValue> {
        let output = self.surface.get_current_texture()
            .map_err(|e| format!("Failed to acquire next swap chain texture: {:?}", e))?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.8,
                            g: 0.6,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..(BOARD_SIZE * 4) as u32, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn handle_click(&mut self, x: f32, y: f32) {
        console_log!("Click at ({}, {})", x, y);
        // Convert screen coordinates to board coordinates
        // This is a simplified version - you'll want to improve this
        let board_x = ((x + 1.0) / 2.0 * BOARD_SIZE as f32) as usize;
        let board_y = ((y + 1.0) / 2.0 * BOARD_SIZE as f32) as usize;
        
        if board_x < BOARD_SIZE && board_y < BOARD_SIZE {
            if self.board[board_y][board_x] == StoneState::Empty {
                self.board[board_y][board_x] = self.current_player;
                self.current_player = match self.current_player {
                    StoneState::Black => StoneState::White,
                    StoneState::White => StoneState::Black,
                    StoneState::Empty => StoneState::Black,
                };
                console_log!("Placed stone at ({}, {})", board_x, board_y);
            }
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

// Initialize function to be called from JavaScript
#[wasm_bindgen(start)]
pub fn init() {
    console_log!("WASM module loaded successfully!");
}
