precision mediump float;

uniform sampler2D tex;
uniform vec2 texture_size_px;

varying vec2 v_tile_pos_px;
varying vec4 v_color;

void main() {
  if (v_tile_pos_px.x < 0.0 && v_tile_pos_px.y < 0.0) {
    gl_FragColor = v_color;
  } else {
    gl_FragColor = texture2D(tex, v_tile_pos_px / texture_size_px) * v_color;
  }
}
