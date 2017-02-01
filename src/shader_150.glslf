#version 150 core

uniform sampler2D tex;

uniform vec2 texture_gl_dimensions;

// Tile-based index into the tilemap
in vec2 v_tilemap_index;
in vec4 v_color;

out vec4 color;

void main() {
  if (v_tilemap_index == vec2(0.0, 5.0)) {
	color = v_color;
  } else {
    color = texture(tex, v_tilemap_index * texture_gl_dimensions) * v_color;
  }
}
