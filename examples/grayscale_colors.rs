use bevy::{
    camera::visibility::{Layer, RenderLayers},
    prelude::*,
    render::render_resource::AsBindGroup,
    sprite_render::{AlphaMode2d, Material2d},
    window::PrimaryWindow,
};
use bevy_offscreen::{
    CameraType, OffscreenRenderingPlugin, OffscreenResizeEvent, get_viewport_size,
    new_offscreen_camera,
};
use wgpu_types::TextureFormat;

const GRAYSCALE_LAYER: Layer = 1;
const COLOR_ID_LAYER: Layer = 2;

const RECTANGLE_SIZE: f32 = 128.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            OffscreenRenderingPlugin::<MainCamera>::new(GRAYSCALE_LAYER),
        ))
        .init_asset::<FinalMaterial>()
        .init_asset::<ColorIdMaterial>()
        .add_observer(handle_resizes)
        .add_systems(Startup, startup)
        .run();
}

#[derive(Component)]
#[require(Camera2d)]
struct MainCamera;

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut color_id_materials: ResMut<Assets<ColorIdMaterial>>,
    mut final_materials: ResMut<Assets<FinalMaterial>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
) {
    let main_camera_entity = commands.spawn(MainCamera).id();

    let (_, grayscale_image) = new_offscreen_camera::<MainCamera, PrimaryWindow>(
        &mut commands,
        &mut images,
        main_camera_entity,
        None,
        &window,
        GRAYSCALE_LAYER,
        CameraType::Camera2d(Camera2d),
        TextureFormat::Rgba8UnormSrgb,
    );

    let (_, color_id_image) = new_offscreen_camera::<MainCamera, PrimaryWindow>(
        &mut commands,
        &mut images,
        main_camera_entity,
        None,
        &window,
        COLOR_ID_LAYER,
        CameraType::Camera2d(Camera2d),
        TextureFormat::R8Uint,
    );

    let rect_mesh = meshes.add(Rectangle::from_size(Vec2::splat(RECTANGLE_SIZE)));

    let grayscale_rect_mat = color_materials.add(ColorMaterial::from_color(Color::linear_rgb(
        0.50, 0.50, 0.50,
    )));

    let left_rect = commands
        .spawn(OffscreenCameraRect {
            render_layers: RenderLayers::layer(GRAYSCALE_LAYER),
            mesh: Mesh2d(rect_mesh.clone()),
            material: MeshMaterial2d(grayscale_rect_mat.clone()),
            transform: Transform::from_translation(Vec3::new(-RECTANGLE_SIZE * 2.0, 0.0, 0.0)),
        })
        .id();

    let right_rect = commands
        .spawn(OffscreenCameraRect {
            render_layers: RenderLayers::layer(GRAYSCALE_LAYER),
            mesh: Mesh2d(rect_mesh.clone()),
            material: MeshMaterial2d(grayscale_rect_mat.clone()),
            transform: Transform::from_translation(Vec3::new(RECTANGLE_SIZE * 2.0, 0.0, 0.0)),
        })
        .id();

    commands.spawn_batch([
        (
            ChildOf(left_rect),
            OffscreenCameraRect {
                render_layers: RenderLayers::layer(COLOR_ID_LAYER),
                mesh: Mesh2d(rect_mesh.clone()),
                material: MeshMaterial2d(color_id_materials.add(ColorIdMaterial { color_id: 0 })),
                transform: Transform::default(),
            },
        ),
        (
            ChildOf(right_rect),
            OffscreenCameraRect {
                render_layers: RenderLayers::layer(COLOR_ID_LAYER),
                mesh: Mesh2d(rect_mesh.clone()),
                material: MeshMaterial2d(color_id_materials.add(ColorIdMaterial { color_id: 1 })),
                transform: Transform::default(),
            },
        ),
    ]);

    // final post process

    commands.spawn((
        PostProcessRectMarker,
        Mesh2d(meshes.add(Rectangle::from_size(
            get_viewport_size(None, &window).as_vec2(),
        ))),
        MeshMaterial2d(final_materials.add(FinalMaterial {
            grayscale: grayscale_image,
            color_id: color_id_image,
            palette: [
                Color::linear_rgb(0.25, 0.5, 0.75).to_linear().to_vec3(),
                Color::linear_rgb(0.5, 0.75, 0.25).to_linear().to_vec3(),
            ],
        })),
        ChildOf(main_camera_entity),
    ));

    // middle
    commands.spawn((Mesh2d(rect_mesh), MeshMaterial2d(grayscale_rect_mat)));
}

#[derive(Component)]
struct PostProcessRectMarker;

#[derive(Bundle)]
struct OffscreenCameraRect<M: Material2d> {
    render_layers: RenderLayers,
    mesh: Mesh2d,
    material: MeshMaterial2d<M>,
    transform: Transform,
}

#[derive(Asset, AsBindGroup, Reflect, Clone)]
pub struct ColorIdMaterial {
    #[uniform(0)]
    color_id: u32,
}

impl Material2d for ColorIdMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "color_id.wgsl".into()
    }

    fn specialize(
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::sprite_render::Material2dKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let Ok(fragment) = descriptor.fragment_mut() else {
            return Ok(());
        };

        let target_state = bevy::render::render_resource::ColorTargetState {
            format: TextureFormat::R8Uint,
            blend: None,
            write_mask: bevy::render::render_resource::ColorWrites::all(),
        };

        fragment.targets = vec![Some(target_state)];

        Ok(())
    }
}

#[derive(Asset, AsBindGroup, Reflect, Clone)]
pub struct FinalMaterial {
    #[texture(0)]
    #[sampler(1)]
    grayscale: Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    color_id: Handle<Image>,

    #[uniform(4)]
    palette: [Vec3; 2],
}

impl Material2d for FinalMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "grayscale_color.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

fn handle_resizes(
    event: On<OffscreenResizeEvent<MainCamera>>,
    mut commands: Commands,
    q_rects: Query<Entity, With<PostProcessRectMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for entity in q_rects {
        commands.entity(entity).insert((Mesh2d(
            meshes.add(Rectangle::from_size(event.new_size.as_vec2())),
        ),));
    }
}
