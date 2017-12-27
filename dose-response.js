var width = 63;
var height = 43;
var squareSize = 16;
var c = document.createElement('canvas');

c.width = width*squareSize;
c.height = height*squareSize;
document.body.append(c);

var ctx = c.getContext('2d');
ctx.textAlign = "center";
ctx.font = '14px mononoki';

var wasm_instance;
var gamestate_ptr;
var pressed_keys = [];


fetch('target/wasm32-unknown-unknown/release/dose-response.wasm')
  .then(response => response.arrayBuffer())

  .then(bytes => WebAssembly.instantiate(bytes, {
    env: {
      random: Math.random,
      draw: function(ptr, len) {
        if(len % 6 != 0) {
          throw new Error("The drawcalls vector must have a multiple of 6 elements!");
        }

        memory = new Uint8Array(wasm_instance.exports.memory.buffer, ptr, len);

        ctx.clearRect(0, 0, width * squareSize, height * squareSize);

        for(let i = 0; i < len; i += 6) {
          let x = memory[i + 0];
          let y = memory[i + 1];
          var glyph = null;
          if(memory[i + 2] != 0) {
            glyph = String.fromCharCode(memory[i + 2]);
          }
          let r = memory[i + 3];
          let g = memory[i + 4];
          let b = memory[i + 5];

          // NOTE: (255, 255) position means fade
          if(x == 255 && y == 255) {
            // NOTE: alpha is stored in the glyph position
            let alpha = memory[i + 2] / 255;  // convert the "alpha" to <0, 1>
            ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${alpha})`;
            ctx.fillRect(0, 0, width * squareSize, height * squareSize);
          } else if(glyph === null) {
            ctx.fillStyle = `rgb(${r},${g},${b})`;
            ctx.fillRect(x * squareSize, y * squareSize, squareSize, squareSize);
          } else {
            ctx.fillStyle = `rgb(${r},${g},${b})`;

            let x_fudge = 8;
            let y_fudge = 13;
            ctx.fillText(glyph, x * squareSize + x_fudge, y * squareSize + y_fudge);
          }
        }

      }
    }
  }))

  .then(results => {

    document.addEventListener('keydown', (event) => {
      pressed_keys.push(normalize_key(event));
    }, true);

    wasm_instance = results.instance;
    gamestate_ptr = results.instance.exports.initialise();

    var previous_frame_timestamp = 0;

    function update(timestamp) {
      window.requestAnimationFrame(update);
      let dt = timestamp - previous_frame_timestamp;
      previous_frame_timestamp = timestamp;

      for(let key of pressed_keys) {
        wasm_instance.exports.key_pressed(
          gamestate_ptr,
          key.numerical_code,
          key.ctrl, key.alt, key.shift);
      }
      pressed_keys = [];

      results.instance.exports.update(gamestate_ptr, dt);
    }
    update(previous_frame_timestamp);

  });
