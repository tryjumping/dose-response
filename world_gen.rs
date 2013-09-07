pub fn default(w: uint, h: uint) -> ~[(uint, uint, char)] {
    let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut result: ~[(uint, uint, char)] = ~[];
    for std::uint::range(0, w) |x| {
        for std::uint::range(0, h) |y| {
            result.push((x, y, chars[(x * y) % chars.char_len()] as char));
        }
    }
    return result;
}
