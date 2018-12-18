use super::*;

#[derive(Eq, PartialEq, PartialOrd)]
enum Token {
  Hex,
  HexDigit(char),
  Digit(char),
  Dot,
  End,
  Plus,
  Minus,
  Mul,
  Div,
  Mod,
  ParOpen,
  ParClose,
  Byte(usize),
  PByte(usize),
  BitPos,
  Value,
  Not,
  And,
  Or,
  Xor,
  Shl,
  Shr,
}

use self::Token::*;

use std::slice;
use itertools::{self, PeekingNext, structs::PutBackN};

fn put_back(it: &mut PutBackN<impl Iterator<Item = char>>, string: &str) {
  for c in string.chars() {
    it.put_back(c);
  }
}

#[no_mangle]
pub unsafe extern fn execIExpression(str: *mut *mut c_char, bInPtr: *mut c_uchar, bitpos: c_char, pPtr: *mut c_char, err: *mut c_char) -> c_int {
  let input_string = CStr::from_ptr(*str).to_str().unwrap();
  let mut it = itertools::put_back_n(input_string.chars());

  let mut b_ptr: [u8; 10] = Default::default();
  let slice = slice::from_raw_parts(bInPtr, mem::size_of_val(&b_ptr));
  b_ptr.copy_from_slice(slice);

  execIExpression2(&mut it, &b_ptr, bitpos, pPtr).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern fn execExpression(str: *mut *mut c_char, bInPtr: *mut c_uchar, floatV: c_float, err: *mut c_char) -> c_float {
  let input_string = CStr::from_ptr(*str).to_str().unwrap();
  let mut it = itertools::put_back_n(input_string.chars());

  let mut b_ptr: [u8; 10] = Default::default();
  let slice = slice::from_raw_parts(bInPtr, mem::size_of_val(&b_ptr));
  b_ptr.copy_from_slice(slice);

  execExpression2(&mut it, &b_ptr, floatV).unwrap_or(0.0)
}

unsafe fn execIExpression2(mut it: &mut PutBackN<impl Iterator<Item = char>>, b_ptr: &[u8; 10], bitpos: c_char, pPtr: *mut c_char) -> Result<c_int, String> {
  let mut term1: c_int = match nextToken(&mut it)? {
    (Plus, _) => execITerm(&mut it, &b_ptr, bitpos, pPtr)?,
    (Minus, _) => -execITerm(&mut it, &b_ptr, bitpos, pPtr)?,
    (Not, _) => !execITerm(&mut it, &b_ptr, bitpos, pPtr)?,
    (_, cs) => {
      put_back(&mut it, &cs);
      execITerm(&mut it, &b_ptr, bitpos, pPtr)?
    },
  };

  loop {
    let term2: c_int = match nextToken(&mut it)? {
      (Plus, _) => execITerm(&mut it, &b_ptr, bitpos, pPtr)?,
      (Minus, _) => -execITerm(&mut it, &b_ptr, bitpos, pPtr)?,
      (Not, _) => !execITerm(&mut it, &b_ptr, bitpos, pPtr)?,
      (_, cs) => {
        put_back(&mut it, &cs);
        return Ok(term1)
      }
    };

    term1 += term2;
  }
}

unsafe fn execExpression2(mut it: &mut PutBackN<impl Iterator<Item = char>>, b_ptr: &[u8; 10], floatV: c_float) -> Result<c_float, String> {
  let f: c_float = match nextToken(&mut it)? {
    (Plus, _) => 1.0,
    (Minus, _) => -1.0,
    (_, cs) => {
      put_back(&mut it, &cs);
      1.0
    },
  };

  let mut term1 = execTerm(&mut it, &b_ptr, floatV)? * f;

  // println!("T1={}", term1);

  loop {
    let f: c_float = match nextToken(&mut it)? {
      (Plus, _) => 1.0,
      (Minus, _) => -1.0,
      (_, cs) => {
        put_back(&mut it, &cs);
        // println!("Exp={}", term1);
        return Ok(term1)
      }
    };

    term1 += execTerm(&mut it, &b_ptr, floatV)? * f;
  }
}

unsafe fn execITerm(mut it: &mut PutBackN<impl Iterator<Item = char>>, b_ptr: &[u8; 10], bitpos: c_char, pPtr: *mut c_char) -> Result<c_int, String> {
  let mut factor1 = execIFactor(&mut it, b_ptr, bitpos, pPtr)?;

  loop {
    let op = match nextToken(&mut it)? {
      (op @ Mul, _) | (op @ Div, _) | (op @ Mod, _) | (op @ And, _) | (op @ Or, _) | (op @ Xor, _) | (op @ Shl, _) | (op @ Shr, _) => op,
      (_, cs) => {
        put_back(&mut it, &cs);
        // println!("ret({})", factor1);
        return Ok(factor1)
      },
    };

    let factor2 = execIFactor(&mut it, b_ptr, bitpos, pPtr)?;

    match op {
      Mul => factor1 *= factor2,
      Div => factor1 /= factor2,
      Mod => factor1 %= factor2,
      And => factor1 &= factor2,
      Or => factor1 |= factor2,
      Xor => factor1 ^= factor2,
      Shl => factor1 <<= factor2,
      Shr => factor1 >>= factor2,
      _ => unreachable!(),
    }
  }
}

unsafe fn execTerm(mut it: &mut PutBackN<impl Iterator<Item = char>>, b_ptr: &[u8; 10], floatV: c_float) -> Result<c_float, String> {
  // println!("execTerm: {}", CStr::from_ptr(*str).to_str().unwrap());

  let mut factor1: c_float = execFactor(&mut it, b_ptr, floatV)?;

  // println!("F1={}", factor1);

  loop {
    let op = match nextToken(&mut it)? {
      (op @ Mul, _) | (op @ Div, _) => op,
      (_, cs) => {
        put_back(&mut it, &cs);
        // println!("ret({})", factor1);
        return Ok(factor1)
      },
    };

    let factor2: c_float = execFactor(&mut it, b_ptr, floatV)?;

    // println!("F2={}", factor2);

    match op {
      Mul => factor1 *= factor2,
      Div => factor1 /= factor2,
      _ => unreachable!(),
    }
  }
}

unsafe fn nextToken(it: &mut PutBackN<impl Iterator<Item = char>>) -> Result<(Token, String), String> {
  let mut token_string = String::with_capacity(2);

  while let Some(_) = it.peeking_next(|c| c.is_whitespace()) {}

  let token = if let Some(c) = it.next() {
    token_string.push(c);

    match c {
      '+' => Plus,
      '-' => Minus,
      '*' => Mul,
      '/' => Div,
      '%' => Mod,
      '(' => ParOpen,
      ')' => ParClose,
      'V' => Value,
      '^' => Xor,
      '&' => And,
      '|' => Or,
      '~' => Not,
      '0' => match it.peeking_next(|c| *c == 'x') {
        Some(c) => {
          token_string.push(c);
          Hex
        },
        _ => Digit('0'),
      },
      '<' => {
        match it.peeking_next(|c| *c == '<') {
          Some(c) => {
            token_string.push(c);
            Shl
          },
          _ => return Err(String::from("unexpected character")),
        }
      },
      '>' => {
        match it.peeking_next(|c| *c == '>') {
          Some(c) => {
            token_string.push(c);
            Shr
          },
          _ => return Err(String::from("unexpected character")),
        }
      },
      'B' => {
        match it.next() {
          Some(c @ '0'..='9') => {
            token_string.push(c);
            Byte(u32::from(c) as usize)
          },
          Some(c @ 'P') => {
            token_string.push(c);
            BitPos
          },
          Some(c) => {
            it.put_back(c);
            return Err(format!("unexpected character: '{}'; expected '0'...'9' or 'P'", c))
          },
          None => return Err(String::from("unexpected EOF")),
        }
      },
      'P' => {
        match it.next() {
          Some(c @ '0'..='9') => {
            token_string.push(c);
            PByte(u32::from(c) as usize)
          },
          Some(c) => {
            it.put_back(c);
            return Err(format!("unexpected character: '{}'; expected '0'...'9'", c))
          },
          _ => return Err(String::from("unexpected EOF")),
        }
      },
      c @ '1'..='9' => Digit(c),
      c @ 'a'..='f' => HexDigit(c),
      '.' => Dot,
      c => {
        it.put_back(c);
        return Err(format!("unexpected character: '{}'", c))
      },
    }
  } else {
    End
  };

  Ok((token, token_string))
}

unsafe fn execFactor(mut it: &mut PutBackN<impl Iterator<Item = char>>, b_ptr: &[u8; 10], floatV: c_float) -> Result<c_float, String> {
  match nextToken(&mut it)? {
    (Byte(i), _) => Ok(b_ptr[i] as c_float),
    (Value, _) => Ok(floatV),
    (Hex, mut hex) => {
      loop {
        match nextToken(&mut it)? {
          (Digit(c), _) | (HexDigit(c), _) => hex.push(c),
          (_, cs) => {
            put_back(&mut it, &cs);
            break
          },
        }
      }

      let without_prefix = hex.trim_start_matches("0x");
      Ok(i32::from_str_radix(without_prefix, 16).unwrap_or(0) as c_float)
    },
    (Digit(c), _) => {
      let mut dec = c.to_string();

      loop {
        match nextToken(&mut it)? {
          (Digit(c), _) => dec.push(c),
          (Dot, _) => {
            dec.push('.');

            loop {
              match nextToken(&mut it)? {
                (Digit(c), _) => dec.push(c),
                (_, cs) => {
                  put_back(&mut it, &cs);
                  break
                },
              }
            }

            break
          }
          (_, cs) => {
            put_back(&mut it, &cs);
            break
          },
        }
      }

      Ok(dec.parse().unwrap_or(0.0))
    },
    (ParOpen, _) => {
      let expression = execExpression2(&mut it, b_ptr, floatV)?;

      match nextToken(&mut it)? {
        (ParClose, _) => Ok(expression),
        _ => return Err("expected ')'".into())
      }
    },
    _ => {
      return Err("expected factor: B0..B9 BP number ( )".into())
    },
  }
}

unsafe fn execIFactor(mut it: &mut PutBackN<impl Iterator<Item = char>>, b_ptr: &[u8; 10], bitpos: c_char, pPtr: *mut c_char) -> Result<c_int, String> {
  match nextToken(&mut it)? {
    (Byte(i), _) => Ok(b_ptr[i] as c_int & 0xff),
    (BitPos, _) => Ok(bitpos as c_int & 0xff),
    (PByte(i), _) => Ok(*pPtr.add(i) as c_int & 0xff),
    (Hex, mut hex) => {
      loop {
        match nextToken(&mut it)? {
          (Digit(c), _) | (HexDigit(c), _) => hex.push(c),
          (_, cs) => {
            put_back(&mut it, &cs);
            break
          },
        }
      }

      let without_prefix = hex.trim_start_matches("0x");
      Ok(c_int::from_str_radix(without_prefix, 16).unwrap_or(0))
    },
    (Digit(c), _) => {
      let mut dec = c.to_string();

      loop {
        match nextToken(&mut it)? {
          (Digit(c), _) => dec.push(c),
          (Dot, _) => {
            dec.push('.');

            loop {
              match nextToken(&mut it)? {
                (Digit(c), _) => dec.push(c),
                (_, cs) => {
                  put_back(&mut it, &cs);
                  break
                },
              }
            }

            break
          }
          (_, cs) => {
            put_back(&mut it, &cs);
            break
          },
        }
      }

      Ok(dec.parse().unwrap_or(0))
    },
    (ParOpen, _) => {
      let expression = execIExpression2(&mut it, b_ptr, bitpos, pPtr)?;

      match nextToken(&mut it)? {
        (ParClose, _) => Ok(expression),
        _ => return Err("expected ')'".into())
      }
    },
    (Not, _) => Ok(!execIFactor(&mut it, b_ptr, bitpos, pPtr)?),
    _ => {
      return Err("expected factor: B0..B9 P0..P9 BP number ( )".into())
    },
  }
}
