#version 450

layout(location = 0) in vec3 position;

layout(location = 0) out vec3 v_color;

layout(set = 0, binding = 0) uniform MVP_Data {
    mat4 clip_space;
} mvp;

void main() {
    v_color = vec3(1.0, 0.0, 0.0);
    gl_Position = mvp.clip_space * vec4(position, 1.0);

}