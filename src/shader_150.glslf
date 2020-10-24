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
in vec2 v_tile_pos_px;
in vec2 v_vertex_pos_px;
in vec4 v_color;

out vec4 out_color;

void main() {
  if (v_tile_pos_px.x < 0.0 && v_tile_pos_px.y < 0.0) {
    out_color = v_color;
  } else if (v_texture_id == 0.0) {
    out_color = texture(textmap, v_tile_pos_px / textmap_size_px) * v_color;
  } else if (v_texture_id == 1.0) {
    out_color = texture(glyphmap, v_tile_pos_px / glyphmap_size_px) * v_color;
  } else if (v_texture_id == 2.0) {
    out_color = texture(tilemap, v_tile_pos_px / tilemap_size_px) * v_color;
  } else if (v_texture_id == 3.0) {
    // NOTE: egui outputs texture coordinates (uv aka v_tile_pos_px) already normalised.
    // That means we shouldn't divide them by eguimap_size_px as they're in the [0, 1] range already.
    // TODO: "denormalise" them again so the shader is consistent?
    // Alternatively: normalise all the other texture coordinates.
    // TODO: rename `v_tile_pos_px` to `v_tile_pos_01` because it's NOT pixels anymore.
    out_color = texture(eguimap, v_tile_pos_px) * v_color;
  }
}
