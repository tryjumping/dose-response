#version 150 core

uniform vec2 tile_count;
// Window size (rendering area) in pixels
uniform vec2 native_display_px;
// Actual display size in pixels. Has the same aspect ratio as
//`native_display_px`, but can be bigger say on fullscreen.
uniform vec2 display_px;
// Additional empty space. If the final rendering area has a different
//aspect ratio, this contains the extra space so we can letterbox or
//whatever.
uniform vec2 extra_px;

in vec2 pos_px;
in vec2 tilemap_index;
in vec4 color;

out vec2 v_tilemap_index;
out vec4 v_color;

void main() {
    v_tilemap_index = tilemap_index;
    v_color = color;

    // This is the full size of the rendered area (window / screen) in pixels
    vec2 full_dimension_px = display_px + extra_px;

    // `pos_px / native_display_px` converts the coordinates to (0, 1)
    // in the native pixel-perfect space. This stretches the image to
    // fit the entire window.
    //
    // `* (display_px / full_dimension_px)` fixes the aspect ratio.
	//
	// `+ (0.5 * extra_px / full_dimension_px)` centeres the image,
	// letterboxing it.
    vec2 pos_fit_to_screen = pos_px / native_display_px * (display_px / full_dimension_px) + (0.5 * extra_px / full_dimension_px);

	// Convert the y-is-down (0, 1) coordinate system to OpenGl's
	// y-is-up, (-1, 1)
    vec2 pos = vec2(1.0, -1.0) * (2.0 * pos_fit_to_screen - 1.0);
    gl_Position = vec4(pos, 0.0, 1.0);
}
