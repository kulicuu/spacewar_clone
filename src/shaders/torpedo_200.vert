#version 300 es

precision mediump float;
uniform vec2 pos_deltas;
uniform float vifo_theta;

in vec3 b_position;

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

void main() {
    mat2 t_2 = r2d(vifo_theta);

    vec2 p_3 = t_2 * b_position.xy;


    vec4 mediate = vec4(p_3[0] + pos_deltas[0], p_3[1] + pos_deltas[1], b_position.z, 1.0);

    vec3 translate_delta = vec3(0.0, 0.0, 0.5);
    mat4 tr_mat = BuildTranslation(translate_delta);

    vec4 translated = mediate * tr_mat;

    gl_Position = translated;

    // gl_Position = translated;


    // gl_Position = vec4(p_3[0] + pos_deltas[0], p_3[1] + pos_deltas[1], 0.0, 1.0);

    // gl_Position = vec4(b_position, 0.0, 1.0);
}