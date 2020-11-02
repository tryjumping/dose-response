#version 150 core

uniform sampler2D textmap;
uniform vec2 textmap_size_px;
uniform sampler2D glyphmap;
uniform vec2 glyphmap_size_px;
uniform sampler2D tilemap;
uniform vec2 tilemap_size_px;
uniform sampler2D eguimap;
uniform vec2 eguimap_size_px;

flat in float v_texture_id;
// NOTE: the units for this depend on the texture. For egui it's a normalised
// <0, 1> real value, for everything else it's a texture coordinate in pixels.
in vec2 v_tile_pos;
in vec2 v_vertex_pos_px;
in vec4 v_color;

out vec4 out_color;

void main() {
  // NOTE: egui outputs texture coordinates
  if (v_tile_pos.x < 0.0 && v_tile_pos.y < 0.0) {
    out_color = v_color;
  } else if (v_texture_id == 0.0) {
    out_color = texture(textmap, v_tile_pos / textmap_size_px) * v_color;
  } else if (v_texture_id == 1.0) {
    out_color = texture(glyphmap, v_tile_pos / glyphmap_size_px) * v_color;
  } else if (v_texture_id == 2.0) {
    out_color = texture(tilemap, v_tile_pos / tilemap_size_px) * v_color;
  } else if (v_texture_id == 3.0) {
    // NOTE: egui outputs texture coordinates (uv aka v_tile_pos) already normalised.
    // That means we shouldn't divide them by eguimap_size_px as they're in the [0, 1] range already.
    out_color = texture(eguimap, v_tile_pos) * v_color;
  }
}
