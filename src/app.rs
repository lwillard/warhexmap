use std::sync::Arc;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

use crate::editor::brush_tool::BrushTool;
use crate::editor::eraser_tool::EraserTool;
use crate::editor::eyedropper_tool::EyedropperTool;
use crate::editor::label_tool::LabelTool;
use crate::editor::pen_tool::PenTool;
use crate::editor::select_tool::SelectTool;
use crate::editor::tool_manager::{ToolKind, ToolManager};
use crate::model::project::Project;
use crate::renderer::camera::{Camera, CameraUniform};
use crate::renderer::decorator_renderer::DecoratorMesh;
use crate::renderer::gpu_state::GpuState;
use crate::renderer::grid_overlay::GridOverlay;
use crate::renderer::hex_mesh::HexMesh;
use crate::renderer::minimap::MinimapRenderer;
use crate::renderer::path_renderer::PathMesh;
use crate::renderer::terrain_textures::TerrainTextures;
use crate::ui::editor_ui::EditorUi;

/// The hex size used for world-space hex layout (outer radius).
const HEX_SIZE: f32 = 32.0;

pub struct App {
    pub gpu: GpuState,
    pub project: Project,
    pub camera: Camera,
    pub tool_manager: ToolManager,
    pub brush_tool: BrushTool,
    pub pen_tool: PenTool,
    pub select_tool: SelectTool,
    pub label_tool: LabelTool,
    pub eraser_tool: EraserTool,
    pub eyedropper_tool: EyedropperTool,
    pub ui: EditorUi,

    // Render resources
    pub terrain_textures: TerrainTextures,
    pub hex_mesh: HexMesh,
    pub grid_overlay: GridOverlay,
    pub path_mesh: PathMesh,
    pub decorator_mesh: DecoratorMesh,
    pub hex_data_texture: wgpu::Texture,
    pub hex_data_view: wgpu::TextureView,

    // Pipelines
    pub terrain_pipeline: wgpu::RenderPipeline,
    pub grid_pipeline: wgpu::RenderPipeline,
    pub path_pipeline: wgpu::RenderPipeline,
    pub decorator_pipeline: wgpu::RenderPipeline,

    // Bind groups & buffers
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub terrain_bind_group: wgpu::BindGroup,

    // egui integration
    pub egui_ctx: egui::Context,
    pub egui_state: egui_winit::State,
    pub egui_renderer: egui_wgpu::Renderer,

    // Input state
    panning: bool,
    last_mouse_pos: (f32, f32),
    map_rect: egui::Rect,

    needs_mesh_rebuild: bool,
}

impl App {
    pub fn new(window: Arc<Window>) -> Self {
        // 1. GPU state
        let gpu = GpuState::new(window.clone());
        let device = &gpu.device;
        let queue = &gpu.queue;
        let surface_format = gpu.config.format;

        // 2. Project with default grid
        let mut project = Project::default();

        // 3. Camera centered on grid
        let camera = Camera::default();

        // 4. Tools
        let tool_manager = ToolManager::default();
        let brush_tool = BrushTool::default();
        let pen_tool = PenTool::default();
        let select_tool = SelectTool::default();
        let label_tool = LabelTool::default();
        let eraser_tool = EraserTool::default();
        let eyedropper_tool = EyedropperTool::default();

        // 5. Terrain textures
        let terrain_textures = TerrainTextures::new(device, queue);

        // 6. Build meshes
        let hex_mesh = HexMesh::build(device, &project.grid, HEX_SIZE);
        let grid_overlay = GridOverlay::build(device, &project.grid, HEX_SIZE);
        let path_mesh = PathMesh::build(device, &project.paths);
        let decorator_mesh = DecoratorMesh::build(device, &project.grid, HEX_SIZE);

        // 7. Hex data texture
        let (hex_data_texture, hex_data_view) = Self::create_hex_data_texture(&mut project.grid, device, queue);

        // 8. Shader modules
        let terrain_shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("terrain_shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("shaders/terrain.wgsl").into(),
                ),
            });
        let grid_shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("grid_shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("shaders/grid_overlay.wgsl").into(),
                ),
            });
        let path_shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("path_shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("shaders/path.wgsl").into(),
                ),
            });
        let decorator_shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("decorator_shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("shaders/decorator.wgsl").into(),
                ),
            });

        // 9. Bind group layouts
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
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

        let terrain_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("terrain_bind_group_layout"),
                entries: &[
                    // Terrain atlas texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Terrain sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                    // Hex data texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: false,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        // 10. Pipeline layouts
        let terrain_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("terrain_pipeline_layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &terrain_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let camera_only_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("camera_only_pipeline_layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        // 11. Camera buffer and bind group
        let camera_uniform = camera.build_uniform(
            gpu.config.width as f32,
            gpu.config.height as f32,
            HEX_SIZE,
            project.grid.grid_offset,
        );
        let camera_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("camera_buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("camera_bind_group"),
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
            });

        // Terrain bind group
        let terrain_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("terrain_bind_group"),
                layout: &terrain_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &terrain_textures.view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            &terrain_textures.sampler,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(
                            &hex_data_view,
                        ),
                    },
                ],
            });

        // 12. Render pipelines
        use crate::renderer::decorator_renderer::DecoratorInstance;
        use crate::renderer::grid_overlay::LineVertex;
        use crate::renderer::hex_mesh::HexVertex;
        use crate::renderer::path_renderer::PathVertex;

        let terrain_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("terrain_pipeline"),
                layout: Some(&terrain_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &terrain_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[HexVertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &terrain_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        let grid_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("grid_pipeline"),
                layout: Some(&camera_only_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &grid_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[LineVertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &grid_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::LineList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        let path_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("path_pipeline"),
                layout: Some(&camera_only_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &path_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[PathVertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &path_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
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

        let decorator_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("decorator_pipeline"),
                layout: Some(&camera_only_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &decorator_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[DecoratorInstance::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &decorator_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
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

        // 13. egui integration
        let egui_ctx = egui::Context::default();
        let viewport_id = egui_ctx.viewport_id();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            viewport_id,
            &window,
            None,
            None,
            None,
        );
        let egui_renderer =
            egui_wgpu::Renderer::new(device, surface_format, None, 1, false);

        // 14. Editor UI
        let ui = EditorUi::new();

        let map_rect = egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(
                gpu.config.width as f32,
                gpu.config.height as f32,
            ),
        );

        Self {
            gpu,
            project,
            camera,
            tool_manager,
            brush_tool,
            pen_tool,
            select_tool,
            label_tool,
            eraser_tool,
            eyedropper_tool,
            ui,

            terrain_textures,
            hex_mesh,
            grid_overlay,
            path_mesh,
            decorator_mesh,
            hex_data_texture,
            hex_data_view,

            terrain_pipeline,
            grid_pipeline,
            path_pipeline,
            decorator_pipeline,

            camera_bind_group,
            camera_buffer,
            camera_bind_group_layout,
            terrain_bind_group,

            egui_ctx,
            egui_state,
            egui_renderer,

            panning: false,
            last_mouse_pos: (0.0, 0.0),
            map_rect,

            needs_mesh_rebuild: false,
        }
    }

    fn create_hex_data_texture(
        grid: &mut crate::model::hex_grid::HexGrid,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let (data, tex_w, tex_h) = grid.build_hex_data_texture();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("hex_data_texture"),
            size: wgpu::Extent3d {
                width: tex_w,
                height: tex_h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Uint,
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
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(tex_w * 4),
                rows_per_image: Some(tex_h),
            },
            wgpu::Extent3d {
                width: tex_w,
                height: tex_h,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.gpu.resize(size.width, size.height);
        }
    }

    /// Let egui and/or the app handle a window event.
    /// Returns true if the event was consumed by egui.
    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) -> bool {
        // Let egui process the event first
        let egui_response = self.egui_state.on_window_event(window, event);
        if egui_response.consumed {
            return true;
        }

        // If egui wants pointer input, don't forward mouse events to the map
        let egui_wants_pointer = self.egui_ctx.wants_pointer_input();
        let egui_wants_keyboard = self.egui_ctx.wants_keyboard_input();

        match event {
            // --- Mouse ---
            WindowEvent::MouseInput { state, button, .. } if !egui_wants_pointer => {
                match button {
                    MouseButton::Middle => {
                        self.panning = *state == ElementState::Pressed;
                    }
                    MouseButton::Left => {
                        if *state == ElementState::Pressed {
                            let (mx, my) = self.last_mouse_pos;
                            let (wx, wy) = self.camera.screen_to_world(mx, my, self.gpu.config.width as f32, self.gpu.config.height as f32);
                            let hex =
                                crate::hex_math::pixel_to_hex(wx, wy, HEX_SIZE);
                            self.apply_tool_press(hex);
                        }
                    }
                    _ => {}
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                let (px, py) = (position.x as f32, position.y as f32);
                if self.panning && !egui_wants_pointer {
                    let dx = px - self.last_mouse_pos.0;
                    let dy = py - self.last_mouse_pos.1;
                    self.camera.pan(-dx, -dy);
                }
                self.last_mouse_pos = (px, py);

                // Update hovered hex for properties panel
                if !egui_wants_pointer {
                    let (wx, wy) = self.camera.screen_to_world(px, py, self.gpu.config.width as f32, self.gpu.config.height as f32);
                    let (q, r) =
                        crate::hex_math::pixel_to_hex(wx, wy, HEX_SIZE);
                    if let Some(cell) = self.project.grid.get(q, r) {
                        self.ui.properties.hovered_hex = Some(cell.clone());
                    } else {
                        self.ui.properties.hovered_hex = None;
                    }
                    self.ui.status_text =
                        format!("Cursor: ({:.0}, {:.0})  Hex: ({}, {})", px, py, q, r);
                }
            }

            WindowEvent::MouseWheel { delta, .. } if !egui_wants_pointer => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 60.0,
                };
                let (mx, my) = self.last_mouse_pos;
                self.camera.zoom_at(mx, my, self.gpu.config.width as f32, self.gpu.config.height as f32, scroll * 0.1);
            }

            // --- Keyboard shortcuts ---
            WindowEvent::KeyboardInput { event, .. }
                if !egui_wants_keyboard
                    && event.state == ElementState::Pressed =>
            {
                if let PhysicalKey::Code(code) = event.physical_key {
                    match code {
                        KeyCode::KeyB => {
                            self.ui.toolbar.active_tool = ToolKind::Brush;
                        }
                        KeyCode::KeyP => {
                            self.ui.toolbar.active_tool = ToolKind::Pen;
                        }
                        KeyCode::KeyS => {
                            self.ui.toolbar.active_tool = ToolKind::Select;
                        }
                        KeyCode::KeyL => {
                            self.ui.toolbar.active_tool = ToolKind::Label;
                        }
                        KeyCode::KeyE => {
                            self.ui.toolbar.active_tool = ToolKind::Eraser;
                        }
                        KeyCode::KeyI => {
                            self.ui.toolbar.active_tool = ToolKind::Eyedropper;
                        }
                        _ => {}
                    }
                }
            }

            _ => {}
        }

        false
    }

    fn apply_tool_press(&mut self, hex: (i32, i32)) {
        let (q, r) = hex;
        let (wx, wy) = crate::hex_math::hex_to_pixel(q, r, HEX_SIZE);
        match self.ui.toolbar.active_tool {
            ToolKind::Brush => {
                self.brush_tool.paint_mode = self.ui.palette.paint_mode;
                self.brush_tool.elevation_value = self.ui.properties.selected_elevation;
                self.brush_tool.climate_value = self.ui.properties.selected_climate;
                self.brush_tool.radius = self.ui.properties.brush_radius;
                self.brush_tool.on_press(
                    wx,
                    wy,
                    &mut self.project.grid,
                    &mut self.tool_manager,
                );
                self.brush_tool.on_release(
                    wx,
                    wy,
                    &mut self.project.grid,
                    &mut self.tool_manager,
                );
                self.needs_mesh_rebuild = true;
            }
            ToolKind::Eraser => {
                self.eraser_tool.on_press(
                    wx,
                    wy,
                    &mut self.project.grid,
                    &mut self.tool_manager,
                );
                self.eraser_tool.on_release(&mut self.tool_manager, &self.project.grid);
                self.needs_mesh_rebuild = true;
            }
            ToolKind::Eyedropper => {
                if let Some(cell) = self.project.grid.get(q, r) {
                    self.ui.properties.selected_elevation = cell.elevation;
                    self.ui.properties.selected_climate = cell.climate;
                }
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        // Handle file operations
        if self.ui.pending_new {
            self.project = Project::default();
            self.needs_mesh_rebuild = true;
        }

        // Rebuild meshes when dirty
        if self.needs_mesh_rebuild {
            let device = &self.gpu.device;
            let queue = &self.gpu.queue;

            self.hex_mesh =
                HexMesh::build(device, &self.project.grid, HEX_SIZE);
            self.grid_overlay =
                GridOverlay::build(device, &self.project.grid, HEX_SIZE);
            self.path_mesh =
                PathMesh::build(device, &self.project.paths);
            self.decorator_mesh =
                DecoratorMesh::build(device, &self.project.grid, HEX_SIZE);

            // Rebuild hex data texture
            let (tex, view) = Self::create_hex_data_texture(&mut self.project.grid, device, queue);
            self.hex_data_texture = tex;
            self.hex_data_view = view;

            // Rebuild terrain bind group with new hex data view
            let terrain_bind_group_layout =
                self.terrain_pipeline.get_bind_group_layout(1);
            self.terrain_bind_group =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("terrain_bind_group"),
                    layout: &terrain_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &self.terrain_textures.view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(
                                &self.terrain_textures.sampler,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(
                                &self.hex_data_view,
                            ),
                        },
                    ],
                });

            self.needs_mesh_rebuild = false;
        }
    }

    pub fn render(&mut self, window: &Window) {
        let surface_texture = match self.gpu.surface.get_current_texture() {
            Ok(tex) => tex,
            Err(wgpu::SurfaceError::Lost) => {
                let size = window.inner_size();
                self.gpu.resize(size.width, size.height);
                return;
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                log::error!("Out of GPU memory");
                return;
            }
            Err(e) => {
                log::warn!("Surface error: {:?}", e);
                return;
            }
        };

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.gpu.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            },
        );

        // Update camera uniform
        let camera_uniform = self.camera.build_uniform(
            self.gpu.config.width as f32,
            self.gpu.config.height as f32,
            HEX_SIZE,
            self.project.grid.grid_offset,
        );
        self.gpu.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );

        // --- egui frame ---
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.gpu.config.width, self.gpu.config.height],
            pixels_per_point: window.scale_factor() as f32,
        };

        let raw_input = self.egui_state.take_egui_input(window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            self.map_rect =
                self.ui.show(ctx, &mut self.camera, None, (0.0, 0.0, 1.0, 1.0));
        });

        self.egui_state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(
                &self.gpu.device,
                &self.gpu.queue,
                *id,
                delta,
            );
        }
        self.egui_renderer.update_buffers(
            &self.gpu.device,
            &self.gpu.queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        // Compute viewport from the egui central panel rect
        let ppp = window.scale_factor() as f32;
        let vp_x = (self.map_rect.min.x * ppp) as u32;
        let vp_y = (self.map_rect.min.y * ppp) as u32;
        let vp_w = ((self.map_rect.width() * ppp) as u32).max(1);
        let vp_h = ((self.map_rect.height() * ppp) as u32).max(1);

        // --- Map render pass ---
        {
            let mut pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("map_pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &surface_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.25,
                                    g: 0.25,
                                    b: 0.25,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

            pass.set_viewport(
                vp_x as f32,
                vp_y as f32,
                vp_w as f32,
                vp_h as f32,
                0.0,
                1.0,
            );
            pass.set_scissor_rect(vp_x, vp_y, vp_w, vp_h);

            // Terrain
            pass.set_pipeline(&self.terrain_pipeline);
            pass.set_bind_group(0, &self.camera_bind_group, &[]);
            pass.set_bind_group(1, &self.terrain_bind_group, &[]);
            pass.set_vertex_buffer(0, self.hex_mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(
                self.hex_mesh.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            pass.draw_indexed(0..self.hex_mesh.num_indices, 0, 0..1);

            // Decorators
            if self.decorator_mesh.num_instances > 0 {
                pass.set_pipeline(&self.decorator_pipeline);
                pass.set_bind_group(0, &self.camera_bind_group, &[]);
                pass.set_vertex_buffer(
                    0,
                    self.decorator_mesh.instance_buffer.slice(..),
                );
                pass.draw(0..6, 0..self.decorator_mesh.num_instances);
            }

            // Paths
            if self.path_mesh.num_vertices > 0 {
                pass.set_pipeline(&self.path_pipeline);
                pass.set_bind_group(0, &self.camera_bind_group, &[]);
                pass.set_vertex_buffer(
                    0,
                    self.path_mesh.vertex_buffer.slice(..),
                );
                pass.draw(0..self.path_mesh.num_vertices, 0..1);
            }

            // Grid overlay
            if self.ui.show_grid && self.grid_overlay.num_vertices > 0 {
                pass.set_pipeline(&self.grid_pipeline);
                pass.set_bind_group(0, &self.camera_bind_group, &[]);
                pass.set_vertex_buffer(
                    0,
                    self.grid_overlay.vertex_buffer.slice(..),
                );
                pass.draw(0..self.grid_overlay.num_vertices, 0..1);
            }
        }

        // --- egui render pass (on top of the map) ---
        {
            let egui_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("egui_pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &surface_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            // egui-wgpu 0.31 requires RenderPass<'static>; use .forget_lifetime()
            let mut egui_pass_static = egui_pass.forget_lifetime();
            self.egui_renderer
                .render(&mut egui_pass_static, &tris, &screen_descriptor);
        }

        // Free egui textures
        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        // Submit and present
        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }
}

use wgpu::util::DeviceExt;
