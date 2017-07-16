// -*- mode: glsl; c-basic-offset: 4; -*-

#version 410 core

in vec3 position;
in vec4 color;

uniform mat4x4 model;
uniform mat3x3 model_normal;
layout (shared) uniform view_and_projection {
    mat4x4 view;
    mat4x4 view_inv;
    mat4x4 projection;
};

out vec4 v_color;

void main(void) {
    gl_Position = projection * view * model * vec4(position, 1.0);
    v_color = color;
}
