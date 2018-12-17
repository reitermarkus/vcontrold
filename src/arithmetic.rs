use super::*;

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
