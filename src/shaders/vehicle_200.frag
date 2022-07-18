#version 300 es

precision mediump float;

uniform float u_time;

// varying highp vec2 vTextureCoord;
// varying highp vec3 vLighting;

// uniform sampler2D uSampler;

out vec4 FragColor;

void main() {
    u_time;
    // vTextureCoord;
    // vLighting;
    // uSampler;
    // float r = sin(u_time * 0.9003);
    // float g = sin(u_time * 0.9005);
    // float b = sin(u_time * 0.9007);

    FragColor = vec4(0.999, 0.999, 0.888, 1.0);
    // FragColor = vec4(r, g, b, 1.0);
}
