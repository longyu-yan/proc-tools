use crate::float2str::common::{log10_pow2, log10_pow5, pow5bits};
use crate::float2str::d2s;
use crate::float2str::{DOUBLE_POW5_INV_SPLIT, DOUBLE_POW5_SPLIT};

pub(crate) const FLOAT_MANTISSA_BITS: u32 = 23;
pub(crate) const FLOAT_EXPONENT_BITS: u32 = 8;
const FLOAT_BIAS: i32 = 127;

const FLOAT_POW5_INV_BITCOUNT: i32 = d2s::DOUBLE_POW5_INV_BITCOUNT - 64;
const FLOAT_POW5_BITCOUNT: i32 = d2s::DOUBLE_POW5_BITCOUNT - 64;

pub(crate) struct FloatingDecimal32 {
    pub mantissa: u32,
    pub exponent: i32,
}

#[inline(always)]
pub(crate) fn f2d(ieee_mantissa: u32, ieee_exponent: u32) -> FloatingDecimal32 {
    let (e2, m2) = if ieee_exponent == 0 {
        (
            // 减去2，边界计算就有2个额外的比特
            1 - FLOAT_BIAS - FLOAT_MANTISSA_BITS as i32 - 2,
            ieee_mantissa,
        )
    } else {
        (
            ieee_exponent as i32 - FLOAT_BIAS - FLOAT_MANTISSA_BITS as i32 - 2,
            (1u32 << FLOAT_MANTISSA_BITS) | ieee_mantissa,
        )
    };
    let even = (m2 & 1) == 0;
    let accept_bounds = even;

    let mv = 4 * m2;
    let mp = 4 * m2 + 2;
    let mm_shift = (ieee_mantissa != 0 || ieee_exponent <= 1) as u32;
    let mm = 4 * m2 - 1 - mm_shift;

    let mut vr: u32;
    let mut vp: u32;
    let mut vm: u32;
    let e10: i32;
    let mut vm_is_trailing_zeros = false;
    let mut vr_is_trailing_zeros = false;
    let mut last_removed_digit = 0u8;
    if e2 >= 0 {
        let q = log10_pow2(e2);
        e10 = q as i32;
        let k = FLOAT_POW5_INV_BITCOUNT + pow5bits(q as i32) - 1;
        let i = -e2 + q as i32 + k;
        vr = mul_pow5_inv_div_pow2(mv, q, i);
        vp = mul_pow5_inv_div_pow2(mp, q, i);
        vm = mul_pow5_inv_div_pow2(mm, q, i);
        if q != 0 && (vp - 1) / 10 <= vm / 10 {
            let l = FLOAT_POW5_INV_BITCOUNT + pow5bits(q as i32 - 1) - 1;
            last_removed_digit = (mul_pow5_inv_div_pow2(mv, q - 1, -e2 + q as i32 - 1 + l) % 10) as u8;
        }
        if q <= 9 {
            if mv % 5 == 0 {
                vr_is_trailing_zeros = multiple_of_power_of_5_32(mv, q);
            } else if accept_bounds {
                vm_is_trailing_zeros = multiple_of_power_of_5_32(mm, q);
            } else {
                vp -= multiple_of_power_of_5_32(mp, q) as u32;
            }
        }
    } else {
        let q = log10_pow5(-e2);
        e10 = q as i32 + e2;
        let i = -e2 - q as i32;
        let k = pow5bits(i) - FLOAT_POW5_BITCOUNT;
        let mut j = q as i32 - k;
        vr = mul_pow5_div_pow2(mv, i as u32, j);
        vp = mul_pow5_div_pow2(mp, i as u32, j);
        vm = mul_pow5_div_pow2(mm, i as u32, j);
        if q != 0 && (vp - 1) / 10 <= vm / 10 {
            j = q as i32 - 1 - (pow5bits(i + 1) - FLOAT_POW5_BITCOUNT);
            last_removed_digit = (mul_pow5_div_pow2(mv, (i + 1) as u32, j) % 10) as u8;
        }
        if q <= 1 {
            vr_is_trailing_zeros = true;
            if accept_bounds {
                // mm = mv - 1 - mm_shift, so it has 1 trailing 0 bit iff mm_shift == 1.
                vm_is_trailing_zeros = mm_shift == 1;
            } else {
                vp -= 1;
            }
        } else if q < 31 {
            vr_is_trailing_zeros = (mv & ((1u32 << (q - 1)) - 1)) == 0;
        }
    }

    let mut removed = 0i32;
    let output = if vm_is_trailing_zeros || vr_is_trailing_zeros {
        while vp / 10 > vm / 10 {
            vm_is_trailing_zeros &= vm - (vm / 10) * 10 == 0;
            vr_is_trailing_zeros &= last_removed_digit == 0;
            last_removed_digit = (vr % 10) as u8;
            vr /= 10;
            vp /= 10;
            vm /= 10;
            removed += 1;
        }
        if vm_is_trailing_zeros {
            while vm % 10 == 0 {
                vr_is_trailing_zeros &= last_removed_digit == 0;
                last_removed_digit = (vr % 10) as u8;
                vr /= 10;
                vp /= 10;
                vm /= 10;
                removed += 1;
            }
        }
        if vr_is_trailing_zeros && last_removed_digit == 5 && vr % 2 == 0 {
            last_removed_digit = 4;
        }
        vr + ((vr == vm && (!accept_bounds || !vm_is_trailing_zeros)) || last_removed_digit >= 5) as u32
    } else {
        while vp / 10 > vm / 10 {
            last_removed_digit = (vr % 10) as u8;
            vr /= 10;
            vp /= 10;
            vm /= 10;
            removed += 1;
        }
        vr + (vr == vm || last_removed_digit >= 5) as u32
    };
    let exp = e10 + removed;

    FloatingDecimal32 {
        exponent: exp,
        mantissa: output,
    }
}

#[inline(always)]
pub(crate) fn mul_pow5_div_pow2(m: u32, i: u32, j: i32) -> u32 {
    let factor = unsafe { DOUBLE_POW5_SPLIT.get_unchecked(i as usize).1 };
    let factor_lo = factor as u32;
    let factor_hi = (factor >> 32) as u32;
    let bits0 = m as u64 * factor_lo as u64;
    let bits1 = m as u64 * factor_hi as u64;

    let sum = (bits0 >> 32) + bits1;
    let shifted_sum = sum >> (j - 32);
    shifted_sum as u32
}

#[inline(always)]
pub(crate) fn multiple_of_power_of_5_32(mut value: u32, p: u32) -> bool {
    let mut count = 0u32;
    loop {
        let q = value / 5;
        let r = value % 5;
        if r != 0 {
            break;
        }
        value = q;
        count += 1;
    }
    count >= p
}

#[inline(always)]
pub(crate) fn mul_pow5_inv_div_pow2(m: u32, q: u32, j: i32) -> u32 {
    let factor = unsafe { DOUBLE_POW5_INV_SPLIT.get_unchecked(q as usize).1 + 1 };
    let factor_lo = factor as u32;
    let factor_hi = (factor >> 32) as u32;
    let bits0 = m as u64 * factor_lo as u64;
    let bits1 = m as u64 * factor_hi as u64;

    let sum = (bits0 >> 32) + bits1;
    let shifted_sum = sum >> (j - 32);
    shifted_sum as u32
}
