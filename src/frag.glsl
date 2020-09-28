#version 450

layout(location=0) out vec4 f_color;
layout(location=0) in vec2 v_tex;

layout(set=0, binding=0) uniform texture2D t_tex;
layout(set=0, binding=1) uniform sampler s_tex;

void main() {
    f_color = texture(sampler2D(t_tex, s_tex), v_tex);
}