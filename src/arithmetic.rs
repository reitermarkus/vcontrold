use super::*;


#[no_mangle]
pub unsafe extern fn execITerm(str: *mut *mut c_char, bPtr: *mut c_uchar, bitpos: c_char, pPtr: *mut c_char, err: *mut c_char) -> c_int {
  let mut item: *mut c_char = ptr::null_mut();
  let mut n: c_int = 0;

  // println!("execITerm: {}", CStr::from_ptr(*str).to_str().unwrap());

  let mut factor1 = execIFactor(str, bPtr, bitpos, pPtr, err);

  if *err != 0 {
    return 0
  }

  loop {
    let op = match nextToken(str, &mut item, &mut n as *mut c_int) {
      MAL => MAL,
      GETEILT => GETEILT,
      MODULO => MODULO,
      UND => UND,
      ODER => ODER,
      XOR => XOR,
      SHL => SHL,
      SHR => SHR,
      _ => {
        pushBack(str, n);
        //printf("  ret(%f)\n",factor1);
        return factor1
      },
    };

    let factor2 = execIFactor(str, bPtr, bitpos, pPtr, err);

    if *err != 0 {
      return 0
    }

    match op {
      MAL => factor1 *= factor2,
      GETEILT => factor1 /= factor2,
      MODULO => factor1 %= factor2,
      UND => factor1 &= factor2,
      ODER => factor1 |= factor2,
      XOR => factor1 ^= factor2,
      SHL => factor1 <<= factor2,
      SHR => factor1 >>= factor2,
      _ => unreachable!(),
    }
  }
}

#[no_mangle]
pub unsafe extern fn execTerm(str: *mut *mut c_char, bPtr: *mut c_uchar, floatV: c_float, err: *mut c_char) -> c_float {
  let mut item: *mut c_char = ptr::null_mut();
  let mut n: c_int = 0;

  // println!("execTerm: {}", CStr::from_ptr(*str).to_str().unwrap());

  let mut factor1: c_float = execFactor(str, bPtr, floatV, err);

  if *err != 0 {
    return 0.0
  }

  // println!("F1={}", factor1);

  loop {
    let op = match nextToken(str, &mut item, &mut n as *mut c_int) {
      MAL => MAL,
      GETEILT => GETEILT,
      _ => {
        pushBack(str, n);

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
      MAL => factor1 *= factor2,
      GETEILT => factor1 /= factor2,
      _ => unreachable!(),
    }
  }
}

#[no_mangle]
pub unsafe extern fn nextToken(input_string: *mut *mut c_char, c: *mut *mut c_char, count: *mut c_int) -> c_int {
  let string = CStr::from_ptr(*input_string).to_str().unwrap();

  let mut it = string.chars().skip_while(|c| c.is_whitespace());

  let skip_len = string.chars().take_while(|c| c.is_whitespace()).collect::<Vec<_>>().len();

  *c = (*input_string).add(skip_len);

  *count = 1;

  let token: c_int = if let Some(c) = it.next() {
    match c {
      '+' => PLUS,
      '-' => MINUS,
      '*' => MAL,
      '/' => GETEILT,
      '%' => MODULO,
      '(' => KAUF,
      ')' => KZU,
      'V' => VALUE,
      '^' => XOR,
      '&' => UND,
      '|' => ODER,
      '~' => NICHT,
      '0' => match it.next() {
        Some('x') => {
          *count += 1;
          HEX
        },
        _ => DIGIT,
      },
      '<' => {
        *count += 1;
        match it.next() {
          Some('<') => SHL,
          _ => ERROR,
        }
      },
      '>' => {
        *count += 1;
        match it.next() {
          Some('>') => SHR,
          _ => ERROR,
        }
      },
      'B' => {
        *count += 1;
        match it.next() {
          Some('0') => BYTE0,
          Some('1') => BYTE1,
          Some('2') => BYTE2,
          Some('3') => BYTE3,
          Some('4') => BYTE4,
          Some('5') => BYTE5,
          Some('6') => BYTE6,
          Some('7') => BYTE7,
          Some('8') => BYTE8,
          Some('9') => BYTE9,
          Some('P') => BITPOS,
          _ => ERROR,
        }
      },
      'P' => {
        *count += 1;
        match it.next() {
          Some('0') => PBYTE0,
          Some('1') => PBYTE1,
          Some('2') => PBYTE2,
          Some('3') => PBYTE3,
          Some('4') => PBYTE4,
          Some('5') => PBYTE5,
          Some('6') => PBYTE6,
          Some('7') => PBYTE7,
          Some('8') => PBYTE8,
          Some('9') => PBYTE9,
          _ => ERROR,
        }
      },
      '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => DIGIT,
      'a' | 'b' | 'c' | 'd' | 'e' | 'f' => HEXDIGIT,
      '.' => PUNKT,
      '\0' => END,
      _ => ERROR,
    }
  } else {
    END
  };

  *input_string = (*input_string).add(*count as usize);

  token as c_int
}

#[no_mangle]
pub unsafe extern fn execFactor(str: *mut *mut c_char, bPtr: *mut c_uchar, floatV: c_float, err: *mut c_char) -> c_float {
  let mut item: *mut c_char = ptr::null_mut();
  let mut n: c_int = 0;

  match nextToken(str, &mut item, &mut n as *mut c_int) {
    BYTE0 => *bPtr.add(0) as c_float,
    BYTE1 => *bPtr.add(1) as c_float,
    BYTE2 => *bPtr.add(2) as c_float,
    BYTE3 => *bPtr.add(3) as c_float,
    BYTE4 => *bPtr.add(4) as c_float,
    BYTE5 => *bPtr.add(5) as c_float,
    BYTE6 => *bPtr.add(6) as c_float,
    BYTE7 => *bPtr.add(7) as c_float,
    BYTE8 => *bPtr.add(8) as c_float,
    BYTE9 => *bPtr.add(9) as c_float,
    VALUE => floatV,
    HEX => {
      let mut hex = String::from("0x");

      loop {
        match nextToken(str, &mut item, &mut n as *mut c_int) {
          DIGIT | HEXDIGIT => hex.push(char::from(*item as u8)),
          _ => break,
        }
      }

      pushBack(str, n);

      let without_prefix = hex.trim_start_matches("0x");
      i32::from_str_radix(without_prefix, 16).unwrap_or(0) as c_float
    },
    DIGIT => {
      let mut dec = String::from("");

      dec.push(char::from(*item as u8));

      loop {
        match nextToken(str, &mut item, &mut n as *mut c_int) {
          DIGIT => dec.push(char::from(*item as u8)),
          PUNKT => {
            dec.push('.');

            loop {
              match nextToken(str, &mut item, &mut n as *mut c_int) {
                DIGIT => dec.push(char::from(*item as u8)),
                _ => break,
              }
            }

            break
          }
          _ => break,
        }
      }

      pushBack(str, n);

      dec.parse().unwrap_or(0.0)
    },
    KAUF => {
      let expression = execExpression(str, bPtr, floatV, err);

      if (*err) == 0 {
        return 0.0
      }

      if nextToken(str, &mut item, &mut n as *mut c_int) != KZU {
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


#[no_mangle]
pub unsafe extern fn execIFactor(str: *mut *mut c_char, bPtr: *mut c_uchar, bitpos: c_char, pPtr: *mut c_char, err: *mut c_char) -> c_int {
  let mut item: *mut c_char = ptr::null_mut();
  let mut n: c_int = 0;

  match nextToken(str, &mut item, &mut n as *mut c_int) {
    BYTE0 => *bPtr.add(0) as c_int & 0xff,
    BYTE1 => *bPtr.add(1) as c_int & 0xff,
    BYTE2 => *bPtr.add(2) as c_int & 0xff,
    BYTE3 => *bPtr.add(3) as c_int & 0xff,
    BYTE4 => *bPtr.add(4) as c_int & 0xff,
    BYTE5 => *bPtr.add(5) as c_int & 0xff,
    BYTE6 => *bPtr.add(6) as c_int & 0xff,
    BYTE7 => *bPtr.add(7) as c_int & 0xff,
    BYTE8 => *bPtr.add(8) as c_int & 0xff,
    BYTE9 => *bPtr.add(9) as c_int & 0xff,
    BITPOS => bitpos as c_int & 0xff,
    PBYTE0 => *pPtr.add(0) as c_int & 0xff,
    PBYTE1 => *pPtr.add(1) as c_int & 0xff,
    PBYTE2 => *pPtr.add(2) as c_int & 0xff,
    PBYTE3 => *pPtr.add(3) as c_int & 0xff,
    PBYTE4 => *pPtr.add(4) as c_int & 0xff,
    PBYTE5 => *pPtr.add(5) as c_int & 0xff,
    PBYTE6 => *pPtr.add(6) as c_int & 0xff,
    PBYTE7 => *pPtr.add(7) as c_int & 0xff,
    PBYTE8 => *pPtr.add(8) as c_int & 0xff,
    PBYTE9 => *pPtr.add(9) as c_int & 0xff,
    HEX => {
      let mut hex = String::from("0x");

      loop {
        match nextToken(str, &mut item, &mut n as *mut c_int) {
          DIGIT | HEXDIGIT => hex.push(char::from(*item as u8)),
          _ => break,
        }
      }

      pushBack(str, n);

      let without_prefix = hex.trim_start_matches("0x");
      c_int::from_str_radix(without_prefix, 16).unwrap_or(0)
    },
    DIGIT => {
      let mut dec = String::from("");

      dec.push(char::from(*item as u8));

      loop {
        match nextToken(str, &mut item, &mut n as *mut c_int) {
          DIGIT => dec.push(char::from(*item as u8)),
          PUNKT => {
            dec.push('.');

            loop {
              match nextToken(str, &mut item, &mut n as *mut c_int) {
                DIGIT => dec.push(char::from(*item as u8)),
                _ => break,
              }
            }

            break
          }
          _ => break,
        }
      }

      pushBack(str, n);

      dec.parse().unwrap_or(0)
    },
    KAUF => {
      let expression = execIExpression(str, bPtr, bitpos, pPtr, err);

      if (*err) == 0 {
        return 0
      }

      if nextToken(str, &mut item, &mut n as *mut c_int) != KZU {
        sprintf(err, CString::new("expected factor:) [%c]\n").unwrap().as_ptr(), *item as c_int);
        return 0
      }

      expression
    },
    NICHT => !execIFactor(str, bPtr, bitpos, pPtr, err),
    _ => {
      sprintf(err, CString::new("expected factor: B0..B9 P0..P9 BP number ( ) [%c]\n").unwrap().as_ptr(), *item as c_int);
      return 0
    },
  }
}
