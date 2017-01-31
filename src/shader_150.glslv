#version 150 core

uniform vec2 world_dimensions;

in vec2 tile_position;
in vec2 tilemap_index;
in vec3 color;

out vec2 v_tilemap_index;
out vec3 v_color;

void main() {
    v_tilemap_index = tilemap_index;
    v_color = color;

    vec2 pos = vec2(1.0, -1.0) * ((2.0 * tile_position / world_dimensions) - 1.0);
    gl_Position = vec4(pos, 0.0, 1.0);
}
