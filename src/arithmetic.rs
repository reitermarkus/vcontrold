use super::*;

#[derive(Eq, PartialEq, PartialOrd)]
enum Token {
  Hex,
  HexDigit(char),
  Digit(char),
  Dot,
  End,
  Error,
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

#[no_mangle]
pub unsafe extern fn execIExpression(str: *mut *mut c_char, bInPtr: *mut c_uchar, bitpos: c_char, pPtr: *mut c_char, err: *mut c_char) -> c_int {
  let input_string = CStr::from_ptr(*str).to_str().unwrap();
  let mut it = itertools::put_back_n(input_string.chars());

  let mut b_ptr: [u8; 10] = Default::default();
  let slice = slice::from_raw_parts(bInPtr, mem::size_of_val(&b_ptr));
  b_ptr.copy_from_slice(slice);

  execIExpression2(&mut it, &b_ptr, bitpos, pPtr, err)
}


#[no_mangle]
pub unsafe extern fn execExpression(str: *mut *mut c_char, bInPtr: *mut c_uchar, floatV: c_float, err: *mut c_char) -> c_float {
  let input_string = CStr::from_ptr(*str).to_str().unwrap();
  let mut it = itertools::put_back_n(input_string.chars());

  let mut b_ptr: [u8; 10] = Default::default();
  let slice = slice::from_raw_parts(bInPtr, mem::size_of_val(&b_ptr));
  b_ptr.copy_from_slice(slice);

  execExpression2(&mut it, &b_ptr, floatV, err)
}

unsafe fn execIExpression2(mut it: &mut PutBackN<impl Iterator<Item = char>>, b_ptr: &[u8; 10], bitpos: c_char, pPtr: *mut c_char, err: *mut c_char) -> c_int {
  let mut term1: c_int = match nextToken(&mut it) {
    (Plus, _) => execITerm(&mut it, &b_ptr, bitpos, pPtr, err),
    (Minus, _) => -execITerm(&mut it, &b_ptr, bitpos, pPtr, err),
    (Not, _) => !execITerm(&mut it, &b_ptr, bitpos, pPtr, err),
    (_, cs) => {
      for c in cs.chars() {
        it.put_back(c);
      }

      execITerm(&mut it, &b_ptr, bitpos, pPtr, err)
    },
  };

  if *err != 0 {
    return 0
  }

  loop {
    let term2: c_int = match nextToken(&mut it) {
      (Plus, _) => execITerm(&mut it, &b_ptr, bitpos, pPtr, err),
      (Minus, _) => -execITerm(&mut it, &b_ptr, bitpos, pPtr, err),
      (Not, _) => !execITerm(&mut it, &b_ptr, bitpos, pPtr, err),
      (_, cs) => {
        for c in cs.chars() {
          it.put_back(c);
        }

        return term1
      }
    };

    if *err != 0 {
      return 0
    }

    term1 += term2;
  }
}

unsafe fn execExpression2(mut it: &mut PutBackN<impl Iterator<Item = char>>, b_ptr: &[u8; 10], floatV: c_float, err: *mut c_char) -> c_float {
  let f: c_float = match nextToken(&mut it) {
    (Plus, _) => 1.0,
    (Minus, _) => -1.0,
    (_, cs) => {
      for c in cs.chars() {
        it.put_back(c);
      }

      1.0
    },
  };

  let mut term1 = execTerm(&mut it, &b_ptr, floatV, err) * f;

  if *err != 0 {
    return 0.0
  }

  // println!("T1={}", term1);

  loop {
    let f: c_float = match nextToken(&mut it) {
      (Plus, _) => 1.0,
      (Minus, _) => -1.0,
      (_, cs) => {
        for c in cs.chars() {
          it.put_back(c);
        }

        // println!("Exp={}", term1);

        return term1
      }
    };

    let term2 = execTerm(&mut it, &b_ptr, floatV, err);

    if *err != 0 {
      return 0.0
    }

    term1 += term2 * f;
  }
}

unsafe fn execITerm(mut it: &mut PutBackN<impl Iterator<Item = char>>, b_ptr: &[u8; 10], bitpos: c_char, pPtr: *mut c_char, err: *mut c_char) -> c_int {
  let mut factor1 = execIFactor(&mut it, b_ptr, bitpos, pPtr, err);

  if *err != 0 {
    return 0
  }

  loop {
    let op = match nextToken(&mut it) {
      (op @ Mul, _) | (op @ Div, _) | (op @ Mod, _) | (op @ And, _) | (op @ Or, _) | (op @ Xor, _) | (op @ Shl, _) | (op @ Shr, _) => op,
      (_, cs) => {
        for c in cs.chars() {
          it.put_back(c);
        }

        // println!("ret({})", factor1);

        return factor1
      },
    };

    let factor2 = execIFactor(&mut it, b_ptr, bitpos, pPtr, err);

    if *err != 0 {
      return 0
    }

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

unsafe fn execTerm(mut it: &mut PutBackN<impl Iterator<Item = char>>, b_ptr: &[u8; 10], floatV: c_float, err: *mut c_char) -> c_float {
  // println!("execTerm: {}", CStr::from_ptr(*str).to_str().unwrap());

  let mut factor1: c_float = execFactor(&mut it, b_ptr, floatV, err);

  if *err != 0 {
    return 0.0
  }

  // println!("F1={}", factor1);

  loop {
    let op = match nextToken(&mut it) {
      (op @ Mul, _) | (op @ Div, _) => op,
      (_, cs) => {
        for c in cs.chars() {
          it.put_back(c);
        }

        // println!("ret({})", factor1);

        return factor1
      },
    };

    let factor2: c_float = execFactor(&mut it, b_ptr, floatV, err);

    // println!("F2={}", factor2);

    if *err != 0 {
      return 0.0
    }

    match op {
      Mul => factor1 *= factor2,
      Div => factor1 /= factor2,
      _ => unreachable!(),
    }
  }
}

unsafe fn nextToken(it: &mut PutBackN<impl Iterator<Item = char>>) -> (Token, String) {
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
          _ => Error,
        }
      },
      '>' => {
        match it.peeking_next(|c| *c == '>') {
          Some(c) => {
            token_string.push(c);
            Shr
          },
          _ => Error,
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
            Error
          },
          None => Error,
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
            Error
          },
          _ => Error,
        }
      },
      c @ '1'..='9' => Digit(c),
      c @ 'a'..='f' => HexDigit(c),
      '.' => Dot,
      c => {
        it.put_back(c);
        Error
      },
    }
  } else {
    End
  };

  (token, token_string)
}

unsafe fn execFactor(mut it: &mut PutBackN<impl Iterator<Item = char>>, b_ptr: &[u8; 10], floatV: c_float, err: *mut c_char) -> c_float {
  match nextToken(&mut it) {
    (Byte(i), _) => b_ptr[i] as c_float,
    (Value, _) => floatV,
    (Hex, mut hex) => {
      loop {
        match nextToken(&mut it) {
          (Digit(c), _) | (HexDigit(c), _) => hex.push(c),
          (_, cs) => {
            for c in cs.chars() {
              it.put_back(c);
            }
            break
          },
        }
      }

      let without_prefix = hex.trim_start_matches("0x");
      i32::from_str_radix(without_prefix, 16).unwrap_or(0) as c_float
    },
    (Digit(c), _) => {
      let mut dec = c.to_string();

      loop {
        match nextToken(&mut it) {
          (Digit(c), _) => dec.push(c),
          (Dot, _) => {
            dec.push('.');

            loop {
              match nextToken(&mut it) {
                (Digit(c), _) => dec.push(c),
                (_, cs) => {
                  for c in cs.chars() {
                    it.put_back(c);
                  }
                  break
                },
              }
            }

            break
          }
          (_, cs) => {
            for c in cs.chars() {
              it.put_back(c);
            }
            break
          },
        }
      }

      dec.parse().unwrap_or(0.0)
    },
    (ParOpen, _) => {
      let expression = execExpression2(&mut it, b_ptr, floatV, err);

      if (*err) == 0 {
        return 0.0
      }

      if nextToken(&mut it).0 != ParClose {
        // sprintf(err, CString::new("expected factor:) [%c]\n").unwrap().as_ptr(), *item as c_int);
        return 0.0
      }

      expression
    },
    _ => {
      // sprintf(err, CString::new("expected factor: B0..B9 number ( ) [%c]\n").unwrap().as_ptr(), *item as c_int);
      return 0.0
    },
  }
}

unsafe fn execIFactor(mut it: &mut PutBackN<impl Iterator<Item = char>>, b_ptr: &[u8; 10], bitpos: c_char, pPtr: *mut c_char, err: *mut c_char) -> c_int {
  match nextToken(&mut it) {
    (Byte(i), _) => b_ptr[i] as c_int & 0xff,
    (BitPos, _) => bitpos as c_int & 0xff,
    (PByte(i), _) => *pPtr.add(i) as c_int & 0xff,
    (Hex, _) => {
      let mut hex = String::from("0x");

      loop {
        match nextToken(&mut it) {
          (Digit(c), _) | (HexDigit(c), _) => hex.push(c),
          (_, cs) => {
            for c in cs.chars() {
              it.put_back(c);
            }
            break
          },
        }
      }

      let without_prefix = hex.trim_start_matches("0x");
      c_int::from_str_radix(without_prefix, 16).unwrap_or(0)
    },
    (Digit(c), _) => {
      let mut dec = c.to_string();

      loop {
        match nextToken(&mut it) {
          (Digit(c), _) => dec.push(c),
          (Dot, _) => {
            dec.push('.');

            loop {
              match nextToken(&mut it) {
                (Digit(c), _) => dec.push(c),
                (_, cs) => {
                  for c in cs.chars() {
                    it.put_back(c);
                  }
                  break
                },
              }
            }

            break
          }
          (_, cs) => {
            for c in cs.chars() {
              it.put_back(c);
            }
            break
          },
        }
      }

      dec.parse().unwrap_or(0)
    },
    (ParOpen, _) => {
      let expression = execIExpression2(&mut it, b_ptr, bitpos, pPtr, err);

      if (*err) == 0 {
        return 0
      }

      if nextToken(&mut it).0 != ParClose {
        // sprintf(err, CString::new("expected factor:) [%c]\n").unwrap().as_ptr(), *item as c_int);
        return 0
      }

      expression
    },
    (Not, _) => !execIFactor(&mut it, b_ptr, bitpos, pPtr, err),
    _ => {
      // sprintf(err, CString::new("expected factor: B0..B9 P0..P9 BP number ( ) [%c]\n").unwrap().as_ptr(), *item as c_int);
      return 0
    },
  }
}
