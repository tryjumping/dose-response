#version 150 core

uniform sampler2D fontmap;
uniform vec2 fontmap_size_px;
uniform sampler2D tilemap;
uniform vec2 tilemap_size_px;

flat in float v_texture_id;
in vec2 v_tile_pos_px;
in vec4 v_color;

out vec4 out_color;

void main() {
  if (v_tile_pos_px.x < 0.0 && v_tile_pos_px.y < 0.0) {
    out_color = v_color;
  } else if (v_texture_id == 0.0) {
    out_color = texture(fontmap, v_tile_pos_px / fontmap_size_px) * v_color;
  } else {
    out_color = texture(tilemap, v_tile_pos_px / tilemap_size_px) * v_color;
  }
}
