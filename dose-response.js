var width = 63;
var height = 43;
var squareSize = 16;
var c = document.createElement('canvas');

c.width = width*squareSize;
c.height = height*squareSize;
document.body.append(c);

var ctx = c.getContext('2d');
ctx.textAlign = "center";
ctx.font = '16px arial';

var wasm_instance;
var gamestate_ptr;
var pressed_keys = [];

const keymap = {
  "1": 0,
  "2": 1,
  "3": 2,
  "4": 3,
  "5": 4,
  "6": 5,
  "7": 6,
  "8": 7,
  "9": 8,
  "0": 9,

  "a": 10,
  "b": 11,
  "c": 12,
  "d": 13,
  "e": 14,
  "f": 15,
  "g": 16,
  "h": 17,
  "i": 18,
  "j": 19,
  "k": 20,
  "l": 21,
  "m": 22,
  "n": 23,
  "o": 24,
  "p": 25,
  "q": 26,
  "r": 27,
  "s": 28,
  "t": 29,
  "u": 30,
  "v": 31,
  "w": 32,
  "x": 33,
  "y": 34,
  "z": 35,

  // `event.key` is uppercase when shift is pressed. But our Rust code
  // expects it to be the same as lowerase so we need to handle it
  // specially:

  "A": 10,
  "B": 11,
  "C": 12,
  "D": 13,
  "E": 14,
  "F": 15,
  "G": 16,
  "H": 17,
  "I": 18,
  "J": 19,
  "K": 20,
  "L": 21,
  "M": 22,
  "N": 23,
  "O": 24,
  "P": 25,
  "Q": 26,
  "R": 27,
  "S": 28,
  "T": 29,
  "U": 30,
  "V": 31,
  "W": 32,
  "X": 33,
  "Y": 34,
  "Z": 35,

  // Codes 36 - 45 are for the numpad

  // Codes 46 - 57 are for the F keys

  "ArrowLeft": 58,
  "ArrowRight": 59,
  "ArrowUp": 60,
  "ArrowDown": 61,
  "Enter": 62,
  " ": 63,
  "Escape": 64
};

const numpad_keymap = {
  "Numpad0": 36,
  "Numpad1": 37,
  "Numpad2": 38,
  "Numpad3": 39,
  "Numpad4": 40,
  "Numpad5": 41,
  "Numpad6": 42,
  "Numpad7": 43,
  "Numpad8": 44,
  "Numpad9": 45
};



fetch('target/wasm32-unknown-unknown/release/dose-response.wasm')
  .then(response => response.arrayBuffer())

  .then(bytes => WebAssembly.instantiate(bytes, {
    env: {
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
            ctx.fillText(glyph, x * squareSize + squareSize / 2, y * squareSize + squareSize / 2);
          }
        }

      }
    }
  }))

  .then(results => {

    document.addEventListener('keydown', (event) => {
      pressed_keys.push(event);
    });

    wasm_instance = results.instance;
    gamestate_ptr = results.instance.exports.initialise();

    var previous_frame_timestamp = 0;

    function update(timestamp) {
      window.requestAnimationFrame(update);
      let dt = timestamp - previous_frame_timestamp;
      previous_frame_timestamp = timestamp;

      for(let key of pressed_keys) {
        var key_code = -1;
        if(key.key in keymap) {
          key_code = keymap[key.key];
        }
        if(key.code.startsWith("Numpad")) {
          key_code = numpad_keymap[key.code];
        }
        wasm_instance.exports.key_pressed(
          gamestate_ptr,
          key_code,
          key.ctrlKey, key.altKey, key.shiftKey);
      }
      pressed_keys = [];

      results.instance.exports.update(gamestate_ptr, dt);
    }
    update(previous_frame_timestamp);

  });
