#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]
fn main() -> std::io::Result<()> {
    // for i in 0x2500..=0x257F {
    //     let t = i - 0x2500;
    //     eprintln!("{t: >3}: {t:016b} {}", char::from_u32(i as u32).unwrap());
    // }

    for t in horizontal() {
        eprintln!("{t:016b}");
    }

    Ok(())
}

const fn horizontal() -> [usize; 9] {
    [
        make_index('─'),
        make_index('━'),
        make_index('┄'),
        make_index('┅'),
        make_index('┈'),
        make_index('┉'),
        make_index('╌'),
        make_index('╍'),
        make_index('═'),
    ]
}

const fn vertical() -> [usize; 9] {
    [
        make_index('│'),
        make_index('┃'),
        make_index('┆'),
        make_index('┇'),
        make_index('┊'),
        make_index('┋'),
        make_index('╎'),
        make_index('╏'),
        make_index('║'),
    ]
}

const fn make_index(ch: char) -> usize {
    let ch = ch as u32;
    assert!(ch >= 0x2500 && ch <= 0x257F);
    (ch - 0x2500) as usize
}

struct Line;
impl Line {
    const NONE: u8 = 0;
    const THIN_SOLID: u8 = 0b0001;
    const HEAVY_SOLID: u8 = 0b0010;
    const DOUBLE: u8 = 0b0011;
    const THIN_DOUBLE_DASH: u8 = 0b0100;
    const HEAVY_DOUBLE_DASH: u8 = 0b0101;
    const THIN_TRIPLE_DASH: u8 = 0b0110;
    const HEAVY_TRIPLE_DASH: u8 = 0b0111;
    const THIN_QUADRUPLE_DASH: u8 = 0b1000;
    const HEAVY_QUADRUPLE_DASH: u8 = 0b1001;

    const fn horizontal() {
        todo!();
    }
    const fn vertical() {
        todo!();
    }
}

const BITSET: [u16; 128] = [0; 128];

const fn something(ch: char) -> (usize,) {
    let index = make_index(ch);
    let mask = make_direction_mask(Line::THIN_SOLID, Line::NONE, Line::THIN_SOLID, Line::NONE);

    todo!();
}

const fn make_direction_mask(left: u8, up: u8, right: u8, down: u8) -> u16 {
    ((right as u16) << 0) | ((up as u16) << 4) | ((left as u16) << 8) | ((down as u16) << 12)
}

// fn set(set:&mut [u16;128], ch:char, )

const fn char_index(ch: char) -> Option<usize> {
    let ch = ch as u32;
    if ch >= 0x2500 && ch <= 0x257F {
        Some((ch - 0x2500) as usize)
    } else {
        return None;
    }
}

// 0: 0000000000000000 ─
// 1: 0000000000000001 ━
// 4: 0000000000000100 ┄
// 5: 0000000000000101 ┅
// 8: 0000000000001000 ┈
// 9: 0000000000001001 ┉
// 76: 0000000001001100 ╌
// 77: 0000000001001101 ╍
// 80: 0000000001010000 ═

// 2: 0000000000000010 │
// 3: 0000000000000011 ┃
// 6: 0000000000000110 ┆
// 7: 0000000000000111 ┇
// 10: 0000000000001010 ┊
// 11: 0000000000001011 ┋
// 78: 0000000001001110 ╎
// 79: 0000000001001111 ╏
// 81: 0000000001010001 ║

// horizontal = vertical - 2
// vertical = horizontal + 2
