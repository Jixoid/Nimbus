#version 450

layout(location = 0) in vec2 v_pos;
layout(location = 1) flat in vec2 v_p0;
layout(location = 2) flat in vec2 v_p1;
layout(location = 3) flat in vec2 v_p2;
layout(location = 4) flat in float v_half_thickness;
layout(location = 5) flat in vec4 v_color;

layout(location = 0) out vec4 out_color;

float sd_segment(vec2 p, vec2 a, vec2 b) {
    vec2 pa = p - a;
    vec2 ba = b - a;
    float denom = dot(ba, ba);
    float h = (denom > 1e-6) ? clamp(dot(pa, ba) / denom, 0.0, 1.0) : 0.0;
    return length(pa - ba * h);
}

float sd_quadratic_bezier(vec2 pos, vec2 A, vec2 B, vec2 C) {
    vec2 a = B - A;
    vec2 b = A - 2.0 * B + C;
    vec2 c = a * 2.0;
    vec2 d = A - pos;

    float bb = dot(b, b);
    if (bb < 1e-4) {
        return sd_segment(pos, A, C);
    }

    float kk = 1.0 / bb;
    float kx = kk * dot(a, b);
    float ky = kk * (2.0 * dot(a, a) + dot(d, b)) / 3.0;
    float kz = kk * dot(d, a);

    float p = ky - kx * kx;
    float p3 = p * p * p;
    float q = kx * (2.0 * kx * kx - 3.0 * ky) + kz;
    float h = q * q + 4.0 * p3;

    float res;
    if (h >= 0.0) {
        h = sqrt(h);
        vec2 x = (vec2(h, -h) - q) * 0.5;
        vec2 uv = sign(x) * pow(abs(x), vec2(1.0 / 3.0));
        float t = clamp(uv.x + uv.y - kx, 0.0, 1.0);
        vec2 qv = d + (c + b * t) * t;
        res = dot(qv, qv);
    } else {
        float z = sqrt(-p);
        float v = acos(clamp(q / (p * z * 2.0), -1.0, 1.0)) / 3.0;
        float m = cos(v);
        float n = sin(v) * 1.732050808;
        vec3 t = clamp(vec3(m + m, -n - m, n - m) * z - kx, 0.0, 1.0);
        vec2 qx = d + (c + b * t.x) * t.x;
        vec2 qy = d + (c + b * t.y) * t.y;
        res = min(dot(qx, qx), dot(qy, qy));
    }

    return sqrt(res);
}

void main() {
    float dist = sd_quadratic_bezier(v_pos, v_p0, v_p1, v_p2) - v_half_thickness;
    float aa = max(fwidth(dist), 0.75);
    float coverage = clamp(0.5 - dist / aa, 0.0, 1.0);
    float alpha = v_color.a * coverage;

    if (alpha <= 0.001) {
        discard;
    }

    out_color = vec4(v_color.rgb, alpha);
}
