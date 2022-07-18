#version 300 es
precision mediump float;
layout(std140) uniform;

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;

uniform Stuff {
    float vifo_theta;
    vec2 pos_deltas;
    mat4 norm_mat;
};

mat4 BuildTranslation(vec3 delta)
{
    return mat4(
        vec4(1.0, 0.0, 0.0, 0.0),
        vec4(0.0, 1.0, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(delta, 1.0));
}

mat2 r2d(float a) {
	float c = cos(a), s = sin(a);
    return mat2(
        c, s, // column 1
        -s, c // column 2
    );
}

mat3 rotate_translate(float theta, float dx, float dy) {
    float c = cos(theta), s = sin(theta);
    return mat3(
        c, -s, dx,
        s, c, dy,
        0, 0, 1
    );
}

void main() {
    vifo_theta;
    pos_deltas;
    norm_mat;

    a_position;
    a_normal;

    mat2 t_2 = r2d(vifo_theta);
    mat3 transform = rotate_translate(vifo_theta, pos_deltas[0], pos_deltas[1]);
    vec2 p_3 = t_2 * a_position.xy;
    vec4 mediate = vec4(p_3[0] + pos_deltas[0], p_3[1] + pos_deltas[1], a_position.z, 1.0);
    vec3 translate_delta = vec3(0.0, 0.0, 0.5);
    mat4 tr_mat = BuildTranslation(translate_delta);
    vec4 translated = mediate * tr_mat;
    gl_Position = translated;

    // gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
}
