use std::fmt;
use std::iter::Peekable;

use super::Number;

#[derive(PartialEq, Clone, Copy)]
pub enum Op {
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  Not,
  And,
  Or,
  Xor,
  Shl,
  Shr,
}

impl fmt::Debug for Op {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Op::Add => write!(f, "'+'"),
      Op::Sub => write!(f, "'-'"),
      Op::Mul => write!(f, "'*'"),
      Op::Div => write!(f, "'/'"),
      Op::Mod => write!(f, "'%'"),
      Op::Not => write!(f, "'~'"),
      Op::And => write!(f, "'&'"),
      Op::Or => write!(f, "'|'"),
      Op::Xor => write!(f, "'^'"),
      Op::Shl => write!(f, "\"<<\""),
      Op::Shr => write!(f, "\">>\""),
    }
  }
}

#[derive(PartialEq, Clone)]
pub enum Var {
  Value,
  Byte(usize),
}

impl fmt::Debug for Var {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Var::Value => write!(f, "$v"),
      Var::Byte(i) => write!(f, "$b{}", i),
    }
  }
}

#[derive(PartialEq)]
pub enum Tok {
  Number(Number),
  Op(Op),
  ParOpen,
  ParClose,
  Var(Var),
}

impl fmt::Debug for Tok {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Tok::Number(n) => write!(f, "{:?}", n),
      Tok::Op(op) => write!(f, "{:?}", op),
      Tok::ParOpen => write!(f, "'('"),
      Tok::ParClose => write!(f, "')'"),
      Tok::Var(var) => write!(f, "{:?}", var),
    }
  }
}

pub fn lex(input: &str) -> Result<Vec<Tok>, String> {
  use self::Tok::*;
  use self::Op::*;

  let mut tokens = Vec::new();

  let mut it = input.chars().peekable();

  while let Some(c) = it.peek() {
    match *c {
      ' ' => {
        it.next();
      },
      '0'...'9' => {
        tokens.push(get_number(&mut it)?);
      },
      '+' => {
        it.next();
        tokens.push(Op(Add));
      },
      '-' => {
        it.next();
        tokens.push(Op(Sub));
      },
      '*' => {
        it.next();
        tokens.push(Op(Mul));
      },
      '/' => {
        it.next();
        tokens.push(Op(Div));
      },
      '%' => {
        it.next();
        tokens.push(Op(Mod));
      },
      '~' => {
        it.next();
        tokens.push(Op(Not));
      },
      '&' => {
        it.next();
        tokens.push(Op(And));
      },
      '|' => {
        it.next();
        tokens.push(Op(Or));
      },
      '^' => {
        it.next();
        tokens.push(Op(Xor));
      },
      '<' => {
        match it.peek() {
          Some('<') => {
            it.next();
            tokens.push(Op(Shl));
          },
          Some(c) => return Err(format!("unexpected character '{}', expected '<'", c)),
          None => return Err(String::from("unexpected end of string")),
        }
      },
      '>' => {
        match it.peek() {
          Some('>') => {
            it.next();
            tokens.push(Op(Shr));
          },
          Some(c) => return Err(format!("unexpected character '{}', expected '>'", c)),
          None => return Err(String::from("unexpected end of string")),
        }
      },
      '(' => {
        it.next();
        tokens.push(ParOpen);
      },
      ')' => {
        it.next();
        tokens.push(ParClose);
      },
      '$' => {
        it.next();
        tokens.push(get_var(&mut it)?);
      },
      _ => {
        return Err(format!("unexpected character '{}'", c))
      },
    }
  }

  Ok(tokens)
}

fn get_number<T: Iterator<Item = char>>(it: &mut Peekable<T>) -> Result<Tok, String> {
  let mut number = String::new();
  let mut radix = 10;

  while let Some(c) = it.peek() {
    match *c {
      c @ '0' if number.len() == 0 => {
        it.next();

        if it.peek() == Some(&'x') {
          it.next();

          match it.peek() {
            Some('0'...'9') | Some('a'...'f') | Some('A'...'F' ) => (),
            Some(c) => return Err(format!("unexpected character '{}', expected '0'...'f'", c)),
            None => return Err(String::from("unexpected end of string")),
          }

          radix = 16;

          while let Some(c) = it.peek() {
            match *c {
              c @ '0'...'9' | c @ 'a'...'f' | c @ 'A'...'F'  => {
                it.next();
                number.push(c);
              },
              _ => break
            }
          }

          break;
        }

        number.push(c);
      },
      c @ '0'...'9' => {
        it.next();
        number.push(c);
      }
      c @ '.' => {
        it.next();
        number.push(c);

        match it.peek() {
          Some('0'...'9') => (),
          Some(c)=>  return Err(format!("unexpected character '{}', expected '0'...'9'", c)),
          None => return Err(String::from("unexpected end of string")),
        }

        while let Some(c) = it.peek() {
          match *c {
            c @ '0'...'9' => {
              it.next();
              number.push(c);
            },
            _ => break,
          }
        }

        return Ok(Tok::Number(Number::Float(number.parse::<f32>().map_err(|err| err.to_string())?)))
      },
      _ => break,
    }
  }

  Ok(Tok::Number(Number::Int(i32::from_str_radix(&number, radix).map_err(|err| err.to_string())?)))
}

fn get_var<T: Iterator<Item = char>>(it: &mut Peekable<T>) -> Result<Tok, String> {
  use self::Tok::*;
  use self::Var::*;

  match it.peek() {
    Some('v') => {
      it.next();
      Ok(Var(Value))
    },
    Some('b') => {
      it.next();
      if let Some(c) = it.peek() {
        match *c {
          c @ '0'...'9' => {
            it.next();
            Ok(Var(Byte(c.to_digit(10).unwrap() as usize)))
          },
          c => Err(format!("unexpected character '{}'", c)),
        }
      } else {
        Err(String::from("unexpected end of string"))
      }
    },
    Some(c) => Err(format!("unexpected character '{}'", c)),
    None => Err(String::from("unexpected end of string")),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn number_float() {
    assert_eq!(lex("3.14").unwrap(), vec![Tok::Number(Number::Float(3.14))]);
  }

  #[test]
  fn number_dec() {
    assert_eq!(lex("42").unwrap(), vec![Tok::Number(Number::Int(42))]);
  }

  #[test]
  fn number_hex() {
    assert_eq!(lex("0xff").unwrap(), vec![Tok::Number(Number::Int(0xff))]);
    assert_eq!(lex("0xFF").unwrap(), vec![Tok::Number(Number::Int(0xff))]);
  }
}
