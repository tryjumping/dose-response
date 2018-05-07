/// Process a `KeyboardEvent` into a simpler interface and add a
/// numerical value that's always consistent.
///
/// Calling: `normalize_key(event)` with `ctrl+up` returns:
///
/// {
///     name: "ArrowUp",
///     code: "ArrowUp",
///     numerical_code: 0xFF52,
///     ctrl: true,
///     alt: false,
///     shift: false
/// }
///
/// The numerical code is based on:
/// https://git.gnome.org/browse/gtk+/plain/gdk/gdkkeysyms.h

var normalize_key = (function () {
  const key_map = {
    // NOTE: these just return their ASCII codes
    "Backspace": 0xff08,  // GDK_KEY_BackSpace
    "Tab": 0xff09,  // GDK_KEY_Tab
    "Enter": 0xff0d,  // GDK_KEY_Return
    "Escape": 0xff1b,  // GDK_KEY_Escape
    "Delete": 0xffff,  // GDK_KEY_Delete

    "ArrowDown": 0xFF54,  // GDK_KEY_Down
    "ArrowLeft": 0xFF51,  // GDK_KEY_Left
    "ArrowRight": 0xFF53,  // GDK_KEY_Right
    "ArrowUp": 0xFF52,  // GDK_KEY_Up

    "End": 0xFF57,  // GDK_KEY_End
    "Home": 0xFF50,  // GDK_KEY_Home
    "PageDown": 0xFF56,  // GDK_KEY_Page_Down
    "PageUp": 0xFF55,  // GDK_KEY_Page_Up

    "F1": 0xFFBE,  // GDK_KEY_F1
    "F2": 0xFFBF,  // GDK_KEY_F2
    "F3": 0xFFC0,  // GDK_KEY_F3
    "F4": 0xFFC1,  // GDK_KEY_F4
    "F5": 0xFFC2,  // GDK_KEY_F5
    "F6": 0xFFC3,  // GDK_KEY_F6
    "F7": 0xFFC4,  // GDK_KEY_F7
    "F8": 0xFFC5,  // GDK_KEY_F8
    "F9": 0xFFC6,  // GDK_KEY_F9
    "F10": 0xFFC7,  // GDK_KEY_F10
    "F11": 0xFFC8,  // GDK_KEY_F11
    "F12": 0xFFC9  // GDK_KEY_F12
  };

  const code_map = {
    "Numpad0": 0xFFB0,  // GDK_KEY_KP_0
    "Numpad1": 0xFFB1,
    "Numpad2": 0xFFB2,
    "Numpad3": 0xFFB3,
    "Numpad4": 0xFFB4,
    "Numpad5": 0xFFB5,
    "Numpad6": 0xFFB6,
    "Numpad7": 0xFFB7,
    "Numpad8": 0xFFB8,
    "Numpad9": 0xFFB9  // GDK_KEY_KP_9
  };


  var normalize_key = function(keyboard_event) {
    var result = {
      name: "",
      code: "",
      numerical_code: 0,
      ctrl: false,
      alt: false,
      shift: false
    };

    if(keyboard_event.ctrlKey) {
      result.ctrl = keyboard_event.ctrlKey;
    }

    if(keyboard_event.altKey) {
      result.alt = keyboard_event.altKey;
    }

    if(keyboard_event.shiftKey) {
      result.shift = keyboard_event.shiftKey;
    }

    if(keyboard_event.key && keyboard_event.code) {
      result.name = keyboard_event.key;
      result.code = keyboard_event.code;

      if(result.name.length === 1) {
        // Use the ASCII code of the printable character:
        result.numerical_code = result.name.charCodeAt(0);
      } else if(key_map.hasOwnProperty(result.name)) {
        // Look it up in the table
        result.numerical_code = key_map[result.name];
      }

      // We must handle numpad explicitly because its
      // `keyboard_event.name` value is identical to the number on the
      // number key row.
      if(keyboard_event.code.startsWith("Numpad")) {
        result.numerical_code = code_map[keyboard_event.code];
      }
    }

    return result;
  };

  return normalize_key;

}());
