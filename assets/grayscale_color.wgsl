#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var grayscale_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var grayscale_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(2) var color_id_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var color_id_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(4) var<uniform> palette: array<vec3<f32>, 2>;

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
  let grayscale_color = textureSample(grayscale_tex, grayscale_sampler, input.uv);
  let color_id = textureSample(color_id_tex, color_id_sampler, input.uv);
  let color = palette[color_id];
  return vec4<f32>(mix(grayscale_color, color, 0.5), grayscale_color.a);
}
