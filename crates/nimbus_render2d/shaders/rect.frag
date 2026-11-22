#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) in vec2 v_local_pos;
layout(location = 1) in vec2 v_rect_half_size;
layout(location = 2) in vec4 v_radii;
layout(location = 3) in vec4 v_fill_color;
layout(location = 4) in vec4 v_border_color;
layout(location = 5) in vec4 v_shadow_color;
layout(location = 6) in vec2 v_shadow_offset;
layout(location = 7) in float v_border_width;
layout(location = 8) in float v_shadow_blur;
layout(location = 9) in float v_noise;
layout(location = 10) flat in uint v_texture_id;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D u_textures[];

float hash(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

float overlay_channel(float b, float n) {
    return (b < 0.5) ? (2.0 * b * n) : (1.0 - 2.0 * (1.0 - b) * (1.0 - n));
}

vec3 apply_overlay_noise(vec3 base, float noise_val, float intensity) {
    float n = mix(0.5, noise_val, intensity);
    return vec3(
        overlay_channel(base.r, n),
        overlay_channel(base.g, n),
        overlay_channel(base.b, n)
    );
}

float rounded_rect_sdf(vec2 p, vec2 b, vec4 r) {
    float rad = (p.x > 0.0) ? ((p.y > 0.0) ? r.z : r.y) : ((p.y > 0.0) ? r.w : r.x);
    rad = min(rad, min(b.x, b.y));
    vec2 q = abs(p) - b + vec2(rad);
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - rad;
}

void main() {
    float dist = rounded_rect_sdf(v_local_pos, v_rect_half_size, v_radii);
    float aa = fwidth(dist);
    if (aa < 0.001) aa = 0.001;

    // Main shape coverage
    float fill_coverage = clamp(0.5 - dist / aa, 0.0, 1.0);

    vec4 base_fill = v_fill_color;
    if (v_texture_id != 0xFFFFFFFFu) {
        vec2 rect_size = max(v_rect_half_size * 2.0, vec2(1.0));
        vec2 tex_uv = clamp((v_local_pos / rect_size) + vec2(0.5), vec2(0.0), vec2(1.0));
        base_fill *= texture(u_textures[nonuniformEXT(v_texture_id)], tex_uv);
    }

    // Border handling
    vec4 shape_color = base_fill;
    if (v_border_width > 0.0 && v_border_color.a > 0.0) {
        float inner_dist = dist + v_border_width;
        float inner_coverage = clamp(0.5 - inner_dist / aa, 0.0, 1.0);
        shape_color = mix(v_border_color, base_fill, inner_coverage);
    }

    if (v_noise > 0.0) {
        float h = hash(floor(v_local_pos));
        shape_color.rgb = apply_overlay_noise(shape_color.rgb, h, v_noise);
    }

    vec4 final_color = vec4(shape_color.rgb, shape_color.a * fill_coverage);

    // Soft shadow calculation
    if (v_shadow_blur > 0.0 && v_shadow_color.a > 0.0) {
        vec2 shadow_pos = v_local_pos - v_shadow_offset;
        float s_dist = rounded_rect_sdf(shadow_pos, v_rect_half_size, v_radii);
        float s_factor = clamp(0.5 - s_dist / (v_shadow_blur * 2.0), 0.0, 1.0);
        float s_alpha = s_factor * s_factor * (3.0 - 2.0 * s_factor) * v_shadow_color.a;
        vec4 shadow = vec4(v_shadow_color.rgb, s_alpha);

        // Standard alpha blend: final_color over shadow
        float out_a = final_color.a + shadow.a * (1.0 - final_color.a);
        if (out_a > 0.0001) {
            vec3 out_rgb = (final_color.rgb * final_color.a + shadow.rgb * shadow.a * (1.0 - final_color.a)) / out_a;
            final_color = vec4(out_rgb, out_a);
        } else {
            final_color = vec4(0.0);
        }
    }

    if (final_color.a <= 0.001) {
        discard;
    }

    out_color = final_color;
}
