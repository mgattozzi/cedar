use crate::{
  chunk::{Chunk, OpCode},
  scanner::{Scanner, Token, TokenType},
  value::{Function, Value},
  CedarError,
};
use std::{borrow::Cow, fmt, iter::Peekable, mem, vec};

const U8_COUNT: isize = std::u8::MAX as isize + 1;

pub fn compile(source: String) -> Result<Function, CedarError> {
  let mut tokens = Scanner::new(source).scan()?;

  // This means we are parsing lines from the repl and need to add an EOF token
  if !tokens.iter().any(|token| token.ty == TokenType::EOF) {
    tokens.push(Token {
      ty: TokenType::EOF,
      line: 1,
      lexeme: "".into(),
    });
  }
  Ok(TokenIter::new(tokens).compile()?)
}

pub struct TokenIter {
  iter: Peekable<vec::IntoIter<Token>>,
  previous: Option<Token>,
  current: Option<Token>,
  function: Function,
  fn_type: FunctionType,
  rules: [ParseRule; 39],
  locals: Vec<Local>, // we use U8_COUNT as our hard limit for locals in scope
  scope_depth: isize,
}

impl fmt::Debug for TokenIter {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    self.chunk_immutable().disassemble("MAIN");
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
      function: Function::new(),
      fn_type: FunctionType::Script,
      locals: vec![Local::new(
        Token {
          ty: TokenType::Fn,
          line: 0,
          lexeme: "".into(),
        },
        0, // depth
      )],
      scope_depth: 0,
      rules: [
        // LeftParen
        ParseRule::new(
          Some(TokenIter::grouping),
          Some(TokenIter::call),
          Precedence::Call,
        ),
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
        ParseRule::new(None, Some(TokenIter::and_), Precedence::And),
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
        ParseRule::new(None, Some(TokenIter::or_), Precedence::Or),
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
  fn chunk_immutable(&self) -> &Chunk {
    &self.function.chunk
  }
  fn chunk(&mut self) -> &mut Chunk {
    &mut self.function.chunk
  }
  fn compile(mut self) -> Result<Function, CedarError> {
    let mut failed = false;
    self.advance();
    while !self.match_token(TokenType::EOF)? {
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
      Ok(self.function)
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
  fn match_token(&mut self, ty: TokenType) -> Result<bool, CedarError> {
    if !self.check(ty)? {
      Ok(false)
    } else {
      self.advance();
      Ok(true)
    }
  }
  fn check(&mut self, ty: TokenType) -> Result<bool, CedarError> {
    Ok(
      self
        .current
        .as_ref()
        .ok_or_else(|| CompilerError::ice("No current value while in check function"))?
        .ty
        == ty,
    )
  }
  fn synchronize(&mut self) -> Result<(), CedarError> {
    while !self.check(TokenType::EOF)? {
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
  fn parse_variable(&mut self) -> Result<Option<Value>, CedarError> {
    self.consume(TokenType::Identifier, "Expect variable name.")?;
    self.declare_variable()?;
    if self.scope_depth > 0 {
      // Locals are not looked up by name at runtime return None
      Ok(None)
    } else {
      // Globals are looked up by name at runtime return a value
      Ok(Some(Value::String(
        self
          .previous
          .clone()
          .ok_or_else(|| CompilerError::ice("No previous value in let_declaration"))?
          .lexeme,
      )))
    }
  }
  fn emit_byte(&mut self, byte: OpCode, value: Option<Value>) -> Result<(), CedarError> {
    let line = self
      .previous
      .as_ref()
      .map(|p| p.line)
      .ok_or_else(|| CompilerError::ice("No previous value when calling emit_byte"))?;
    self.chunk().write_chunk(byte, value, line)
  }
  fn emit_return(&mut self) -> Result<(), CedarError> {
    self.emit_byte(OpCode::Null, None)?;
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

    if can_assign && self.match_token(TokenType::Equal)? {
      Err(CompilerError::new(&self.previous.as_ref().unwrap(), "Expected infix function").into())
    } else {
      Ok(())
    }
  }
  fn get_rule(&self, ty: &TokenType) -> &ParseRule {
    &self.rules[ty.as_usize()]
  }
  fn declaration(&mut self) -> Result<(), CedarError> {
    if self.match_token(TokenType::Fn)? {
      self.fn_declaration()
    } else if self.match_token(TokenType::Let)? {
      self.let_declaration()
    } else {
      self.statement()
    }
  }
  fn fn_declaration(&mut self) -> Result<(), CedarError> {
    let global = self.parse_variable()?;
    self.mark_initialized();
    self.function(FunctionType::Function)?;
    self.define_variable(global)
  }
  fn let_declaration(&mut self) -> Result<(), CedarError> {
    let global = self.parse_variable()?;
    if self.match_token(TokenType::Equal)? {
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
  fn declare_variable(&mut self) -> Result<(), CedarError> {
    if self.scope_depth == 0 {
      // Global variables are implicitly declared
      return Ok(());
    }
    let name = self
      .previous
      .clone()
      .ok_or_else(|| CompilerError::ice("No previous value in declare_variable"))?;
    for local in self.locals.iter().rev() {
      let depth = match local.depth {
        Depth::Initialized(d) => d,
        Depth::Uninitialized => -1,
      };
      if depth != -1 && depth < self.scope_depth {
        break;
      }
      if name.lexeme == local.name.lexeme {
        return Err(
          CompilerError::new(&name, "Variable with this name already declared in scope").into(),
        );
      }
    }
    self.add_local(name)
  }
  fn define_variable(&mut self, global: Option<Value>) -> Result<(), CedarError> {
    if self.scope_depth > 0 {
      self.mark_initialized();
      return Ok(());
    }

    self.emit_byte(OpCode::DefineGlobal, global)
  }
  fn mark_initialized(&mut self) {
    if self.scope_depth != 0 {
      self.locals.iter_mut().last().unwrap().depth = Depth::Initialized(self.scope_depth);
    }
  }
  fn argument_list(&mut self) -> Result<u8, CedarError> {
    let mut arg_count = 0;
    if !self.check(TokenType::RightParen)? {
      while {
        self.expression()?;
        arg_count += 1;
        if arg_count == 255 {
          return Err(CompilerError::error("Cannot have more than 255 arguments").into());
        }
        self.match_token(TokenType::Comma)?
      } {}
    }
    self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
    Ok(arg_count)
  }
  fn and_(&mut self, _: bool) -> Result<(), CedarError> {
    let end_jump = self.emit_jump(OpCode::JumpIfFalse)?;
    self.emit_byte(OpCode::Pop, None)?;
    self.parse_precedence(Precedence::And)?;
    self.patch_jump(end_jump)
  }
  fn or_(&mut self, _: bool) -> Result<(), CedarError> {
    let else_jump = self.emit_jump(OpCode::JumpIfFalse)?;
    let end_jump = self.emit_jump(OpCode::Jump)?;
    self.patch_jump(else_jump)?;
    self.emit_byte(OpCode::Pop, None)?;
    self.parse_precedence(Precedence::Or)?;
    self.patch_jump(end_jump)
  }
  fn add_local(&mut self, name: Token) -> Result<(), CedarError> {
    if self.locals.len() == U8_COUNT as usize {
      Err(CompilerError::new(&name, "Too many local variables in function").into())
    } else {
      let local = Local::new(name, self.scope_depth);
      self.locals.push(local);
      Ok(())
    }
  }
  fn variable(&mut self, can_assign: bool) -> Result<(), CedarError> {
    self.named_variable(can_assign)
  }
  fn named_variable(&mut self, can_assign: bool) -> Result<(), CedarError> {
    let get_op;
    let set_op;
    let arg;
    let name = self.previous.clone().unwrap();
    let depth = self.resolve_local(&name.lexeme)?;
    match depth {
      Depth::Initialized(depth) => {
        get_op = OpCode::GetLocal;
        set_op = OpCode::SetLocal;
        if depth > U8_COUNT {
          return Err(CompilerError::new(&name, "Too many levels of scoping in function").into());
        }
        arg = Some(Value::Byte(depth as u8));
      }
      Depth::Uninitialized => {
        get_op = OpCode::GetGlobal;
        set_op = OpCode::SetGlobal;
        arg = Some(Value::String(name.lexeme));
      }
    }
    if can_assign && self.match_token(TokenType::Equal)? {
      self.expression()?;
      self.emit_byte(set_op, arg)
    } else {
      self.emit_byte(get_op, arg)
    }
  }
  fn resolve_local(&mut self, name: &str) -> Result<Depth, CedarError> {
    for (pos, local) in self.locals.iter().rev().enumerate() {
      if local.name.lexeme == name {
        return Ok(Depth::Initialized(
          self.locals.len() as isize - 1 - pos as isize,
        ));
      }
    }
    Ok(Depth::Uninitialized)
  }
  fn statement(&mut self) -> Result<(), CedarError> {
    if self.match_token(TokenType::Print)? {
      self.print_statement()
    } else if self.match_token(TokenType::If)? {
      self.if_statement()
    } else if self.match_token(TokenType::Return)? {
      self.return_statement()
    } else if self.match_token(TokenType::While)? {
      self.while_statement()
    } else if self.match_token(TokenType::For)? {
      self.for_statement()
    } else if self.match_token(TokenType::LeftBrace)? {
      self.begin_scope();
      self.block()?;
      self.end_scope()
    } else {
      self.expression_statement()
    }
  }
  fn print_statement(&mut self) -> Result<(), CedarError> {
    self.expression()?;
    self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
    self.emit_byte(OpCode::Print, None)
  }
  fn if_statement(&mut self) -> Result<(), CedarError> {
    self.expression()?;
    let then_jump = self.emit_jump(OpCode::JumpIfFalse)?;
    self.emit_byte(OpCode::Pop, None)?;
    self.statement()?;
    let else_jump = self.emit_jump(OpCode::Jump)?;
    self.patch_jump(then_jump)?;
    self.emit_byte(OpCode::Pop, None)?;
    if self.match_token(TokenType::Else)? {
      self.statement()?;
    }
    self.patch_jump(else_jump)
  }
  fn return_statement(&mut self) -> Result<(), CedarError> {
    if self.fn_type == FunctionType::Script {
      return Err(CompilerError::error("Cannot return from top level code").into());
    }
    if self.match_token(TokenType::Semicolon)? {
      self.emit_return()
    } else {
      self.expression()?;
      self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
      self.emit_byte(OpCode::Return, None)
    }
  }
  fn while_statement(&mut self) -> Result<(), CedarError> {
    let loop_start = self.chunk().code.len();
    self.expression()?;
    let exit_jump = self.emit_jump(OpCode::JumpIfFalse)?;
    self.emit_byte(OpCode::Pop, None)?;
    self.statement()?;
    self.emit_loop(loop_start)?;
    self.patch_jump(exit_jump)?;
    self.emit_byte(OpCode::Pop, None)
  }
  fn for_statement(&mut self) -> Result<(), CedarError> {
    self.begin_scope();
    if self.match_token(TokenType::Semicolon)? {
      // no initializer
    } else if self.match_token(TokenType::Let)? {
      self.let_declaration()?;
    } else {
      self.expression_statement()?;
    }

    let mut loop_start = self.chunk().code.len();
    let mut exit_jump = None;
    if !self.match_token(TokenType::Semicolon)? {
      self.expression()?;
      self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;
      exit_jump = Some(self.emit_jump(OpCode::JumpIfFalse)?);
      self.emit_byte(OpCode::Pop, None)?;
    }

    if !self.match_token(TokenType::LeftBrace)? {
      let body_jump = self.emit_jump(OpCode::Jump)?;
      let increment_start = self.chunk().code.len();
      self.expression()?;
      self.emit_byte(OpCode::Pop, None)?;
      self.emit_loop(loop_start)?;
      loop_start = increment_start;
      self.patch_jump(body_jump)?;
    }
    self.statement()?;
    self.emit_loop(loop_start)?;
    if let Some(exit_jump) = exit_jump {
      self.patch_jump(exit_jump)?;
      self.emit_byte(OpCode::Pop, None)?;
    }
    self.end_scope()
  }
  fn emit_loop(&mut self, loop_start: usize) -> Result<(), CedarError> {
    let chunk = self.chunk();
    chunk.write_byte(OpCode::Loop.into());
    let offset = chunk.code.len() - loop_start + 2;
    if offset > std::u16::MAX as usize {
      return Err(CompilerError::error("Loop body too large").into());
    }
    chunk.write_byte(((offset >> 8) & 0xff) as u8);
    chunk.write_byte((offset & 0xff) as u8);
    // For dissasembler
    chunk.lines.push(0xff);
    chunk.lines.push(0xff);
    chunk.lines.push(0xff);
    Ok(())
  }
  fn emit_jump(&mut self, jump: OpCode) -> Result<usize, CedarError> {
    let chunk = self.chunk();
    chunk.write_byte(jump.into());
    chunk.write_byte(0xff);
    chunk.write_byte(0xff);
    // For dissasembler
    chunk.lines.push(0xff);
    chunk.lines.push(0xff);
    chunk.lines.push(0xff);
    Ok(chunk.code.len() - 2)
  }
  fn patch_jump(&mut self, offset: usize) -> Result<(), CedarError> {
    let chunk = self.chunk();
    let jump = chunk.code.len() - offset - 2;

    if jump > std::u16::MAX as usize {
      return Err(CompilerError::error("Too much code to jump over.").into());
    }

    chunk.code[offset] = ((jump >> 8) & 0xff) as u8;
    chunk.code[offset + 1] = (jump & 0xff) as u8;
    Ok(())
  }
  fn expression_statement(&mut self) -> Result<(), CedarError> {
    self.expression()?;
    self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
    self.emit_byte(OpCode::Pop, None)
  }
  fn begin_scope(&mut self) {
    self.scope_depth += 1;
  }
  fn end_scope(&mut self) -> Result<(), CedarError> {
    self.scope_depth -= 1;

    let mut local_count = self.locals.len();
    while local_count > 0
      && match self.locals[local_count - 1].depth {
        Depth::Initialized(d) => d,
        // I don't really know what the best thing to do here is
        Depth::Uninitialized => -1,
      } > self.scope_depth
    {
      self.emit_byte(OpCode::Pop, None)?;
      self.locals.pop();
      local_count -= 1;
    }

    Ok(())
  }
  fn block(&mut self) -> Result<(), CedarError> {
    while !self.check(TokenType::RightBrace)? && !self.check(TokenType::EOF)? {
      self.declaration()?;
    }
    self.consume(TokenType::RightBrace, "Expect '}' after block.")
  }
  fn function(&mut self, ty: FunctionType) -> Result<(), CedarError> {
    let current_ty = self.fn_type;
    let scope_depth = self.scope_depth;
    self.scope_depth = 0;
    self.fn_type = ty;
    self.locals = Vec::new();
    let mut function = Function::new();
    let mut locals = Vec::new();
    mem::swap(&mut self.function, &mut function);
    mem::swap(&mut self.locals, &mut locals);

    if self.fn_type != FunctionType::Script {
      self.function.name = self
        .previous
        .as_ref()
        .expect("No previous value in function")
        .lexeme
        .clone();
    }

    self.begin_scope();
    self.consume(TokenType::LeftParen, "Expect '(' after function name")?;
    if !self.check(TokenType::RightParen)? {
      while {
        self.function.arity += 1;
        if self.function.arity > 255 {
          CompilerError::error("Cannot have more than 255 parameters");
        }
        let variable = self.parse_variable()?;
        self.define_variable(variable)?;
        self.match_token(TokenType::Comma)?
      } {}
    }
    self.consume(TokenType::RightParen, "Expect ')' after parameters")?;

    self.consume(TokenType::LeftBrace, "Expect '{' before function body")?;
    self.block()?;

    self.end_compiler()?;

    mem::swap(&mut self.function, &mut function);
    mem::swap(&mut self.locals, &mut locals);
    self.fn_type = current_ty;
    self.scope_depth = scope_depth;
    self.emit_byte(OpCode::Constant, Some(Value::Function(function)))
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
  fn call(&mut self, _: bool) -> Result<(), CedarError> {
    let arg_count = self.argument_list()?;
    self.emit_byte(OpCode::Call, Some(Value::Byte(arg_count)))
  }
}

#[derive(Debug)]
pub struct Local {
  name: Token,
  depth: Depth,
}

impl Local {
  fn new(name: Token, depth: isize) -> Self {
    Self {
      name,
      depth: Depth::Initialized(depth),
    }
  }
}

#[derive(Debug, PartialEq, Eq)]
enum Depth {
  Initialized(isize),
  Uninitialized,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FunctionType {
  Function,
  Script,
}

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
  fn error<M>(message: M) -> Self
  where
    M: Into<Cow<'static, str>>,
  {
    CompilerError::Message {
      message: format!("[error] Error: {}", message.into()).into(),
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
