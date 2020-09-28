#version 450


layout(location=0) in vec3 a_pos;
layout(location=1) in vec2 a_tex;

layout(location=0) out vec2 v_tex;

void main() {
    v_tex = a_tex;
    gl_Position = vec4(a_pos, 1.0);
}