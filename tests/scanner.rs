use cedar::{
  scanner::{Scanner, Token, TokenType},
  CedarError,
};

#[test]
/// Note that this test only checks that we get the right tokens not that we get
/// valid syntax
fn tokenize() -> Result<(), CedarError> {
  let file = std::fs::read_to_string("tests/cedar-scripts/scanner.cdr")?;
  let mut scanner = Scanner::new(file);
  let tokens = scanner.scan()?;
  let token_test = vec![
    Token {
      ty: TokenType::LeftBrace,
      line: 1,
      lexeme: "{".into(),
    },
    Token {
      ty: TokenType::RightBrace,
      line: 2,
      lexeme: "}".into(),
    },
    Token {
      ty: TokenType::LeftParen,
      line: 3,
      lexeme: "(".into(),
    },
    Token {
      ty: TokenType::RightParen,
      line: 4,
      lexeme: ")".into(),
    },
    Token {
      ty: TokenType::Comma,
      line: 5,
      lexeme: ",".into(),
    },
    Token {
      ty: TokenType::Dot,
      line: 6,
      lexeme: ".".into(),
    },
    Token {
      ty: TokenType::Minus,
      line: 7,
      lexeme: "-".into(),
    },
    Token {
      ty: TokenType::Plus,
      line: 8,
      lexeme: "+".into(),
    },
    Token {
      ty: TokenType::Semicolon,
      line: 9,
      lexeme: ";".into(),
    },
    Token {
      ty: TokenType::Slash,
      line: 10,
      lexeme: "/".into(),
    },
    Token {
      ty: TokenType::Star,
      line: 11,
      lexeme: "*".into(),
    },
    Token {
      ty: TokenType::Bang,
      line: 12,
      lexeme: "!".into(),
    },
    Token {
      ty: TokenType::BangEqual,
      line: 13,
      lexeme: "!=".into(),
    },
    Token {
      ty: TokenType::EqualEqual,
      line: 14,
      lexeme: "==".into(),
    },
    Token {
      ty: TokenType::GreaterEqual,
      line: 15,
      lexeme: ">=".into(),
    },
    Token {
      ty: TokenType::LessEqual,
      line: 16,
      lexeme: "<=".into(),
    },
    Token {
      ty: TokenType::Less,
      line: 17,
      lexeme: "<".into(),
    },
    Token {
      ty: TokenType::Greater,
      line: 18,
      lexeme: ">".into(),
    },
    Token {
      ty: TokenType::Identifier,
      line: 19,
      lexeme: "testing".into(),
    },
    Token {
      ty: TokenType::String,
      line: 20,
      lexeme: "\"Hello\"".into(),
    },
    Token {
      ty: TokenType::Number,
      line: 21,
      lexeme: "31.24".into(),
    },
    Token {
      ty: TokenType::Number,
      line: 22,
      lexeme: "415".into(),
    },
    Token {
      ty: TokenType::And,
      line: 23,
      lexeme: "and".into(),
    },
    Token {
      ty: TokenType::Class,
      line: 24,
      lexeme: "class".into(),
    },
    Token {
      ty: TokenType::Else,
      line: 25,
      lexeme: "else".into(),
    },
    Token {
      ty: TokenType::Identifier,
      line: 26,
      lexeme: "false".into(),
    },
    Token {
      ty: TokenType::Fn,
      line: 27,
      lexeme: "fn".into(),
    },
    Token {
      ty: TokenType::For,
      line: 28,
      lexeme: "for".into(),
    },
    Token {
      ty: TokenType::If,
      line: 29,
      lexeme: "if".into(),
    },
    Token {
      ty: TokenType::Null,
      line: 30,
      lexeme: "null".into(),
    },
    Token {
      ty: TokenType::Or,
      line: 31,
      lexeme: "or".into(),
    },
    Token {
      ty: TokenType::Print,
      line: 32,
      lexeme: "print".into(),
    },
    Token {
      ty: TokenType::Return,
      line: 33,
      lexeme: "return".into(),
    },
    Token {
      ty: TokenType::Super,
      line: 34,
      lexeme: "super".into(),
    },
    Token {
      ty: TokenType::SelfTok,
      line: 35,
      lexeme: "self".into(),
    },
    Token {
      ty: TokenType::True,
      line: 36,
      lexeme: "true".into(),
    },
    Token {
      ty: TokenType::Let,
      line: 37,
      lexeme: "let".into(),
    },
    Token {
      ty: TokenType::While,
      line: 38,
      lexeme: "while".into(),
    },
    Token {
      ty: TokenType::EOF,
      line: 40,
      lexeme: "while\n// Test comments and tabs on the next line\n    ".into(),
    },
  ];
  assert_eq!(tokens, token_test);
  Ok(())
}
