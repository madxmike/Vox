#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec3 v_color;

layout(set = 0, binding = 0) uniform MVP {
    mat4 model;
    mat4 view;
    mat4 projection;
} mvp;

void main() {
    mat4 world = mvp.view * mvp.model;
    v_normal = normal;
    v_color = v_normal;
    gl_Position = mvp.projection * world * vec4(position, 1.0);

}