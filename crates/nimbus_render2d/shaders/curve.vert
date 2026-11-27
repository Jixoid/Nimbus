#version 450
#extension GL_EXT_buffer_reference : require
#extension GL_EXT_scalar_block_layout : require
#extension GL_EXT_buffer_reference_uvec2 : require

struct CurveInstance {
    vec2 p0;
    vec2 p1;
    vec2 p2;
    float thickness;
    float _pad;
    vec4 color;
};

layout(buffer_reference, scalar) readonly buffer CurveBuffer {
    CurveInstance instances[];
};

layout(push_constant) uniform PushConstants {
    uvec2 instance_addr;
    vec2 screen_size;
} pc;

layout(location = 0) out vec2 v_pos;
layout(location = 1) flat out vec2 v_p0;
layout(location = 2) flat out vec2 v_p1;
layout(location = 3) flat out vec2 v_p2;
layout(location = 4) flat out float v_half_thickness;
layout(location = 5) flat out vec4 v_color;

const vec2 QUAD_VERTICES[6] = vec2[6](
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 1.0)
);

void main() {
    CurveBuffer buf = CurveBuffer(pc.instance_addr);
    CurveInstance inst = buf.instances[gl_InstanceIndex];

    vec2 uv = QUAD_VERTICES[gl_VertexIndex];
    float pad = inst.thickness * 0.5 + 2.0;

    vec2 bmin = min(min(inst.p0, inst.p1), inst.p2) - vec2(pad);
    vec2 bmax = max(max(inst.p0, inst.p1), inst.p2) + vec2(pad);
    vec2 screen_pos = mix(bmin, bmax, uv);

    vec2 ndc = (screen_pos / pc.screen_size) * 2.0 - 1.0;
    gl_Position = vec4(ndc, 0.0, 1.0);

    v_pos = screen_pos;
    v_p0 = inst.p0;
    v_p1 = inst.p1;
    v_p2 = inst.p2;
    v_half_thickness = inst.thickness * 0.5;
    v_color = inst.color;
}
