use crate::CedarError;
use std::{borrow::Cow, fmt, str};

pub struct Scanner {
  start: usize,
  current: usize,
  line: usize,
  source: Vec<u8>,
}

impl Scanner {
  pub fn new(source: String) -> Self {
    Self {
      start: 0,
      current: 0,
      line: 1,
      // This makes it easier to index into
      source: source.into_bytes(),
    }
  }

  pub fn scan(&mut self) -> Result<Vec<Token>, CedarError> {
    let mut tokens = Vec::new();
    while {
      tokens.push(self.scan_token()?);
      !self.is_at_end()
    } {}
    Ok(tokens)
  }
  pub fn scan_token(&mut self) -> Result<Token, CedarError> {
    self.skip_whitespace();
    // Edge case where if you end the file in whitespace we
    // end up doing an out of bounds index in self.advance()
    // and so we check here before continuing
    if self.is_at_end() {
      return Ok(self.make_token(TokenType::EOF));
    }
    self.start = self.current;
    let c = self.advance() as char;
    match c {
      '(' => return Ok(self.make_token(TokenType::LeftParen)),
      ')' => return Ok(self.make_token(TokenType::RightParen)),
      '{' => return Ok(self.make_token(TokenType::LeftBrace)),
      '}' => return Ok(self.make_token(TokenType::RightBrace)),
      ';' => return Ok(self.make_token(TokenType::Semicolon)),
      ',' => return Ok(self.make_token(TokenType::Comma)),
      '.' => return Ok(self.make_token(TokenType::Dot)),
      '-' => return Ok(self.make_token(TokenType::Minus)),
      '+' => return Ok(self.make_token(TokenType::Plus)),
      '/' => return Ok(self.make_token(TokenType::Slash)),
      '*' => return Ok(self.make_token(TokenType::Star)),
      '!' => {
        return Ok(if self.match_char('=') {
          self.make_token(TokenType::BangEqual)
        } else {
          self.make_token(TokenType::Bang)
        })
      }
      '=' => {
        return Ok(if self.match_char('=') {
          self.make_token(TokenType::EqualEqual)
        } else {
          self.make_token(TokenType::Equal)
        })
      }
      '<' => {
        return Ok(if self.match_char('=') {
          self.make_token(TokenType::LessEqual)
        } else {
          self.make_token(TokenType::Less)
        })
      }
      '>' => {
        return Ok(if self.match_char('=') {
          self.make_token(TokenType::GreaterEqual)
        } else {
          self.make_token(TokenType::Greater)
        })
      }
      '"' => return self.string(),
      _ if c.is_ascii_digit() => return self.number(),
      _ if c.is_ascii_alphabetic() => return self.identifier(),
      _ => (),
    }
    if self.is_at_end() {
      Ok(self.make_token(TokenType::EOF))
    } else {
      Err(
        ScannerError::new(format!(
          "Unexpected character [line {}]: {:?}",
          self.line,
          self.peek() as char,
        ))
        .into(),
      )
    }
  }

  pub fn is_at_end(&self) -> bool {
    self.current == self.source.len() - 1
  }
  pub fn make_token(&self, ty: TokenType) -> Token {
    Token {
      ty,
      line: self.line,
      lexeme: {
        unsafe {
          // TODO: Make sure we never slice things at the wrong part of a string
          String::from_utf8_unchecked(self.source[self.start..self.current].into()).into()
        }
      },
    }
  }
  pub fn match_char(&mut self, input: char) -> bool {
    if self.is_at_end() || self.source[self.current] as char != input {
      false
    } else {
      self.current += 1;
      true
    }
  }
  pub fn advance(&mut self) -> char {
    self.current += 1;
    self.source[self.current - 1] as char
  }

  pub fn peek(&self) -> char {
    if self.is_at_end() {
      '\0'
    } else {
      self.source[self.current] as char
    }
  }

  pub fn peek_next(&self) -> char {
    if self.is_at_end() {
      '\0'
    } else {
      self.source[self.current + 1] as char
    }
  }

  pub fn skip_whitespace(&mut self) {
    loop {
      let c = self.peek();
      match c {
        ' ' | '\r' | '\t' => {
          self.current += 1;
        }
        '\n' => {
          self.line += 1;
          self.current += 1;
        }
        // Skip Comments
        '/' => {
          if self.peek_next() == '/' {
            while self.peek() != '\n' && !self.is_at_end() {
              self.current += 1;
            }
          } else {
            break;
          }
        }
        _ => break,
      }
    }
  }
  pub fn string(&mut self) -> Result<Token, CedarError> {
    while self.peek() != '"' && !self.is_at_end() {
      if self.peek() == '\n' {
        self.line += 1;
      }
      self.current += 1;
    }

    if self.is_at_end() {
      return Err(ScannerError::new("Unterminated string.").into());
    }
    self.current += 1;
    Ok(self.make_token(TokenType::String))
  }
  pub fn number(&mut self) -> Result<Token, CedarError> {
    while self.peek().is_ascii_digit() {
      self.current += 1;
    }
    if self.peek() == '.' && self.peek_next().is_ascii_digit() {
      self.current += 1;
      while self.peek().is_ascii_digit() {
        self.current += 1;
      }
    } else if self.peek() == '.' && !self.peek_next().is_ascii_digit() {
      return Err(
        ScannerError::new(format!(
          "On line {} a number ends with '.' which is invalid.",
          self.line
        ))
        .into(),
      );
    }
    Ok(self.make_token(TokenType::Number))
  }
  pub fn identifier(&mut self) -> Result<Token, CedarError> {
    while self.peek().is_ascii_alphanumeric() || self.peek() == '-' {
      self.current += 1;
      if self.peek() == '-' && !self.peek_next().is_ascii_alphanumeric() {
        return Err(
          ScannerError::new(format!(
            "On line {} an identifier ends with '-' which is invalid.",
            self.line
          ))
          .into(),
        );
      }
    }
    Ok({
      let ty = self.identifier_type();
      self.make_token(ty)
    })
  }
  pub fn identifier_type(&self) -> TokenType {
    match self.source[self.start] as char {
      'a' => self.check_keyword(1, 2, "nd", TokenType::And),
      'c' => self.check_keyword(1, 4, "lass", TokenType::Class),
      'e' => self.check_keyword(1, 3, "lse", TokenType::Else),
      'i' => self.check_keyword(1, 1, "f", TokenType::If),
      'n' => self.check_keyword(1, 3, "ull", TokenType::Null),
      'o' => self.check_keyword(1, 1, "r", TokenType::Or),
      'p' => self.check_keyword(1, 4, "rint", TokenType::Print),
      'r' => self.check_keyword(1, 5, "eturn", TokenType::Return),
      's' if self.current - self.start > 1 => match self.source[self.start + 1] as char {
        'u' => self.check_keyword(2, 3, "per", TokenType::Super),
        'e' => self.check_keyword(2, 2, "lf", TokenType::SelfTok),
        _ => TokenType::Identifier,
      },
      'l' => self.check_keyword(1, 2, "et", TokenType::Let),
      'w' => self.check_keyword(1, 4, "hile", TokenType::While),
      'f' if self.current - self.start > 1 => match self.source[self.start + 1] as char {
        'a' => self.check_keyword(2, 4, "alse", TokenType::False),
        'o' => self.check_keyword(2, 1, "r", TokenType::For),
        'n' => TokenType::Fn,
        _ => TokenType::Identifier,
      },
      't' => self.check_keyword(1, 3, "rue", TokenType::True),
      _ => TokenType::Identifier,
    }
  }
  pub fn check_keyword(&self, start: usize, length: usize, rest: &str, ty: TokenType) -> TokenType {
    let is_rest = || {
      let head = self.start + start;
      let tail = self.start + start + length;
      unsafe { str::from_utf8_unchecked(&self.source[head..tail]) == rest }
    };
    if self.current - self.start == start + length && is_rest() {
      ty
    } else {
      TokenType::Identifier
    }
  }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Token {
  pub ty: TokenType,
  pub line: usize,
  pub lexeme: Cow<'static, str>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum TokenType {
  // Single-character tokens.
  LeftParen,
  RightParen,
  LeftBrace,
  RightBrace,
  Comma,
  Dot,
  Minus,
  Plus,
  Semicolon,
  Slash,
  Star,

  // One or two character tokens.
  Bang,
  BangEqual,
  Equal,
  EqualEqual,
  Greater,
  GreaterEqual,
  Less,
  LessEqual,

  // Literals.
  Identifier,
  String,
  Number,

  // Keywords.
  And,
  Class,
  Else,
  False,
  Fn,
  For,
  If,
  Null,
  Or,
  Print,
  Return,
  Super,
  SelfTok,
  True,
  Let,
  While,

  EOF,
}

#[derive(Debug)]
pub struct ScannerError {
  message: Cow<'static, str>,
}

impl ScannerError {
  fn new<M>(message: M) -> Self
  where
    M: Into<Cow<'static, str>>,
  {
    Self {
      message: message.into(),
    }
  }
}

impl fmt::Display for ScannerError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.message)
  }
}

impl std::error::Error for ScannerError {}
