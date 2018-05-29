function loadJS(url) {
  return new Promise(function(resolve, reject) {
    var scriptTag = document.createElement('script');
    scriptTag.onload = resolve;
    scriptTag.onreadystatechange = resolve;
    scriptTag.src = url;
    document.head.appendChild(scriptTag);
  });
}


function loadScripts(scripts, callback) {
  const promises = scripts.map(loadJS);
  Promise.all(promises).then(callback);
}


function play_game(canvas, loaded_callback) {
  const scripts = [
    "twgl.min.js",
    "normalize_key.js"
  ];

  loadScripts(scripts, function() {
    console.log("All scripts loaded");
    actually_play_game(canvas, loaded_callback);
  });
}


function actually_play_game(canvas, loaded_callback) {
  // Width and Height in tiles:
  var width = 47;
  var height = 30;
  // TODO: see if we can reduce this to 18 px. That's the size we've empiricaly
  // determined to fit in the smaller laptops.
  var squareSize = 21;


  var c = canvas;
  console.log("Setting up the canvas", c);
  c.width = width*squareSize;
  c.height = height*squareSize;
  const gl = canvas.getContext("webgl");
  gl.enable(gl.BLEND);
  gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
  const texture = twgl.createTexture(gl, {src: "font.png"});


  var programInfo = null;
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


  var wasm_env = {
    random: Math.random,
    sin: Math.sin,
    draw: function(ptr, len, texture_width_px, texture_height_px) {
      const bytesInFloat = 4;
      // NOTE: both ptr and len are assuming a byte array. So we
      // have to divide by 4 to get the floats.
      const memory = new Float32Array(wasm_instance.exports.memory.buffer, ptr, len / bytesInFloat);
      const packedBuffer = twgl.createBufferFromTypedArray(gl, memory);
      const floatsPerElement = 8;
      const stride = floatsPerElement * bytesInFloat;
      const bufferInfo = {
        numElements: memory.length / floatsPerElement,
        attribs: {
          pos_px: {
            buffer: packedBuffer,
            numComponents: 2,
            type: gl.FLOAT,
            stride: stride,
            offset: 0 * bytesInFloat,
            drawType: gl.DYNAMIC_DRAW
          },
          tile_pos_px: {
            buffer: packedBuffer,
            numComponents: 2,
            type: gl.FLOAT,
            stride: stride,
            offset: 2 * bytesInFloat,
            drawType: gl.DYNAMIC_DRAW
          },
          color:  {
            buffer: packedBuffer,
            numComponents: 4,
            type: gl.FLOAT,
            stride: stride,
            offset: 4 * bytesInFloat,
            drawType: gl.DYNAMIC_DRAW
          }
        }
      };

      twgl.resizeCanvasToDisplaySize(gl.canvas);
      gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
      gl.clearColor(0.0, 0.0, 0.0, 1.0);
      gl.clear(gl.COLOR_BUFFER_BIT);

      const uniforms = {
        native_display_px: [gl.canvas.width, gl.canvas.height],
        display_px: [gl.canvas.width, gl.canvas.height],
        extra_px: [0, 0],
        texture_size_px: [texture_width_px, texture_height_px],
        tex: texture
      };

      gl.useProgram(programInfo.program);
      twgl.setBuffersAndAttributes(gl, programInfo, bufferInfo);
      twgl.setUniforms(programInfo, uniforms);
      twgl.drawBufferInfo(gl, bufferInfo);

      // NOTE: We must delete the buffer or it will stay in memory
      // forever and will leak more and more every tick.
      gl.deleteBuffer(packedBuffer);
    }
  };


  console.log("Fetching resources");

  let wasm_promise = fetch("dose-response.wasm")
    .then(function(response) {
      return response.arrayBuffer();
    })
    .then(function(bytes) {
      return WebAssembly.instantiate(bytes, {
        env: wasm_env
      });
    });


  let vertex_shader_promise = fetch("webgl_vertex_shader.glsl")
      .then(function(response) {
        return response.text();
      })
      .then(function(text) {
        console.log("Leaded vertex shader text.");
        return text;
      });


  let fragment_shader_promise = fetch("webgl_fragment_shader.glsl")
      .then(function(response) {
        return response.text();
      })
      .then(function(text) {
        console.log("Leaded fragment shader text.");
        return text;
      });

  Promise.all([wasm_promise, vertex_shader_promise, fragment_shader_promise])
    .then(function(results) {
      console.log("All resources loaded.");
      const wasm_result = results[0];
      const vertex_shader = results[1];
      const fragment_shader = results[2];

      if(loaded_callback) {
        loaded_callback();
      }

      wasm_instance = wasm_result.instance;
      gamestate_ptr = wasm_result.instance.exports.initialise();
      programInfo = twgl.createProgramInfo(gl, [vertex_shader, fragment_shader]);


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

        wasm_result.instance.exports.update(
          gamestate_ptr, dt,
          mouse.tile_x, mouse.tile_y, mouse.pixel_x, mouse.pixel_y,
          mouse.left, mouse.right);
        mouse.left = false;
        mouse.right = false;
      }

      update(previous_frame_timestamp);
    });
}
