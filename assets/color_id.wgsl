#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> color_id: u32;

@fragment
fn fragment(input: VertexOutput) -> @location(0) u32 {
  return color_id;
}
