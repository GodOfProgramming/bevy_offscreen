use bevy::{
    camera::{
        ImageRenderTarget, RenderTarget,
        visibility::{Layer, RenderLayers},
    },
    prelude::*,
    render::render_resource::AsBindGroup,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
    window::PrimaryWindow,
};
use bevy_offscreen::{
    OffscreenCamera, OffscreenResizeEvent, get_viewport_size, sync::OffscreenCameraSyncPlugin,
};
use wgpu_types::TextureFormat;

const GRAYSCALE_LAYER: Layer = 1;
const COLOR_ID_LAYER: Layer = 2;

const RECTANGLE_SIZE: f32 = 128.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            // OffscreenCameraSyncPlugin::<MainCamera>::new(),
            Material2dPlugin::<FinalMaterial>::default(),
            Material2dPlugin::<ColorIdMaterial>::default(),
        ))
        .add_observer(handle_resizes)
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut color_id_materials: ResMut<Assets<ColorIdMaterial>>,
    mut final_materials: ResMut<Assets<FinalMaterial>>,
) {
    let image_size = window.physical_size();
    let grayscale_image =
        Image::new_target_texture(image_size.x, image_size.y, TextureFormat::Rgba8UnormSrgb);
    let grayscale_image_handle = images.add(grayscale_image);

    let color_id_image =
        Image::new_target_texture(image_size.x, image_size.y, TextureFormat::Rgba8UnormSrgb);
    let color_id_image_handle = images.add(color_id_image);

    let rect_mesh = meshes.add(Rectangle::from_size(Vec2::splat(RECTANGLE_SIZE)));

    let grayscale_rect_mat = color_materials.add(ColorMaterial::from_color(Color::linear_rgb(
        0.50, 0.50, 0.50,
    )));

    commands.spawn((
        OffscreenCameraRect {
            render_layers: RenderLayers::layer(GRAYSCALE_LAYER),
            mesh: Mesh2d(rect_mesh.clone()),
            material: MeshMaterial2d(grayscale_rect_mat.clone()),
            transform: Transform::from_translation(Vec3::new(-RECTANGLE_SIZE * 2.0, 0.0, 0.0)),
        },
        Children::spawn(Spawn(OffscreenCameraRect {
            render_layers: RenderLayers::layer(COLOR_ID_LAYER),
            mesh: Mesh2d(rect_mesh.clone()),
            material: MeshMaterial2d(color_id_materials.add(ColorIdMaterial { color_id: 0.0 })),
            transform: Transform::default(),
        })),
    ));

    commands.spawn((
        OffscreenCameraRect {
            render_layers: RenderLayers::layer(GRAYSCALE_LAYER),
            mesh: Mesh2d(rect_mesh.clone()),
            material: MeshMaterial2d(grayscale_rect_mat.clone()),
            transform: Transform::from_translation(Vec3::new(RECTANGLE_SIZE * 2.0, 0.0, 0.0)),
        },
        Children::spawn(Spawn(OffscreenCameraRect {
            render_layers: RenderLayers::layer(COLOR_ID_LAYER),
            mesh: Mesh2d(rect_mesh.clone()),
            material: MeshMaterial2d(color_id_materials.add(ColorIdMaterial { color_id: 1.0 })),
            transform: Transform::default(),
        })),
    ));

    commands.spawn((
        MainCamera,
        Children::spawn((
            Spawn((
                OffscreenCamera::<MainCamera>::default(),
                RenderLayers::layer(GRAYSCALE_LAYER),
                Camera2d,
                Camera {
                    target: RenderTarget::Image(ImageRenderTarget::from(
                        grayscale_image_handle.clone(),
                    )),
                    clear_color: ClearColorConfig::Custom(Color::NONE),
                    ..default()
                },
            )),
            Spawn((
                OffscreenCamera::<MainCamera>::default(),
                RenderLayers::layer(COLOR_ID_LAYER),
                Camera2d,
                Camera {
                    target: RenderTarget::Image(ImageRenderTarget::from(
                        color_id_image_handle.clone(),
                    )),
                    clear_color: ClearColorConfig::Custom(Color::NONE),
                    ..default()
                },
            )),
            Spawn((
                Mesh2d(meshes.add(Rectangle::from_size(
                    get_viewport_size(None, &window).as_vec2(),
                ))),
                MeshMaterial2d(final_materials.add(FinalMaterial {
                    grayscale: grayscale_image_handle,
                    color_id: color_id_image_handle,
                    palette: [
                        Color::linear_rgb(0.25, 0.5, 0.75).to_linear().to_vec3(),
                        Color::linear_rgb(0.5, 0.75, 0.25).to_linear().to_vec3(),
                    ],
                })),
            )),
        )),
    ));
}

#[derive(Component)]
#[require(Camera2d)]
struct MainCamera;

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
    color_id: f32,
}

impl Material2d for ColorIdMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "color_id.wgsl".into()
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
