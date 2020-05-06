#version 150 core

// Game display size in pixels. This covers the entire rendering area of the
// game. The OpenGL viewport is set ot be this times the DPI so there are no
// extra pixels as far as the shaders are concerned.
uniform vec2 display_px;

in float texture_id;
in vec2 pos_px;
in vec2 tile_pos_px;
in vec4 color;

flat out float v_texture_id;
out vec2 v_tile_pos_px;
out vec4 v_color;

void main() {
    v_texture_id = texture_id;
    v_tile_pos_px = tile_pos_px;
    v_color = color;

    // `pos_px / display_px` converts the coordinates to (0, 1)
    // in the native pixel-perfect space. This stretches the image to
    // fit the entire viewport.
    vec2 pos_fit_to_screen = pos_px / display_px;

	// Convert the y-is-down (0, 1) coordinate system to OpenGl's
	// y-is-up, (-1, 1)
    vec2 pos = vec2(1.0, -1.0) * (2.0 * pos_fit_to_screen - 1.0);
    gl_Position = vec4(pos, 0.0, 1.0);
}
