#[inline(always)]
pub(crate) fn decimal_length9(v: u32) -> isize {
    if v < 10 {
        1
    } else if v < 100 {
        2
    } else if v < 1000 {
        3
    } else if v < 10000 {
        4
    } else if v < 100000 {
        5
    } else if v < 1000000 {
        6
    } else if v < 10000000 {
        7
    } else if v < 100000000 {
        8
    } else {
        9
    }
}

#[inline(always)]
pub(crate) fn pow5bits(e: i32) -> i32 {
    (((e as u32 * 1217359) >> 19) + 1) as i32
}

#[inline(always)]
pub(crate) fn log10_pow2(e: i32) -> u32 {
    (e as u32 * 78913) >> 18
}

#[inline(always)]
pub(crate) fn log10_pow5(e: i32) -> u32 {
    (e as u32 * 732923) >> 20
}
