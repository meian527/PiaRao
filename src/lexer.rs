use core::panic;

use phf::{phf_map};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TokenType {
	EOF, 
	OpAdd, OpSub, OpMul, OpDiv, OpMod,OpPow, 
	OpNot, OpGt, OpLt, OpEq, OpNe, OpGe, OpLe,
	Ident, LParen, RParen, Assign, Digit, SemiColon, Comma, Dot, LBrace, RBrace,
	KWLet, KWAnd, KWOr, KWIf, KWElse,KWFn, KWReturn, KWThen,
}
static KEYWORD_STR : phf::Map<&'static str, TokenType> = phf_map! {
	"let" => TokenType::KWLet,
	"and" => TokenType::KWAnd,
	"or" => TokenType::KWOr,
	"if" => TokenType::KWIf,
	"else" => TokenType::KWElse,
	"fn" => TokenType::KWFn,
	"return" => TokenType::KWReturn,
	"then" => TokenType::KWThen,
};
static TOKEN_TYPE_STR : phf::Map<u8, TokenType> = phf_map! {
	b'+' => TokenType::OpAdd, 
	b'-' => TokenType::OpSub,
	b'*' => TokenType::OpMul,
	b'/' => TokenType::OpDiv,
	b'%' => TokenType::OpMod,
	b'^' => TokenType::OpPow,
	b'=' => TokenType::Assign,
	b'(' => TokenType::LParen,
	b')' => TokenType::RParen,
	b';' => TokenType::SemiColon,
	b',' => TokenType::Comma,
	b'!' => TokenType::OpNot,
	b'>' => TokenType::OpGt,
	b'<' => TokenType::OpLt,
	b'.' => TokenType::Dot,
	b'{' => TokenType::LBrace,
	b'}' => TokenType::RBrace,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
	pub t: TokenType,
	pub v: String, 
	pub l: usize,
	pub c: usize
}
pub struct Lexer<'a> {
	len: usize,
	s: &'a [u8],
	pos: usize,
	line: usize,
	col: usize,
}
impl<'a> Lexer<'a> {
	pub fn new(src: &'a String) -> Self {
		Self {
			len: src.len(), s: src.chars().as_str().as_bytes(), pos: 0, line: 1, col: 1
		}
	}
	fn advance(&mut self) {
		self.pos += 1;
		if self.pos < self.len {
			if self.s[self.pos]== b'\n' {
				self.pos += 1;
				self.line += 1;
				self.col = 1;
			} else {
				self.col += 1;
			}
		}
	}
	pub fn next(&mut self) -> Token {
		let mut result = Token{t: TokenType::EOF, v: String::new(), l: self.line, c: self.col};
		if self.s[self.pos].is_ascii_whitespace() {
			while self.pos < self.len && self.s[self.pos].is_ascii_whitespace() {
				self.advance();
			}
		}
		if self.pos < self.len {
		match self.s[self.pos] {
			b'+' | b'-' | b'*' | b'/' | b'%' | b'^' | b'(' | b')' | b';' | b',' | b'.' => {
				result.v.push(self.s[self.pos] as char);
				result.t = TOKEN_TYPE_STR[&self.s[self.pos]].clone();
				self.advance();
			},
			b'!' | b'>' | b'<' | b'='  => {
				result.v.push(self.s[self.pos] as char);
				self.advance();
				if self.s[self.pos] == b'=' {
					result.v.push('=');
					self.advance();
					result.t = match result.v.as_bytes()[0] {
						b'=' => TokenType::OpEq,
						b'!' => TokenType::OpNe,
						b'>' => TokenType::OpGe,
						b'<' => TokenType::OpLe,
						_ => panic!("Unexpected operator '{}'", result.v)
					}
				} else {
					result.t = match result.v.as_bytes()[0] {
						b'!' => TokenType::OpNot,
						b'>' => TokenType::OpGt,
						b'<' => TokenType::OpLt,
						b'=' => TokenType::Assign,
						_ => panic!("Unexpected operator '{}'", result.v)
					}
				}
			},
			_ => {
				if self.s[self.pos].is_ascii_digit() {
					result.t = TokenType::Digit;
					while self.s[self.pos].is_ascii_digit() {
						result.v.push(self.s[self.pos] as char);
						self.advance();
					}
				} else if self.s[self.pos].is_ascii_alphabetic() || self.s[self.pos] == b'_' {
					while self.s[self.pos].is_ascii_alphanumeric() || self.s[self.pos] == b'_' || self.s[self.pos].is_ascii_digit() {
						result.v.push(self.s[self.pos] as char);
						self.advance();
					}
					result.t = KEYWORD_STR.get(&result.v.as_str()).cloned().unwrap_or(TokenType::Ident);
				}
				else {
					panic!("Unexpected character '{}' at line {}, column {}", self.s[self.pos] as char, self.line, self.col);
				}
			}
		}
		}
		result
	}

	pub fn tokenize(&mut self) -> Vec<Token> {
		let mut tokens = Vec::new();
		while self.pos < self.len {
			tokens.push(self.next());
		}
		tokens
	}
}