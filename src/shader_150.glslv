#version 150 core

uniform vec2 tile_count;
uniform vec2 display_px;
uniform vec2 extra_px;

in vec2 tile_position;
in vec2 tilemap_index;
in vec4 color;

out vec2 v_tilemap_index;
out vec4 v_color;

void main() {
    v_tilemap_index = tilemap_index;
    v_color = color;

    vec2 full_dimension_px = display_px + extra_px;
    vec2 tile_pos = (tile_position / tile_count);
    vec2 tile_pos_in_display_space = tile_pos * (display_px / full_dimension_px);
    vec2 tile_pos_offseted = tile_pos_in_display_space + (0.5 * extra_px / full_dimension_px);

    vec2 pos = vec2(1.0, -1.0) * (2.0 * tile_pos_offseted - 1.0);
    gl_Position = vec4(pos, 0.0, 1.0);
}
