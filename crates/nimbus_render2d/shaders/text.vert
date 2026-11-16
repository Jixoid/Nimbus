#version 450
#extension GL_EXT_buffer_reference : require
#extension GL_EXT_scalar_block_layout : require
#extension GL_ARB_gpu_shader_int64 : require

struct GlyphInstance {
    vec4 rect;
    vec4 uv_rect;
    vec4 color;
};

layout(buffer_reference, scalar) readonly buffer GlyphBuffer {
    GlyphInstance glyphs[];
};

layout(push_constant) uniform TextPushConstants {
    uint64_t glyph_addr;
    vec2 screen_size;
    uint texture_id;
    uint _pad;
} pc;

layout(location = 0) out vec2 v_uv;
layout(location = 1) out vec4 v_color;
layout(location = 2) flat out uint v_texture_id;

const vec2 QUAD_VERTICES[6] = vec2[6](
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 1.0)
);

void main() {
    GlyphBuffer buf = GlyphBuffer(pc.glyph_addr);
    GlyphInstance glyph = buf.glyphs[gl_InstanceIndex];

    vec2 corner = QUAD_VERTICES[gl_VertexIndex];
    vec2 screen_pos = glyph.rect.xy + glyph.rect.zw * corner;

    vec2 ndc = (screen_pos / pc.screen_size) * 2.0 - 1.0;
    gl_Position = vec4(ndc, 0.0, 1.0);

    v_uv = mix(glyph.uv_rect.xy, glyph.uv_rect.zw, corner);
    v_color = glyph.color;
    v_texture_id = pc.texture_id;
}
