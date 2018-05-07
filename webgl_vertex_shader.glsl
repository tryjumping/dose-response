// Window size (rendering area) in pixels
uniform vec2 native_display_px;

attribute vec2 pos_px;
attribute vec2 tile_pos_px;
attribute vec4 color;

varying vec2 v_tile_pos_px;
varying vec4 v_color;

void main() {
  v_color = color;
  v_tile_pos_px = tile_pos_px;

  // Convert the pixel position to the (0, 1) coordinate space:
  vec2 pos_01 = pos_px / native_display_px;

  // Convert pos to the GL coordinate space (-1, 1), y grows up:
  vec2 pos_gl = vec2(1.0, -1.0) * (2.0 * pos_01 - 1.0);
  gl_Position = vec4(pos_gl, 0.0, 1.0);
}
