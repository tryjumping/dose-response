function renderWebGL(canvas) {
  canvas.width = 800;
  canvas.height = 600;
  const gl = canvas.getContext("webgl");
  const programInfo = twgl.createProgramInfo(gl, ["vs", "fs"]);

  const texture = twgl.createTexture(gl, {src: "target/wasm32-unknown-unknown/release/build/dose-response-4dd4bb781a4647a3/out/font.png"});

  var data = new Float32Array([
  -1.0, 1.0,  // xy
  0.0, 0.0,   // uv
  1.0, 0, 0, 1.0,  // rgba

  1.0, 1.0,  // xy
  1.0, 0.0,  // uv
  0.0, 1.0, 0.0, 1.0,  // rgba

  0.0, 0.0,  // xy
  0.5, 1.0,  // uv
  0.0, 0.0, 1.0, 1.0,  // rgba

  -1.0, -1.0,  // xy
  0.9, 0.9,    // uv
  1.0, 0, 0, 1.0,  // rgba

  1.0, -1.0,  // xy
  0.9, 0.9,  // uv
  0.0, 1.0, 0.0, 1.0,  // rgba

  0.0, 0.0,  // xy
  0.9, 0.9,  // uv
  0.0, 0.0, 1.0, 1.0 // rgba

  ]);
  const packedBuffer = twgl.createBufferFromTypedArray(gl, data);

  const floatsPerElement = 8;
  const bytesInFloat = 4;
  const stride = floatsPerElement * bytesInFloat;
  const bufferInfo = {
      numElements: data.length / floatsPerElement,
      attribs: {
        position: { buffer: packedBuffer, numComponents: 2, type: gl.FLOAT, stride: stride, offset: 0 * bytesInFloat, drawType: gl.DYNAMIC_DRAW },
        tile_pos_px: {buffer: packedBuffer, numComponents: 2, type: gl.FLOAT, stride: stride, offset: 2 * bytesInFloat, drawType: gl.DYNAMIC_DRAW },
        color:  { buffer: packedBuffer, numComponents: 4, type: gl.FLOAT, stride: stride, offset: 4 * bytesInFloat, drawType: gl.DYNAMIC_DRAW },
      },
  };

  function render(time) {
    twgl.resizeCanvasToDisplaySize(gl.canvas);
    gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);

    gl.clearColor(1.0, 0.0, 1.0, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);

    const uniforms = {
      tex: texture
      //resolution: [gl.canvas.width, gl.canvas.height],
    };

    gl.useProgram(programInfo.program);
    twgl.setBuffersAndAttributes(gl, programInfo, bufferInfo);
    twgl.setUniforms(programInfo, uniforms);
    twgl.drawBufferInfo(gl, bufferInfo);

    requestAnimationFrame(render);
  }
  requestAnimationFrame(render);
}


function wrapText(ctx, text, maxWidth) {
  var lines = [];
  var words = text.split(" ");
  var currentLine = words[0];

  for(var i = 1; i < words.length; i++) {
    var word = words[i];
    var width = ctx.measureText(currentLine + " " + word).width;
    if(width < maxWidth) {
      currentLine += " " + word;
    } else {
      lines.push(currentLine);
      currentLine = word;
    }
  }
  lines.push(currentLine);
  return lines;
}


function play_game(canvas, wasm_path) {
  renderWebGL(canvas);
  return;

  var width = 47;
  var height = 30;
  var squareSize = 18;

  var c = canvas;
  var ctx = c.getContext('2d');

  var wasm_instance;
  var gamestate_ptr;
  var pressed_keys = [];
  var mouse = {
    tile_x: 0,
    tile_y: 0,
    pixel_x: 0,
    pixel_y: 0,
    left: false,
    right: false
  };
  var left_pressed_this_frame = false;
  var right_pressed_this_frame = false;

  console.log("Fetching: ", wasm_path);

  fetch(wasm_path)
    .then(function(response) {
      return response.arrayBuffer();
    })

    .then(function(bytes) {
      return WebAssembly.instantiate(bytes, {
        env: {
          random: Math.random,
          sin: Math.sin,
          draw: function(ptr, len) {
            memory = new Uint8Array(wasm_instance.exports.memory.buffer, ptr, len);

            var image = document.getElementById("spritesheet");

            ctx.clearRect(0, 0, width * squareSize, height * squareSize);

            var decoder = new msgpack.Decoder();
            decoder.on("data", function(chunk) {

              var discriminant = chunk[0];
              var data = chunk[1];

              switch(discriminant) {
              case 0:  // Rectangle
                var rect = data[0];
                var x = rect[0][0];
                var y = rect[0][1];
                var width = x - rect[1][0] - 1;
                var height = y - rect[1][1] - 1;

                var color = data[1][0];
                var alpha = data[1][1];
                ctx.fillStyle = "rgba(" + color[0] + "," + color[1] + "," + color[2] + "," + alpha + ")";
                //ctx.fillStyle = "rgb(" + color[0] + "," + color[1] + "," + color[2] + ")";
                ctx.fillRect(x, y, width, height);
                break;

              case 1: // Image
                //console.log(data);

                var src_top_left = data[0][0];
                var src_bottom_right = data[0][1];

                var src_w = src_bottom_right[0] - src_top_left[0] + 1;
                var src_h = src_bottom_right[1] - src_top_left[1] + 1;

                var dst_top_left = data[1][0];
                var dst_bottom_right = data[1][1];
                var dst_w = dst_bottom_right[0] - dst_top_left[0] + 1;
                var dst_h = dst_bottom_right[1] - dst_top_left[1] + 1;

                var color = data[2];
                // var x = data[0][0];
                // var y = data[0][1];
                // var color = data[1];

                //ctx.drawImage(image, 21, 0, 21, 21, 10, 10, 21, 21);
                // ctx.fillStyle = "rgb(" + color[0] + "," + color[1] + "," + color[2] + ")";
                ctx.fillStyle = "rgb(255, 0, 0)";
                ctx.drawImage(image,
                              src_top_left[0], src_top_left[1], src_w, src_h,
                              dst_top_left[0], dst_top_left[1], dst_w, dst_h);

                // ctx.fillRect(x * squareSize, y * squareSize, squareSize, squareSize);
                break;

              case 2: // Text
                // var x = data[0][0];
                // var y = data[0][1];
                // var text = data[1];
                // var color = data[2];

                // var text_options = data[3];
                // var align = text_options[0][0];
                // var wrap = text_options[1];
                // var text_width = text_options[2];


                // // TODO: implement fit_to_grid rendering!
                // var fit_to_grid = text_options[3];

                // switch(align) {
                // case 0:
                //   ctx.textAlign = "left";
                //   break;
                // case 1:
                //   ctx.textAlign = "right";
                //   break;
                // case 2:
                //   if(text_width > 0) {
                //     ctx.textAlign = "center";
                //     x += text_width / 2;
                //   }
                //   break;
                // default:
                //   ctx.textAlign = "left";
                // }

                // ctx.fillStyle = "rgb(" + color[0] + "," + color[1] + "," + color[2] + ")";
                // var x_fudge = 0;
                // var y_fudge = 15;

                // if(wrap && text_width > 0) {
                //   var lines = wrapText(ctx, text, text_width * squareSize);
                //   // TODO: this duplicates the height calculation in `wrapped_text_height_in_tiles`!
                //   var font_height_px = parseInt(ctx.font.match(/\d+/), 10);
                //   var line_height_px = font_height_px * 1.3;
                //   var line_height = Math.ceil(line_height_px / squareSize);
                //   for(let i = 0; i < lines.length; i++) {
                //     ctx.fillText(lines[i], x * squareSize + x_fudge, y * squareSize + y_fudge + (line_height_px * i));
                //   }
                // } else {
                //   ctx.fillText(text, x * squareSize + x_fudge, y * squareSize + y_fudge);
                // }
                break;

              case 3: // Rectangle
                // var x = data[0][0][0];
                // var y = data[0][0][1];
                // var bottom_right = data[0][1];
                // var rect_width = bottom_right[0] - x + 1;
                // var rect_height = bottom_right[1] - y + 1;
                // var color = data[1];

                // ctx.fillStyle = "rgb(" + color[0] + "," + color[1] + "," + color[2] + ")";
                // ctx.fillRect(x * squareSize, y * squareSize, rect_width * squareSize, rect_height * squareSize);
                break;

              case 4: // Fade
                // var alpha = 1 - data[0];
                // var color = data[1];

                // ctx.fillStyle = "rgba(" + color[0] + "," + color[1] + "," + color[2] + "," + alpha + ")";
                // ctx.fillRect(0, 0, width * squareSize, height * squareSize);
                break;

              default:
                console.log("Unknown drawcall type:", discriminant);
              }
            });

            decoder.decode(memory);
            decoder.end();
          },
          wrapped_text_height_in_tiles: function(text_ptr, text_len, max_width_in_tiles) {
            let buffer = new Uint8Array(wasm_instance.exports.memory.buffer, text_ptr, text_len);
            let decoder = new TextDecoder();
            let text = decoder.decode(buffer);
            let lines = wrapText(ctx, text, max_width_in_tiles * squareSize);
            return lines.length;
          },
          wrapped_text_width_in_tiles: function(text_ptr, text_len, max_width_in_tiles) {
            let buffer = new Uint8Array(wasm_instance.exports.memory.buffer, text_ptr, text_len);
            let decoder = new TextDecoder();
            let text = decoder.decode(buffer);
            let lines = wrapText(ctx, text, max_width_in_tiles * squareSize);
            var maxWidthPx = 0;
            for(let i = 0; i < lines.length; i++) {
              let width = ctx.measureText(lines[i]).width;
              if(maxWidthPx < width) {
                maxWidthPx = width;
              }
            }
            return Math.ceil( maxWidthPx / squareSize);
          }
        }
      });
    })

    .then(function(results) {
      console.log("Wasm loaded.");

      console.log("Setting up the canvas", c);
      c.width = width*squareSize;
      c.height = height*squareSize;
      ctx.font = '16px mononoki';

      document.addEventListener('keydown', function(event) {
        let key = normalize_key(event);

        // Prevent default for these keys. They will scroll the page
        // otherwise.
        let stopkeys = {
          "ArrowDown": true,
          "ArrowUp": true,
          "ArrowLeft": true,
          "ArrowRight": true,
          " ": true,
          "PageDown": true,
          "PageUp": true,
          "Home": true,
          "End": true
        };
        if(stopkeys[key.name]) {
          event.preventDefault();
        }

        if(key.numerical_code != 0) {
          pressed_keys.push(key);
        }
      }, true);

      var getMousePos = function(canvas, event) {
        var rect = canvas.getBoundingClientRect();
        let x = (event.clientX - rect.left) / (rect.right - rect.left) * canvas.width;
        let y = (event.clientY - rect.top) / (rect.bottom - rect.top) * canvas.height;
        let tile_x = x / squareSize;
        let tile_y = y / squareSize;
        if(x >= 0 && y >= 0 && x < canvas.width && y < canvas.height) {
          return {
            x: Math.floor(x),
            y: Math.floor(y),
            tile_x: Math.floor(tile_x),
            tile_y: Math.floor(tile_y)
          };
        } else {
          return null;
        }
      };

      document.addEventListener('mousemove', function(event) {
        let current_mouse = getMousePos(canvas, event);
        if(current_mouse) {
          mouse.pixel_x = current_mouse.x;
          mouse.pixel_y = current_mouse.y;
          mouse.tile_x = current_mouse.tile_x;
          mouse.tile_y = current_mouse.tile_y;
        }
      });
      document.addEventListener('mousedown', function(event) {
        let current_mouse = getMousePos(canvas, event);
        if(current_mouse) {
          mouse.pixel_x = current_mouse.x;
          mouse.pixel_y = current_mouse.y;
          mouse.tile_x = current_mouse.tile_x;
          mouse.tile_y = current_mouse.tile_y;
        }
      });
      document.addEventListener('mouseup', function(event) {
        let current_mouse = getMousePos(canvas, event);
        if(current_mouse) {
          mouse.pixel_x = current_mouse.x;
          mouse.pixel_y = current_mouse.y;
          mouse.tile_x = current_mouse.tile_x;
          mouse.tile_y = current_mouse.tile_y;
          if(event.button === 0) {
            mouse.left = true;
          } else if (event.button === 2) {
            mouse.right = true;
          }
        }
      });


      wasm_instance = results.instance;
      gamestate_ptr = results.instance.exports.initialise();

      var previous_frame_timestamp = 0;

      function update(timestamp) {
        window.requestAnimationFrame(update);
        let dt = timestamp - previous_frame_timestamp;
        previous_frame_timestamp = timestamp;

        for(var index = 0; index < pressed_keys.length; index++) {
          var key = pressed_keys[index];
          wasm_instance.exports.key_pressed(
            gamestate_ptr,
            key.numerical_code,
            key.ctrl, key.alt, key.shift);
        }
        pressed_keys = [];

        results.instance.exports.update(
          gamestate_ptr, dt,
          mouse.tile_x, mouse.tile_y, mouse.pixel_x, mouse.pixel_y,
          mouse.left, mouse.right);
        mouse.left = false;
        mouse.right = false;
      }

      update(previous_frame_timestamp);
    });
}
