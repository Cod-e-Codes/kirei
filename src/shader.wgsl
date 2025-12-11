// Vertex shader
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) rect_pos: vec2<f32>,
    @location(4) rect_size: vec2<f32>,
    @location(5) radius: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) rect_pos: vec2<f32>,
    @location(3) rect_size: vec2<f32>,
    @location(4) radius: f32,
    @location(5) pixel_pos: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> screen_size: vec2<f32>;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Convert from pixel coordinates (0,0 top-left) to clip space (-1,-1 to 1,1)
    let x = (model.position.x / screen_size.x) * 2.0 - 1.0;
    let y = 1.0 - (model.position.y / screen_size.y) * 2.0;
    
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.color = model.color;
    out.tex_coords = model.tex_coords;
    out.rect_pos = model.rect_pos;
    out.rect_size = model.rect_size;
    out.radius = model.radius;
    out.pixel_pos = model.position;
    return out;
}

// Fragment shader
@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

// Signed distance field for rounded rectangle
fn sdf_rounded_rect(pixel_pos: vec2<f32>, rect_pos: vec2<f32>, rect_size: vec2<f32>, radius: f32) -> f32 {
    // Calculate position relative to rect center
    let rect_center = rect_pos + rect_size * 0.5;
    let half_size = rect_size * 0.5;
    let pos = abs(pixel_pos - rect_center);
    
    // Distance to rounded corner
    let corner_dist = pos - half_size + vec2<f32>(radius, radius);
    let outside_dist = length(max(corner_dist, vec2<f32>(0.0, 0.0)));
    let inside_dist = min(max(corner_dist.x, corner_dist.y), 0.0);
    
    return outside_dist + inside_dist - radius;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture and multiply by vertex color
    var tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    var final_color = tex_color * in.color;
    
    // Apply rounded corners if radius > 0
    if (in.radius > 0.0) {
        let dist = sdf_rounded_rect(in.pixel_pos, in.rect_pos, in.rect_size, in.radius);
        
        // Smooth antialiasing
        let alpha = 1.0 - smoothstep(-1.0, 1.0, dist);
        final_color.a *= alpha;
    }
    
    return final_color;
}
