#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec3 v_color;

layout(location = 0) out vec4 f_color;

const vec3 LIGHT = vec3(1.0, 4.0, 1.0);

void main() {
    float brightness = dot(normalize(v_normal), normalize(LIGHT));
    vec3 dark_color = vec3(1.0, 0.0, 0.0);

    f_color = vec4(mix(dark_color, v_color, brightness), 1.0);
}