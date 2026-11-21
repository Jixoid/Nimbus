#version 450
#extension GL_EXT_buffer_reference : require
#extension GL_EXT_scalar_block_layout : require
#extension GL_ARB_gpu_shader_int64 : require

struct RectInstance {
    vec4 rect;          // x, y, width, height
    vec4 radii;         // tl, tr, br, bl
    vec4 fill_color;
    vec4 border_color;
    vec4 shadow_color;
    vec2 shadow_offset;
    float border_width;
    float shadow_blur;
    float noise;
    vec3 _pad;
};

layout(buffer_reference, scalar) readonly buffer InstanceBuffer {
    RectInstance instances[];
};

layout(push_constant) uniform PushConstants {
    uint64_t instance_addr;
    vec2 screen_size;
} pc;

layout(location = 0) out vec2 v_local_pos;
layout(location = 1) out vec2 v_rect_half_size;
layout(location = 2) out vec4 v_radii;
layout(location = 3) out vec4 v_fill_color;
layout(location = 4) out vec4 v_border_color;
layout(location = 5) out vec4 v_shadow_color;
layout(location = 6) out vec2 v_shadow_offset;
layout(location = 7) out float v_border_width;
layout(location = 8) out float v_shadow_blur;
layout(location = 9) out float v_noise;

const vec2 QUAD_VERTICES[6] = vec2[6](
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 1.0)
);

void main() {
    InstanceBuffer buf = InstanceBuffer(pc.instance_addr);
    RectInstance inst = buf.instances[gl_InstanceIndex];

    vec2 uv = QUAD_VERTICES[gl_VertexIndex];

    float pad = inst.shadow_blur * 2.0;
    vec2 pad_min = vec2(pad) + max(-inst.shadow_offset, vec2(0.0));
    vec2 pad_max = vec2(pad) + max(inst.shadow_offset, vec2(0.0));

    vec2 rect_min = inst.rect.xy - pad_min;
    vec2 rect_max = inst.rect.xy + inst.rect.zw + pad_max;
    vec2 screen_pos = mix(rect_min, rect_max, uv);

    vec2 ndc = (screen_pos / pc.screen_size) * 2.0 - 1.0;
    gl_Position = vec4(ndc, 0.0, 1.0);

    vec2 rect_center = inst.rect.xy + inst.rect.zw * 0.5;
    v_local_pos = screen_pos - rect_center;
    v_rect_half_size = inst.rect.zw * 0.5;
    v_radii = inst.radii;
    v_fill_color = inst.fill_color;
    v_border_color = inst.border_color;
    v_shadow_color = inst.shadow_color;
    v_shadow_offset = inst.shadow_offset;
    v_border_width = inst.border_width;
    v_shadow_blur = inst.shadow_blur;
    v_noise = inst.noise;
}
