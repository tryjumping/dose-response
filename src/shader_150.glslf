#version 150 core

uniform sampler2D tex;
uniform vec2 texture_size_px;

in vec2 v_tile_pos_px;
in vec4 v_color;

out vec4 out_color;

void main() {
  if (v_tile_pos_px.x < 0.0 && v_tile_pos_px.y < 0.0) {
    out_color = v_color;
  } else {
    out_color = texture(tex, v_tile_pos_px / texture_size_px) * v_color;
  }
}
