// Ported from Loretta.CodeAnalysis.Lua.Utilities.HexFloat (b767b4e): HexFloat
// C# source: src/Compilers/Lua/Portable/Utilities/HexFloat.cs
// Original: Copyright (c) Stephan Tolksdorf 2008-2013, Simplified BSD License.

/// Hexadecimal floating-point conversion utilities.
pub struct HexFloat;

impl HexFloat {
    #[rustfmt::skip]
    const ASCII_HEX_VALUE_PLUS_1S: [u8; 128] = [
        0,  0,  0,  0,  0,  0,  0,  0, 0,  0, 0, 0, 0, 0, 0, 0,
        0,  0,  0,  0,  0,  0,  0,  0, 0,  0, 0, 0, 0, 0, 0, 0,
        0,  0,  0,  0,  0,  0,  0,  0, 0,  0, 0, 0, 0, 0, 0, 0,
        1,  2,  3,  4,  5,  6,  7,  8, 9, 10, 0, 0, 0, 0, 0, 0,
        0, 11, 12, 13, 14, 15, 16,  0, 0,  0, 0, 0, 0, 0, 0, 0,
        0,  0,  0,  0,  0,  0,  0,  0, 0,  0, 0, 0, 0, 0, 0, 0,
        0, 11, 12, 13, 14, 15, 16,  0, 0,  0, 0, 0, 0, 0, 0, 0,
        0,  0,  0,  0,  0,  0,  0,  0, 0,  0, 0, 0, 0, 0, 0, 0,
    ];

    /// Converts a double to its hexadecimal string representation.
    pub fn double_to_hex_string(x: f64) -> String {
        const EXP_BITS: i32 = 11;
        const MAX_BITS: i32 = 53;
        const MAX_CHARS: usize = 24;
        const MAX_BIASED_EXP: i32 = (1 << EXP_BITS) - 1;
        const MAX_EXP: i32 = 1 << (EXP_BITS - 1);
        const BIAS: i32 = MAX_EXP - 1;
        const MAX_FRACT_NIBBLES: usize = ((MAX_BITS - 1 + 3) / 4) as usize;
        const MASK: u64 = (1u64 << (MAX_BITS - 1)) - 1;

        let xn = x.to_bits();
        let sign = (xn >> (MAX_BITS - 1 + EXP_BITS)) as i32;
        let e = ((xn >> (MAX_BITS - 1)) & MAX_BIASED_EXP as u64) as i32;
        let s = xn & MASK;

        if e < MAX_BIASED_EXP {
            if e == 0 && s == 0 {
                return if sign == 0 {
                    "0x0.0p0".to_string()
                } else {
                    "-0x0.0p0".to_string()
                };
            }

            let mut result = Vec::with_capacity(MAX_CHARS);
            if sign != 0 {
                result.push('-');
            }
            result.push('0');
            result.push('x');
            result.push(if e > 0 { '1' } else { '0' });
            result.push('.');

            let mut last_non_null = result.len();
            for j in 0..MAX_FRACT_NIBBLES {
                let h = ((s >> ((MAX_FRACT_NIBBLES - 1 - j) << 2)) & 0xf) as usize;
                if h != 0 {
                    last_non_null = result.len();
                }
                result.push("0123456789abcdef".as_bytes()[h] as char);
            }
            result.truncate(last_non_null + 1);
            result.push('p');

            let mut abs_exp = e;
            if abs_exp >= BIAS {
                abs_exp -= BIAS;
            } else {
                result.push('-');
                abs_exp = if abs_exp > 0 {
                    -(abs_exp - BIAS)
                } else {
                    BIAS - 1
                };
            }

            let li = if abs_exp < 10 {
                1
            } else if abs_exp < 100 {
                2
            } else if abs_exp < 1000 {
                3
            } else {
                4
            };
            let start = result.len();
            result.resize(start + li, '\0');
            let mut e_tmp = abs_exp;
            let mut idx = start + li;
            loop {
                let r = e_tmp % 10;
                e_tmp /= 10;
                idx -= 1;
                result[idx] = (48 + r) as u8 as char;
                if e_tmp == 0 {
                    break;
                }
            }

            result.into_iter().collect()
        } else if s == 0 {
            if sign == 0 {
                "Infinity".to_string()
            } else {
                "-Infinity".to_string()
            }
        } else {
            "NaN".to_string()
        }
    }

    /// Parses a hexadecimal string representation of a double.
    pub fn double_from_hex_string(s: &str) -> Result<f64, HexFloatError> {
        const EXP_BITS: i32 = 11;
        const MAX_BITS: i32 = 53;
        const MAX_EXP: i32 = 1 << (EXP_BITS - 1);
        const MIN_EXP: i32 = -MAX_EXP + 3;
        const MIN_S_EXP: i32 = MIN_EXP - (MAX_BITS - 1);
        const MAX_BITS2: i32 = MAX_BITS + 2;
        const MASK: u64 = (1u64 << (MAX_BITS - 1)) - 1;

        let n = s.len();
        if n == 0 {
            return Err(HexFloatError::InvalidFormat);
        }

        if n > ((i32::MAX as i64 + MIN_S_EXP as i64 - 10) / 4) as usize {
            return Err(HexFloatError::Overflow);
        }

        let bytes = s.as_bytes();
        let mut sign: i32 = 0;
        let mut xn: u64 = 0;
        let mut n_bits: i32 = -1;
        let mut exp: i32 = 0;
        let mut i = 0;

        // sign
        if bytes[i] == b'+' {
            i = 1;
        } else if bytes[i] == b'-' {
            i = 1;
            sign = 1;
        }

        // "0x" prefix
        if i + 1 < n && (bytes[i + 1] == b'x' || bytes[i + 1] == b'X') {
            if bytes[i] != b'0' {
                return Err(HexFloatError::InvalidFormat);
            }
            i += 2;
        }

        let mut past_dot = false;
        loop {
            if i == n {
                if !past_dot {
                    exp = n_bits;
                }
                if n_bits >= 0 {
                    break;
                } else {
                    return Err(HexFloatError::InvalidFormat);
                }
            }
            let c = bytes[i];
            i += 1;

            if (c as usize) < 128 {
                let h = Self::ASCII_HEX_VALUE_PLUS_1S[c as usize];
                if h != 0 {
                    let h = h as i32 - 1;
                    if n_bits <= 0 {
                        xn |= h as u64;
                        n_bits = 0;
                        let mut h_tmp = h;
                        while h_tmp > 0 {
                            n_bits += 1;
                            h_tmp >>= 1;
                        }
                        if past_dot {
                            exp -= 4 - n_bits;
                        }
                    } else if n_bits <= MAX_BITS2 - 4 {
                        xn <<= 4;
                        xn |= h as u64;
                        n_bits += 4;
                    } else if n_bits < MAX_BITS2 {
                        let n_rem_bits = MAX_BITS2 - n_bits;
                        let n_surplus_bits = 4 - n_rem_bits;
                        let surplus_bits = h & (0xf >> n_rem_bits);
                        let surplus_bits = (0xfffe >> surplus_bits) & 1;
                        xn <<= n_rem_bits;
                        xn |= ((h >> n_surplus_bits) | surplus_bits) as u64;
                        n_bits += 4;
                    } else {
                        xn |= ((0xfffe >> h) & 1) as u64;
                        n_bits += 4;
                    }
                    continue;
                }
            }

            if c == b'.' {
                if past_dot {
                    return Err(HexFloatError::InvalidFormat);
                }
                past_dot = true;
                exp = if n_bits >= 0 { n_bits } else { 0 };
                continue;
            }

            if (c | 0x20) == b'p' && n_bits >= 0 {
                if !past_dot {
                    exp = n_bits;
                }
                let mut e_sign = 1i32;
                if i < n && (bytes[i] == b'-' || bytes[i] == b'+') {
                    if bytes[i] == b'-' {
                        e_sign = -1;
                    }
                    i += 1;
                }
                if i == n {
                    return Err(HexFloatError::InvalidFormat);
                }
                let mut e: i32 = 0;
                while i < n {
                    let c = bytes[i];
                    i += 1;
                    if (c.wrapping_sub(b'0')) <= 9 {
                        if e <= (i32::MAX - 9) / 10 {
                            e = e * 10 + (c - b'0') as i32;
                        } else {
                            e = i32::MAX - 8;
                        }
                    } else {
                        return Err(HexFloatError::InvalidFormat);
                    }
                }
                e *= e_sign;
                exp = exp
                    .checked_add(e)
                    .unwrap_or(if e < 0 { i32::MIN } else { i32::MAX });
                break;
            }

            // Check for Infinity or NaN
            i -= 1;
            if n_bits == -1 && i + 3 <= n {
                if ((bytes[i] | 0x20) == b'i')
                    && ((bytes[i + 1] | 0x20) == b'n')
                    && ((bytes[i + 2] | 0x20) == b'f')
                    && (i + 3 == n
                        || (i + 8 == n
                            && ((bytes[i + 3] | 0x20) == b'i')
                            && ((bytes[i + 4] | 0x20) == b'n')
                            && ((bytes[i + 5] | 0x20) == b'i')
                            && ((bytes[i + 6] | 0x20) == b't')
                            && ((bytes[i + 7] | 0x20) == b'y')))
                {
                    return Ok(if sign == 0 {
                        f64::INFINITY
                    } else {
                        f64::NEG_INFINITY
                    });
                } else if i + 3 == n
                    && ((bytes[i] | 0x20) == b'n')
                    && ((bytes[i + 1] | 0x20) == b'a')
                    && ((bytes[i + 2] | 0x20) == b'n')
                {
                    return Ok(f64::NAN);
                }
            }
            return Err(HexFloatError::InvalidFormat);
        }

        if n_bits == 0 {
            return Ok(if sign == 0 { 0.0 } else { -0.0 });
        }

        if exp <= MAX_EXP {
            if exp >= MIN_EXP && n_bits <= MAX_BITS {
                if n_bits < MAX_BITS {
                    xn <<= (MAX_BITS - n_bits) as u64;
                }
                xn &= MASK;
            } else {
                if n_bits < MAX_BITS2 {
                    xn <<= (MAX_BITS2 - n_bits) as u64;
                }
                let mut is_subnormal = 0i32;
                if exp < MIN_EXP {
                    if exp < MIN_S_EXP - 1 {
                        return Ok(if sign == 0 { 0.0 } else { -0.0 });
                    }
                    is_subnormal = 1;
                    loop {
                        xn = (xn >> 1) | (xn & 1);
                        exp += 1;
                        if exp >= MIN_EXP {
                            break;
                        }
                    }
                    if xn <= 2 {
                        return Ok(if sign == 0 { 0.0 } else { -0.0 });
                    }
                }
                let r = (xn as i32) & 0x7;
                xn >>= 2;
                if r >= 6 || r == 3 {
                    xn = xn.wrapping_add(1);
                    xn &= MASK;
                    if xn == 0 {
                        exp += 1;
                        if exp > MAX_EXP {
                            return Err(HexFloatError::Overflow);
                        }
                    }
                } else {
                    xn &= MASK;
                }
                exp -= is_subnormal;
            }
            exp -= MIN_EXP - 1;
            xn |= ((sign as u64) << (MAX_BITS - 1 + EXP_BITS)) | ((exp as u64) << (MAX_BITS - 1));
            Ok(f64::from_bits(xn))
        } else {
            Err(HexFloatError::Overflow)
        }
    }
}

/// Errors from hex float parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexFloatError {
    /// The string is not a valid hex float format.
    InvalidFormat,
    /// The value is too large or too small for a double.
    Overflow,
}

impl std::fmt::Display for HexFloatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HexFloatError::InvalidFormat => {
                write!(f, "Invalid hexadecimal floating-point format")
            }
            HexFloatError::Overflow => {
                write!(f, "Floating-point value overflow or underflow")
            }
        }
    }
}

impl std::error::Error for HexFloatError {}
