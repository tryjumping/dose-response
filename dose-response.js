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
                ctx.textAlign = "start";
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

                var text_options = data[3];
                var align = text_options[0][0];
                var wrap = text_options[1];
                var text_width = text_options[2];


                // TODO: implement fit_to_grid rendering!
                var fit_to_grid = text_options[3];

                switch(align) {
                case 0:
                  ctx.textAlign = "left";
                  break;
                case 1:
                  ctx.textAlign = "right";
                  break;
                case 2:
                  if(width > 0) {
                    ctx.textAlign = "center";
                    x += text_width / 2;
                  }
                  break;
                default:
                  ctx.textAlign = "left";
                }

                ctx.fillStyle = "rgb(" + color[0] + "," + color[1] + "," + color[2] + ")";
                var x_fudge = 8;
                var y_fudge = 13;

                if(wrap && text_width > 0) {
                  var lines = wrapText(ctx, text, text_width * squareSize);
                  // TODO: this duplicates the height calculation in `wrapped_text_height_in_tiles`!
                  var font_height_px = parseInt(ctx.font.match(/\d+/), 10);
                  var line_height_px = font_height_px * 1.3;
                  var line_height = Math.ceil(line_height_px / squareSize);
                  for(let i = 0; i < lines.length; i++) {
                    ctx.fillText(lines[i], x * squareSize + x_fudge, y * squareSize + y_fudge + (line_height_px * i));
                  }
                } else {
                  ctx.fillText(text, x * squareSize + x_fudge, y * squareSize + y_fudge);
                }
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
          },
          wrapped_text_height_in_tiles: function(text_ptr, text_len, max_width_in_tiles) {
            let buffer = new Uint8Array(wasm_instance.exports.memory.buffer, text_ptr, text_len);
            let decoder = new TextDecoder();
            let text = decoder.decode(buffer);
            let lines = wrapText(ctx, text, max_width_in_tiles * squareSize);
            return lines.length;
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

      // console.log("Playing the game.");
      // let text = "Hello world! This is an intentionally long text that is going to overflow at some point and so it is perfect for testing word-wrapping in this situation. We will probably have to expose the wrapText function to wasm or at least one that gives you the wrapped height in pixels or something.";
      // ctx.fillStyle = "rgb(255, 255, 255)";
      // var lines = wrapText(ctx, text, 200);
      // let fontHeight = parseInt(ctx.font.match(/\d+/), 10);
      // let lineHeight = (fontHeight * 1.3) | 0;
      // console.log(fontHeight, lineHeight);
      // for(let i = 0; i < lines.length; i++) {
      //   ctx.fillText(lines[i], 20, 20 + (lineHeight * i));
      // }
      update(previous_frame_timestamp);
    });
}
