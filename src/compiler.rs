use crate::{
  chunk::{Chunk, OpCode},
  scanner::{Scanner, Token, TokenType},
  value::Value,
  CedarError,
};
use std::{borrow::Cow, fmt, iter::Peekable, vec};

pub fn compile(source: String) -> Result<Chunk, CedarError> {
  let tokens = Scanner::new(source).scan()?;

  // The input was empty and so we only have an EOF token
  if tokens.len() == 1 {
    let mut chunk = Chunk::new();
    chunk.write_chunk(OpCode::Return.into(), None, 0)?;
    Ok(chunk)
  } else {
    Ok(TokenIter::new(tokens).compile()?)
  }
}

pub struct TokenIter {
  iter: Peekable<vec::IntoIter<Token>>,
  previous: Option<Token>,
  current: Option<Token>,
  chunk: Chunk,
  rules: [ParseRule; 39],
}

impl fmt::Debug for TokenIter {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    self.chunk.disassemble("MAIN");
    write!(
      f,
      "Iter: {:#?}\nPrevious: {:#?}\nCurrent: {:#?}\n",
      self.iter, self.previous, self.current
    )
  }
}

impl TokenIter {
  fn new(tokens: Vec<Token>) -> TokenIter {
    Self {
      iter: tokens.into_iter().peekable(),
      previous: None,
      current: None,
      chunk: Chunk::new(),
      rules: [
        // LeftParen
        ParseRule::new(Some(TokenIter::grouping), None, Precedence::None),
        // RightParen
        ParseRule::new(None, None, Precedence::None),
        // LeftBrace
        ParseRule::new(None, None, Precedence::None),
        // RightBrace
        ParseRule::new(None, None, Precedence::None),
        // Comma
        ParseRule::new(None, None, Precedence::None),
        // Dot
        ParseRule::new(None, None, Precedence::None),
        // Minus
        ParseRule::new(
          Some(TokenIter::unary),
          Some(TokenIter::binary),
          Precedence::Term,
        ),
        // Plus
        ParseRule::new(None, Some(TokenIter::binary), Precedence::Term),
        // Semicolon
        ParseRule::new(None, None, Precedence::None),
        // Slash
        ParseRule::new(None, Some(TokenIter::binary), Precedence::Factor),
        // Star
        ParseRule::new(None, Some(TokenIter::binary), Precedence::Factor),
        // Bang
        ParseRule::new(Some(TokenIter::unary), None, Precedence::None),
        // BangEqual
        ParseRule::new(None, Some(TokenIter::binary), Precedence::Equality),
        // Equal
        ParseRule::new(None, None, Precedence::None),
        // EqualEqual
        ParseRule::new(None, Some(TokenIter::binary), Precedence::Equality),
        // Greater
        ParseRule::new(None, Some(TokenIter::binary), Precedence::Comparison),
        // GreaterEqual
        ParseRule::new(None, Some(TokenIter::binary), Precedence::Comparison),
        // Less
        ParseRule::new(None, Some(TokenIter::binary), Precedence::Comparison),
        // LessEqual
        ParseRule::new(None, Some(TokenIter::binary), Precedence::Comparison),
        // Identifier
        ParseRule::new(Some(TokenIter::variable), None, Precedence::None),
        // String
        ParseRule::new(Some(TokenIter::string), None, Precedence::None),
        // Number
        ParseRule::new(Some(TokenIter::number), None, Precedence::Factor),
        // And
        ParseRule::new(None, None, Precedence::None),
        // Class
        ParseRule::new(None, None, Precedence::None),
        // Else
        ParseRule::new(None, None, Precedence::None),
        // False
        ParseRule::new(Some(TokenIter::literal), None, Precedence::None),
        // For
        ParseRule::new(None, None, Precedence::None),
        // Fun
        ParseRule::new(None, None, Precedence::None),
        // If
        ParseRule::new(None, None, Precedence::None),
        // Null
        ParseRule::new(Some(TokenIter::literal), None, Precedence::None),
        // Or
        ParseRule::new(None, None, Precedence::None),
        // Print
        ParseRule::new(None, None, Precedence::None),
        // Return
        ParseRule::new(None, None, Precedence::None),
        // Super
        ParseRule::new(None, None, Precedence::None),
        // Self
        ParseRule::new(None, None, Precedence::None),
        // True
        ParseRule::new(Some(TokenIter::literal), None, Precedence::None),
        // Let
        ParseRule::new(None, None, Precedence::None),
        // While
        ParseRule::new(None, None, Precedence::None),
        // EOF
        ParseRule::new(None, None, Precedence::None),
      ],
    }
  }
  fn compile(mut self) -> Result<Chunk, CedarError> {
    let mut failed = false;
    self.advance();
    while !self.match_token(&TokenType::EOF)? {
      if let Err(e) = self.declaration() {
        failed = true;
        eprintln!("{}", e);
        self.synchronize()?;
      }
    }
    if failed {
      Err(CompilerError::failed().into())
    } else {
      self.end_compiler()?;
      Ok(self.chunk)
    }
  }
  fn advance(&mut self) {
    let current = self.iter.next();
    let previous = self.current.take();
    self.previous = previous;
    self.current = current;
  }
  fn consume<M>(&mut self, ty: TokenType, message: M) -> Result<(), CedarError>
  where
    M: Into<Cow<'static, str>>,
  {
    if self
      .current
      .as_ref()
      .ok_or_else(|| CompilerError::ice("No current value while in consume function"))?
      .ty
      == ty
    {
      self.advance();
      Ok(())
    } else {
      Err(CompilerError::new(self.current.as_ref().unwrap(), message).into())
    }
  }
  fn match_token(&mut self, ty: &TokenType) -> Result<bool, CedarError> {
    if !self.check(ty)? {
      Ok(false)
    } else {
      self.advance();
      Ok(true)
    }
  }
  fn check(&mut self, ty: &TokenType) -> Result<bool, CedarError> {
    Ok(
      self
        .current
        .as_ref()
        .ok_or_else(|| CompilerError::ice("No current value while in check function"))?
        .ty
        == *ty,
    )
  }
  fn synchronize(&mut self) -> Result<(), CedarError> {
    while !self.check(&TokenType::EOF)? {
      if self
        .previous
        .as_ref()
        .map(|p| p.ty == TokenType::Semicolon)
        .unwrap_or(false)
      {
        break;
      }

      match self
        .current
        .as_ref()
        .ok_or_else(|| CompilerError::ice("No current value in synchronize"))?
        .ty
      {
        TokenType::Class
        | TokenType::Fn
        | TokenType::Let
        | TokenType::For
        | TokenType::If
        | TokenType::While
        | TokenType::Print
        | TokenType::Return => break,
        _ => self.advance(),
      }
    }
    Ok(())
  }
  fn parse_variable(&mut self) -> Result<Cow<'static, str>, CedarError> {
    self.consume(TokenType::Identifier, "Expect variable name.")?;
    Ok(
      self
        .previous
        .clone()
        .ok_or_else(|| CompilerError::ice("No previous value in let_declaration"))?
        .lexeme,
    )
  }
  fn emit_byte(&mut self, byte: OpCode, value: Option<Value>) -> Result<(), CedarError> {
    let line = self
      .previous
      .as_ref()
      .map(|p| p.line)
      .ok_or_else(|| CompilerError::ice("No previous value when calling emit_byte"))?;
    self.chunk.write_chunk(byte.into(), value, line)
  }
  fn emit_return(&mut self) -> Result<(), CedarError> {
    self.emit_byte(OpCode::Return, None)
  }
  fn end_compiler(&mut self) -> Result<(), CedarError> {
    self.emit_return()
  }
  fn emit_constant(&mut self, value: Option<Value>) -> Result<(), CedarError> {
    self.emit_byte(OpCode::Constant, value)
  }
  fn number(&mut self, _: bool) -> Result<(), CedarError> {
    let number = self
      .previous
      .as_ref()
      .map(|c| -> Result<Value, CedarError> { Ok(Value::Number(c.lexeme.parse()?)) })
      .transpose()?;
    self.emit_constant(number)
  }
  fn string(&mut self, _: bool) -> Result<(), CedarError> {
    let mut string = self
      .previous
      .as_ref()
      .ok_or_else(|| CompilerError::ice("No previous value in string"))?
      .lexeme
      .clone();
    // Empty string case
    if string.len() == 2 {
      *string.to_mut() = "".into();
    } else {
      *string.to_mut() = string[1..string.len() - 1].into();
    }
    self.emit_constant(Some(Value::String(string)))
  }
  fn literal(&mut self, _: bool) -> Result<(), CedarError> {
    match self.previous.as_ref().map(|t| t.ty) {
      Some(TokenType::False) => self.emit_byte(OpCode::False, None),
      Some(TokenType::True) => self.emit_byte(OpCode::True, None),
      Some(TokenType::Null) => self.emit_byte(OpCode::Null, None),
      _ => unreachable!(),
    }
  }
  fn parse_precedence(&mut self, precedence: Precedence) -> Result<(), CedarError> {
    self.advance();
    let token = self
      .previous
      .as_ref()
      .ok_or_else(|| CompilerError::ice("No previous value in parse_precedence"))?;
    let prefix = self.get_rule(&token.ty).prefix;
    let can_assign = precedence <= Precedence::Assignment;
    match prefix {
      Some(prefix) => {
        prefix(self, can_assign)?;
      }
      None => return Err(CompilerError::new(&token, "Expected expression").into()),
    }
    while {
      match self.current.as_ref() {
        Some(token) => precedence <= self.get_rule(&token.ty).precedence,
        None => false,
      }
    } {
      self.advance();
      let token = self
        .previous
        .as_ref()
        .ok_or_else(|| CompilerError::ice("No previous value in parse_precedence"))?;
      let infix = self.get_rule(&token.ty).infix;
      match infix {
        Some(infix) => {
          infix(self, can_assign)?;
        }
        None => return Err(CompilerError::new(&token, "Expected infix function").into()),
      }
    }

    if can_assign && self.match_token(&TokenType::Equal)? {
      Err(CompilerError::new(&self.previous.as_ref().unwrap(), "Expected infix function").into())
    } else {
      Ok(())
    }
  }
  fn get_rule(&self, ty: &TokenType) -> &ParseRule {
    &self.rules[ty.as_usize()]
  }
  fn declaration(&mut self) -> Result<(), CedarError> {
    if self.match_token(&TokenType::Let)? {
      self.let_declaration()
    } else {
      self.statement()
    }
  }
  fn let_declaration(&mut self) -> Result<(), CedarError> {
    let global = self.parse_variable()?;
    if self.match_token(&TokenType::Equal)? {
      self.expression()?;
    } else {
      self.emit_byte(OpCode::Null, None)?;
    }
    self.consume(
      TokenType::Semicolon,
      "Expect ';' after variable declaration.",
    )?;
    self.define_variable(global)
  }
  fn define_variable(&mut self, global: Cow<'static, str>) -> Result<(), CedarError> {
    self.emit_byte(OpCode::DefineGlobal, Some(Value::String(global)))
  }
  fn variable(&mut self, can_assign: bool) -> Result<(), CedarError> {
    self.named_variable(can_assign)
  }
  fn named_variable(&mut self, can_assign: bool) -> Result<(), CedarError> {
    let arg = self
      .previous
      .clone()
      .ok_or_else(|| CompilerError::ice("No previous value in variable"))?
      .lexeme;
    if can_assign && self.match_token(&TokenType::Equal)? {
      self.expression()?;
      self.emit_byte(OpCode::SetGlobal, Some(Value::String(arg)))
    } else {
      self.emit_byte(OpCode::GetGlobal, Some(Value::String(arg)))
    }
  }
  fn statement(&mut self) -> Result<(), CedarError> {
    if self.match_token(&TokenType::Print)? {
      self.print_statement()
    } else {
      self.expression_statement()
    }
  }
  fn print_statement(&mut self) -> Result<(), CedarError> {
    self.expression()?;
    self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
    self.emit_byte(OpCode::Print, None)
  }
  fn expression_statement(&mut self) -> Result<(), CedarError> {
    self.expression()?;
    self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
    self.emit_byte(OpCode::Pop, None)
  }
  fn expression(&mut self) -> Result<(), CedarError> {
    self.parse_precedence(Precedence::Assignment)
  }
  fn grouping(&mut self, _: bool) -> Result<(), CedarError> {
    self.expression()?;
    self.consume(TokenType::RightParen, "Expect ')' after expression")
  }
  fn unary(&mut self, _: bool) -> Result<(), CedarError> {
    let ty = self
      .previous
      .as_ref()
      .map(|p| p.ty)
      .ok_or_else(|| CompilerError::ice("No previous value in unary expression"))?;
    self.parse_precedence(Precedence::Unary)?;
    match ty {
      TokenType::Minus => self.emit_byte(OpCode::Negate, None),
      TokenType::Bang => self.emit_byte(OpCode::Not, None),
      _ => unreachable!(),
    }
  }
  fn binary(&mut self, _: bool) -> Result<(), CedarError> {
    let operator_ty = self
      .previous
      .as_ref()
      .map(|p| p.ty)
      .ok_or_else(|| CompilerError::ice("No previous value in binary expression"))?;
    let rule = self.get_rule(&operator_ty);
    let precedence = match rule.precedence {
      Precedence::None => Precedence::Assignment,
      Precedence::Assignment => Precedence::Or,
      Precedence::Or => Precedence::And,
      Precedence::And => Precedence::Equality,
      Precedence::Equality => Precedence::Comparison,
      Precedence::Comparison => Precedence::Term,
      Precedence::Term => Precedence::Factor,
      Precedence::Factor => Precedence::Unary,
      Precedence::Unary => Precedence::Call,
      Precedence::Call => Precedence::Primary,
      Precedence::Primary => Precedence::Primary,
    };
    self.parse_precedence(precedence)?;
    match operator_ty {
      TokenType::Plus => self.emit_byte(OpCode::Add, None),
      TokenType::Minus => self.emit_byte(OpCode::Subtract, None),
      TokenType::Star => self.emit_byte(OpCode::Multiply, None),
      TokenType::Slash => self.emit_byte(OpCode::Divide, None),
      TokenType::BangEqual => self.emit_byte(OpCode::NotEqual, None),
      TokenType::EqualEqual => self.emit_byte(OpCode::Equal, None),
      TokenType::Greater => self.emit_byte(OpCode::Greater, None),
      TokenType::GreaterEqual => self.emit_byte(OpCode::GreaterOrEqual, None),
      TokenType::Less => self.emit_byte(OpCode::Less, None),
      TokenType::LessEqual => self.emit_byte(OpCode::LessOrEqual, None),
      _ => unreachable!(),
    }
  }
}

// Do not under any circumstances change this ordering as the derive is based on
// the order of the variants, much like a C style enum is just bigger numbers
// for each variant.
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
enum Precedence {
  None,
  Assignment, // =
  Or,         // or
  And,        // and
  Equality,   // == !=
  Comparison, // < > <= >=
  Term,       // + -
  Factor,     // * /
  Unary,      // ! -
  Call,       // . ()
  Primary,
}

struct ParseRule {
  prefix: ParseFn,
  infix: ParseFn,
  precedence: Precedence,
}

impl ParseRule {
  fn new(prefix: ParseFn, infix: ParseFn, precedence: Precedence) -> Self {
    Self {
      prefix,
      infix,
      precedence,
    }
  }
}

type ParseFn = Option<fn(&mut TokenIter, bool) -> Result<(), CedarError>>;

#[derive(Debug)]
pub enum CompilerError {
  Message { message: Cow<'static, str> },
  Failed,
}

impl CompilerError {
  fn new<M>(token: &Token, message: M) -> Self
  where
    M: Into<Cow<'static, str>>,
  {
    CompilerError::Message {
      message: {
        let message = message.into();
        if token.ty == TokenType::EOF {
          format!("[line {}] Error at end: {}", token.line, message).into()
        } else {
          format!(
            "[line {}] Error at '{}': {}",
            token.line, token.lexeme, message
          )
          .into()
        }
      },
    }
  }
  fn failed() -> Self {
    CompilerError::Failed
  }
  fn ice<M>(message: M) -> Self
  where
    M: Into<Cow<'static, str>>,
  {
    CompilerError::Message {
      message: format!("[ICE] Error: {}", message.into()).into(),
    }
  }
}

impl fmt::Display for CompilerError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      CompilerError::Message { message } => write!(f, "{}", message),
      CompilerError::Failed => write!(f, ""),
    }
  }
}

impl std::error::Error for CompilerError {}
