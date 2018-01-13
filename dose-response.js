function play_game(canvas, wasm_path) {
  var width = 63;
  var height = 43;
  var squareSize = 16;

  var c = canvas;
  var ctx = c.getContext('2d');

  var wasm_instance;
  var gamestate_ptr;
  var pressed_keys = [];

  console.log("Fetching: ", wasm_path);

  fetch(wasm_path)
    .then(function(response) {
      return response.arrayBuffer();
    })

    .then(function(bytes) {
      return WebAssembly.instantiate(bytes, {
        env: {
          random: Math.random,
          draw: function(ptr, len) {
            memory = new Uint8Array(wasm_instance.exports.memory.buffer, ptr, len);

            ctx.clearRect(0, 0, width * squareSize, height * squareSize);

            var decoder = new msgpack.Decoder();
            decoder.on("data", function(chunk) {

              var discriminant = chunk[0];
              var data = chunk[1];

              switch(discriminant) {
              case 0:  // Char
                var x = data[0][0];
                var y = data[0][1];
                var glyph = data[1];
                var color = data[2];

                ctx.fillStyle = "rgb(" + color[0] + "," + color[1] + "," + color[2] + ")";
                var x_fudge = 8;
                var y_fudge = 13;
                ctx.fillText(glyph, x * squareSize + x_fudge, y * squareSize + y_fudge);
                break;

              case 1: // Background
                var x = data[0][0];
                var y = data[0][1];
                var color = data[1];

                ctx.fillStyle = "rgb(" + color[0] + "," + color[1] + "," + color[2] + ")";
                ctx.fillRect(x * squareSize, y * squareSize, squareSize, squareSize);
                break;

              case 2: // Text
                var x = data[0][0];
                var y = data[0][1];
                var text = data[1];
                var color = data[2];

                ctx.fillStyle = "rgb(" + color[0] + "," + color[1] + "," + color[2] + ")";
                var x_fudge = 8;
                var y_fudge = 13;
                ctx.fillText(text, x * squareSize + x_fudge, y * squareSize + y_fudge);
                break;

              case 3: // Rectangle
                var x = data[0][0];
                var y = data[0][1];
                var dimensions = data[1];
                var color = data[2];

                ctx.fillStyle = "rgb(" + color[0] + "," + color[1] + "," + color[2] + ")";
                ctx.fillRect(x * squareSize, y * squareSize, dimensions[0] * squareSize, dimensions[1] * squareSize);
                break;

              case 4: // Fade
                var alpha = 1 - data[0];
                var color = data[1];

                ctx.fillStyle = "rgba(" + color[0] + "," + color[1] + "," + color[2] + "," + alpha + ")";
                ctx.fillRect(0, 0, width * squareSize, height * squareSize);
                break;

              default:
                console.log("Unknown drawcall type:", discriminant);
              }
            });

            decoder.decode(memory);
            decoder.end();
          }
        }
      });
    })

    .then(function(results) {
      console.log("Wasm loaded.");

      console.log("Setting up the canvas", c);
      c.width = width*squareSize;
      c.height = height*squareSize;
      ctx.font = '14px mononoki';

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

        results.instance.exports.update(gamestate_ptr, dt);
      }

      console.log("Playing the game.");
      update(previous_frame_timestamp);
    });
}
