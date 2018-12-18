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

unsafe fn push_back(str: *mut *mut c_char, count: usize) {
  *str = (*str).sub(count);
}

#[no_mangle]
pub unsafe extern fn execIExpression(str: *mut *mut c_char, bInPtr: *mut c_uchar, bitpos: c_char, pPtr: *mut c_char, err: *mut c_char) -> c_int {
  let mut bPtr: [c_uchar; 10] = mem::zeroed();

  // println!("execIExpression: {}", CStr::from_ptr(*str).to_str().unwrap());

  // Tweak bPtr Bytes 0..9 and copy them to nPtr
  // We did not receive characters
  for n in 0..10 {
    bPtr[n] = *bInPtr.add(n);
  }

  let mut item: *mut c_char = ptr::null_mut();
  let mut n = 0;

  let mut term1: c_int = match nextToken(str, &mut item, &mut n) {
    Plus => execITerm(str, &mut bPtr as *mut _ as *mut c_uchar, bitpos, pPtr, err),
    Minus => -execITerm(str, &mut bPtr as *mut _ as *mut c_uchar, bitpos, pPtr, err),
    Not => !execITerm(str, &mut bPtr as *mut _ as *mut c_uchar, bitpos, pPtr, err),
    _ => {
      push_back(str, n);
      execITerm(str, &mut bPtr as *mut _ as *mut c_uchar, bitpos, pPtr, err)
    },
  };

  if *err != 0 {
    return 0
  }

  loop {
    let term2: c_int = match nextToken(str, &mut item, &mut n) {
      Plus => execITerm(str, &mut bPtr as *mut _ as *mut c_uchar, bitpos, pPtr, err),
      Minus => -execITerm(str, &mut bPtr as *mut _ as *mut c_uchar, bitpos, pPtr, err),
      Not => !execITerm(str, &mut bPtr as *mut _ as *mut c_uchar, bitpos, pPtr, err),
      _ => {
        push_back(str, n);
        return term1
      }
    };

    if *err != 0 {
      return 0
    }

    term1 += term2;
  }
}

#[no_mangle]
pub unsafe extern fn execExpression(str: *mut *mut c_char, bInPtr: *mut c_uchar, floatV: c_float, err: *mut c_char) -> c_float {
  let mut bPtr: [c_uchar; 10] = mem::zeroed();

  // println!("execExpression: {}", CStr::from_ptr(*str).to_str().unwrap());

  // Tweak bPtr Bytes 0..9 and copy them to nPtr
  // We did not receive characters
  for n in 0..10 {
    bPtr[n] = *bInPtr.add(n);
  }

  let mut item: *mut c_char = ptr::null_mut();
  let mut n = 0;

  let f: c_float = match nextToken(str, &mut item, &mut n) {
    Plus => 1.0,
    Minus => -1.0,
    _ => {
      push_back(str, n);
      1.0
    },
  };

  let mut term1 = execTerm(str, &mut bPtr as *mut _ as *mut c_uchar, floatV, err) * f;

  if *err != 0 {
    return 0.0
  }

  // println!("T1={}", term1);

  loop {
    let f: c_float = match nextToken(str, &mut item, &mut n) {
      Plus => 1.0,
      Minus => -1.0,
      _ => {
        // println!("Exp={}", term1);
        push_back(str, n);
        return term1
      }
    };

    let term2 = execTerm(str, &mut bPtr as *mut _ as *mut c_uchar, floatV, err);

    if *err != 0 {
      return 0.0
    }

    term1 += term2 * f;
  }
}

unsafe fn execITerm(str: *mut *mut c_char, bPtr: *mut c_uchar, bitpos: c_char, pPtr: *mut c_char, err: *mut c_char) -> c_int {
  let mut item: *mut c_char = ptr::null_mut();
  let mut n = 0;

  // println!("execITerm: {}", CStr::from_ptr(*str).to_str().unwrap());

  let mut factor1 = execIFactor(str, bPtr, bitpos, pPtr, err);

  if *err != 0 {
    return 0
  }

  loop {
    let op = match nextToken(str, &mut item, &mut n) {
      op @ Mul | Div | Mod | And | Or | Xor | Shl | Shr => op,
      _ => {
        push_back(str, n);
        //printf("  ret(%f)\n",factor1);
        return factor1
      },
    };

    let factor2 = execIFactor(str, bPtr, bitpos, pPtr, err);

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

unsafe fn execTerm(str: *mut *mut c_char, bPtr: *mut c_uchar, floatV: c_float, err: *mut c_char) -> c_float {
  let mut item: *mut c_char = ptr::null_mut();
  let mut n = 0;

  // println!("execTerm: {}", CStr::from_ptr(*str).to_str().unwrap());

  let mut factor1: c_float = execFactor(str, bPtr, floatV, err);

  if *err != 0 {
    return 0.0
  }

  // println!("F1={}", factor1);

  loop {
    let op = match nextToken(str, &mut item, &mut n) {
      op @ Mul | Div => op,
      _ => {
        push_back(str, n);

        // println!("ret({})", factor1);

        return factor1
      },
    };

    let factor2: c_float = execFactor(str, bPtr, floatV, err);

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

unsafe fn nextToken(input_string: *mut *mut c_char, c: *mut *mut c_char, count: &mut usize) -> Token {
  let string = CStr::from_ptr(*input_string).to_str().unwrap();

  let mut it = string.chars().skip_while(|c| c.is_whitespace());

  let skip_len = string.chars().take_while(|c| c.is_whitespace()).collect::<Vec<_>>().len();

  *c = (*input_string).add(skip_len);

  *count = 1;

  let token = if let Some(c) = it.next() {
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
      '0' => match it.next() {
        Some('x') => {
          *count += 1;
          Hex
        },
        _ => Digit('0'),
      },
      '<' => {
        *count += 1;
        match it.next() {
          Some('<') => Shl,
          _ => Error,
        }
      },
      '>' => {
        *count += 1;
        match it.next() {
          Some('>') => Shr,
          _ => Error,
        }
      },
      'B' => {
        *count += 1;
        match it.next() {
          Some(c @ '0'..='9') => Byte(u32::from(c) as usize),
          Some('P') => BitPos,
          _ => Error,
        }
      },
      'P' => {
        *count += 1;
        match it.next() {
          Some(c @ '0'..='9') => PByte(u32::from(c) as usize),
          _ => Error,
        }
      },
      c @ '1'..='9' => Digit(c),
      c @ 'a'..='f' => HexDigit(c),
      '.' => Dot,
      '\0' => End,
      _ => Error,
    }
  } else {
    End
  };

  *input_string = (*input_string).add(*count);

  token
}

unsafe fn execFactor(str: *mut *mut c_char, bPtr: *mut c_uchar, floatV: c_float, err: *mut c_char) -> c_float {
  let mut item: *mut c_char = ptr::null_mut();
  let mut n = 0;

  match nextToken(str, &mut item, &mut n) {
    Byte(i) => *bPtr.add(i) as c_float,
    Value => floatV,
    Hex => {
      let mut hex = String::from("0x");

      loop {
        match nextToken(str, &mut item, &mut n) {
          Digit(c) | HexDigit(c) => hex.push(c),
          _ => break,
        }
      }

      push_back(str, n);

      let without_prefix = hex.trim_start_matches("0x");
      i32::from_str_radix(without_prefix, 16).unwrap_or(0) as c_float
    },
    Digit(c) => {
      let mut dec = c.to_string();

      loop {
        match nextToken(str, &mut item, &mut n) {
          Digit(c) => dec.push(c),
          Dot => {
            dec.push('.');

            loop {
              match nextToken(str, &mut item, &mut n) {
                Digit(c) => dec.push(c),
                _ => break,
              }
            }

            break
          }
          _ => break,
        }
      }

      push_back(str, n);

      dec.parse().unwrap_or(0.0)
    },
    ParOpen => {
      let expression = execExpression(str, bPtr, floatV, err);

      if (*err) == 0 {
        return 0.0
      }

      if nextToken(str, &mut item, &mut n) != ParClose {
        sprintf(err, CString::new("expected factor:) [%c]\n").unwrap().as_ptr(), *item as c_int);
        return 0.0
      }

      expression
    },
    _ => {
      sprintf(err, CString::new("expected factor: B0..B9 number ( ) [%c]\n").unwrap().as_ptr(), *item as c_int);
      return 0.0
    },
  }
}

unsafe fn execIFactor(str: *mut *mut c_char, bPtr: *mut c_uchar, bitpos: c_char, pPtr: *mut c_char, err: *mut c_char) -> c_int {
  let mut item: *mut c_char = ptr::null_mut();
  let mut n = 0;

  match nextToken(str, &mut item, &mut n) {
    Byte(i) => *bPtr.add(i) as c_int & 0xff,
    BitPos => bitpos as c_int & 0xff,
    PByte(i) => *pPtr.add(i) as c_int & 0xff,
    Hex => {
      let mut hex = String::from("0x");

      loop {
        match nextToken(str, &mut item, &mut n) {
          Digit(c) | HexDigit(c) => hex.push(c),
          _ => break,
        }
      }

      push_back(str, n);

      let without_prefix = hex.trim_start_matches("0x");
      c_int::from_str_radix(without_prefix, 16).unwrap_or(0)
    },
    Digit(c) => {
      let mut dec = c.to_string();

      loop {
        match nextToken(str, &mut item, &mut n) {
          Digit(c) => dec.push(c),
          Dot => {
            dec.push('.');

            loop {
              match nextToken(str, &mut item, &mut n) {
                Digit(c) => dec.push(c),
                _ => break,
              }
            }

            break
          }
          _ => break,
        }
      }

      push_back(str, n);

      dec.parse().unwrap_or(0)
    },
    ParOpen => {
      let expression = execIExpression(str, bPtr, bitpos, pPtr, err);

      if (*err) == 0 {
        return 0
      }

      if nextToken(str, &mut item, &mut n) != ParClose {
        sprintf(err, CString::new("expected factor:) [%c]\n").unwrap().as_ptr(), *item as c_int);
        return 0
      }

      expression
    },
    Not => !execIFactor(str, bPtr, bitpos, pPtr, err),
    _ => {
      sprintf(err, CString::new("expected factor: B0..B9 P0..P9 BP number ( ) [%c]\n").unwrap().as_ptr(), *item as c_int);
      return 0
    },
  }
}
