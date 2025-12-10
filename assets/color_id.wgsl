#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> color_id: f32;

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(color_id, color_id, color_id, 1.0);
}
