use std::fmt;
use std::str::FromStr;

use super::{Number, Tok, Op, Var, lex};

// or     :=  or | xor      |  xor
// xor    :=  xor ^ and     |  and
// and    :=  and & shift   |  shift
// shift  :=  shift << add  |  shift >> add  |  add
// add    :=  add - mul     |  add + mul     |  mul
// mul    :=  mul * final   |  mul / final   |  mul % final | final
// final  :=  number        |  var           |  ( or )

#[derive(Clone)]
pub enum ParseNode {
  Binary(Op, Box<ParseNode>, Box<ParseNode>),
  Unary(Op, Box<ParseNode>),
  Number(Number),
  Var(Var),
}

impl fmt::Debug for ParseNode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ParseNode::Binary(op, lhs, rhs) => match op {
        Op::Add => write!(f, "({:?} + {:?})", lhs, rhs),
        Op::Sub => write!(f, "({:?} - {:?})", lhs, rhs),
        Op::Mul => write!(f, "({:?} * {:?})", lhs, rhs),
        Op::Div => write!(f, "({:?} / {:?})", lhs, rhs),
        Op::Mod => write!(f, "({:?} % {:?})", lhs, rhs),
        Op::And => write!(f, "({:?} & {:?})", lhs, rhs),
        Op::Xor => write!(f, "({:?} ^ {:?})", lhs, rhs),
        Op::Or  => write!(f, "({:?} | {:?})", lhs, rhs),
        Op::Shl => write!(f, "({:?} << {:?})", lhs, rhs),
        Op::Shr => write!(f, "({:?} >> {:?})", lhs, rhs),
        _ => unreachable!(),
      },
      ParseNode::Unary(op, expr) => match op {
        Op::Not => write!(f, "~({:?})", expr),
        Op::Sub => write!(f, "-({:?})", expr),
        _ => unreachable!(),
      },
      ParseNode::Number(n) => write!(f, "{:?}", n),
      ParseNode::Var(var) => write!(f, "{:?}", var),
    }
  }
}

impl FromStr for ParseNode {
  type Err = String;

  fn from_str(s: &str) -> Result<ParseNode, Self::Err> {
    parse_root(&lex(s)?)
  }
}

fn parse_root(tokens: &Vec<Tok>) -> Result<ParseNode, String> {
  Ok(parse_or(tokens, 0)?.0)
}

fn parse_or(tokens: &Vec<Tok>, pos: usize) -> Result<(ParseNode, usize), String> {
  let (node, next_pos) = parse_xor(tokens, pos)?;
  parse_or_rhs(node, tokens, next_pos)
}

fn parse_or_rhs(lhs: ParseNode, tokens: &Vec<Tok>, pos: usize) -> Result<(ParseNode, usize), String> {
  match tokens.get(pos) {
    Some(Tok::Op(op @ Op::Or)) => {
      let (rhs, next_pos) = parse_xor(tokens, pos + 1)?;

      let node = ParseNode::Binary(*op, Box::new(lhs), Box::new(rhs));

      let (rec_node, next_next_pos) = parse_or_rhs(node, tokens, next_pos)?;

      Ok((rec_node, next_next_pos))
    },
    _ => Ok((lhs, pos))
  }
}

fn parse_xor(tokens: &Vec<Tok>, pos: usize) -> Result<(ParseNode, usize), String> {
  let (node, next_pos) = parse_and(tokens, pos)?;
  parse_xor_rhs(node, tokens, next_pos)
}

fn parse_xor_rhs(lhs: ParseNode, tokens: &Vec<Tok>, pos: usize) -> Result<(ParseNode, usize), String> {
  match tokens.get(pos) {
    Some(Tok::Op(op @ Op::Xor)) => {
      let (rhs, next_pos) = parse_and(tokens, pos + 1)?;

      let node = ParseNode::Binary(*op, Box::new(lhs), Box::new(rhs));

      let (rec_node, next_next_pos) = parse_xor_rhs(node, tokens, next_pos)?;

      Ok((rec_node, next_next_pos))
    },
    _ => Ok((lhs, pos))
  }
}

fn parse_and(tokens: &Vec<Tok>, pos: usize) -> Result<(ParseNode, usize), String> {
  let (node, next_pos) = parse_shift(tokens, pos)?;
  parse_and_rhs(node, tokens, next_pos)
}

fn parse_and_rhs(lhs: ParseNode, tokens: &Vec<Tok>, pos: usize) -> Result<(ParseNode, usize), String> {
  match tokens.get(pos) {
    Some(Tok::Op(op @ Op::And)) => {
      let (rhs, next_pos) = parse_shift(tokens, pos + 1)?;

      let node = ParseNode::Binary(*op, Box::new(lhs), Box::new(rhs));

      let (rec_node, next_next_pos) = parse_and_rhs(node, tokens, next_pos)?;

      Ok((rec_node, next_next_pos))
    },
    _ => Ok((lhs, pos))
  }
}

fn parse_shift(tokens: &Vec<Tok>, pos: usize) -> Result<(ParseNode, usize), String> {
  let (node, next_pos) = parse_add(tokens, pos)?;
  parse_shift_rhs(node, tokens, next_pos)
}

fn parse_shift_rhs(lhs: ParseNode, tokens: &Vec<Tok>, pos: usize) -> Result<(ParseNode, usize), String> {
  match tokens.get(pos) {
    Some(Tok::Op(op @ Op::Shl)) |
    Some(Tok::Op(op @ Op::Shr)) => {
      let (rhs, next_pos) = parse_add(tokens, pos + 1)?;

      let node = ParseNode::Binary(*op, Box::new(lhs), Box::new(rhs));

      let (rec_node, next_next_pos) = parse_shift_rhs(node, tokens, next_pos)?;

      Ok((rec_node, next_next_pos))
    },
    _ => Ok((lhs, pos))
  }
}

fn parse_add(tokens: &Vec<Tok>, pos: usize) -> Result<(ParseNode, usize), String> {
  let (lhs, next_pos) = parse_mul(tokens, pos)?;
  parse_add_rhs(lhs, tokens, next_pos)
}

fn parse_add_rhs(lhs: ParseNode, tokens: &Vec<Tok>, pos: usize) -> Result<(ParseNode, usize), String> {
  match tokens.get(pos) {
    Some(Tok::Op(op @ Op::Add)) | Some(Tok::Op(op @ Op::Sub)) => {
      let (rhs, next_pos) = parse_mul(tokens, pos + 1)?;

      let node = ParseNode::Binary(*op, Box::new(lhs), Box::new(rhs));

      let (rec_node, next_next_pos) = parse_add_rhs(node, tokens, next_pos)?;

      Ok((rec_node, next_next_pos))
    },
    _ => Ok((lhs, pos)),
  }
}

fn parse_mul(tokens: &Vec<Tok>, pos: usize) -> Result<(ParseNode, usize), String> {
  let (node, next_pos) = parse_final(tokens, pos)?;
  parse_mul_rhs(node, tokens, next_pos)
}

fn parse_mul_rhs(lhs: ParseNode, tokens: &Vec<Tok>, pos: usize) -> Result<(ParseNode, usize), String> {
  match tokens.get(pos) {
    Some(Tok::Op(op @ Op::Mul)) |
    Some(Tok::Op(op @ Op::Div)) |
    Some(Tok::Op(op @ Op::Mod))  => {
      let (rhs, next_pos) = parse_final(tokens, pos + 1)?;

      let node = ParseNode::Binary(*op, Box::new(lhs), Box::new(rhs));

      let (rec_node, next_next_pos) = parse_mul_rhs(node, tokens, next_pos)?;

      Ok((rec_node, next_next_pos))
    },
    _ => Ok((lhs, pos))
  }
}

fn parse_final(tokens: &Vec<Tok>, pos: usize) -> Result<(ParseNode, usize), String> {
  let c = tokens.get(pos).ok_or("Unexpected end of input, expected open paren, variable or number")?;

  match c {
    Tok::Op(op @ Op::Not) | Tok::Op(op @ Op::Sub) => {
      let (node, next_pos) = parse_final(tokens, pos + 1)?;
      Ok((ParseNode::Unary(*op, Box::new(node)), next_pos))
    },
    Tok::Number(n) => Ok((ParseNode::Number(n.clone()), pos + 1)),
    Tok::Var(var) => Ok((ParseNode::Var(var.clone()), pos + 1)),
    Tok::ParOpen => {
      parse_or(tokens, pos + 1).and_then(|(or_node, next_pos)| {
        let c2 = tokens.get(next_pos).ok_or("unexpected end of input, expected ')'")?;

        match c2 {
          Tok::ParClose => {
            Ok((or_node, next_pos + 1))
          },
          c2 => {
            Err(format!("expected ')', but found {:?}", c2))
          },
        }
      })
    },
    t => {
      Err(format!("unexpected token {:?}, expected '(', variable or number", t))
    }
  }
}
