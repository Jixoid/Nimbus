/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;


use crate::ast::{QdlDocument, QdlNode, QdlValue};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
	Styles,
	Root,
	Node,
	BraceOpen,
	BraceClose,
	BracketOpen,
	BracketClose,
	Colon,
	Comma,
	At,
	Caret,
	StringLit(String),
	Number(f64),
	Ident(String),
	Eof,
}

pub struct Lexer<'a> {
	chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
	pub fn new(input: &'a str) -> Self {
		Self {
			chars: input.chars().peekable(),
		}
	}

	pub fn tokenize(mut self) -> Result<Vec<Token>, String> {
		let mut tokens = Vec::new();

		while let Some(&ch) = self.chars.peek() {
			if ch.is_whitespace() {
				self.chars.next();
				continue;
			}

			// Single line comment
			if ch == '/' {
				self.chars.next();
				if let Some(&next_ch) = self.chars.peek() {
					if next_ch == '/' {
						self.chars.next();
						while let Some(&c) = self.chars.peek() {
							self.chars.next();
							if c == '\n' {
								break;
							}
						}
						continue;
					} else if next_ch == '*' {
						self.chars.next();
						while let Some(c) = self.chars.next() {
							if c == '*' && self.chars.peek() == Some(&'/') {
								self.chars.next();
								break;
							}
						}
						continue;
					} else {
						return Err("Unexpected token '/'".to_string());
					}
				}
			}

			match ch {
				'{' => {
					self.chars.next();
					tokens.push(Token::BraceOpen);
				}
				'}' => {
					self.chars.next();
					tokens.push(Token::BraceClose);
				}
				'[' => {
					self.chars.next();
					tokens.push(Token::BracketOpen);
				}
				']' => {
					self.chars.next();
					tokens.push(Token::BracketClose);
				}
				':' => {
					self.chars.next();
					tokens.push(Token::Colon);
				}
				',' => {
					self.chars.next();
					tokens.push(Token::Comma);
				}
				'@' => {
					self.chars.next();
					tokens.push(Token::At);
				}
				'^' => {
					self.chars.next();
					tokens.push(Token::Caret);
				}
				'"' => {
					self.chars.next();
					let mut s = String::new();
					while let Some(c) = self.chars.next() {
						if c == '"' {
							break;
						} else if c == '\\' {
							if let Some(esc) = self.chars.next() {
								match esc {
									'n' => s.push('\n'),
									'r' => s.push('\r'),
									't' => s.push('\t'),
									'"' => s.push('"'),
									'\\' => s.push('\\'),
									other => s.push(other),
								}
							}
						} else {
							s.push(c);
						}
					}
					tokens.push(Token::StringLit(s));
				}
				_ if ch.is_ascii_digit() || (ch == '-' && self.peek_is_digit()) => {
					let mut s = String::new();
					s.push(self.chars.next().unwrap());
					while let Some(&c) = self.chars.peek() {
						if c.is_ascii_digit() || c == '.' {
							s.push(self.chars.next().unwrap());
						} else {
							break;
						}
					}
					let val: f64 = s
						.parse()
						.map_err(|e| format!("Invalid number '{s}': {e}"))?;
					tokens.push(Token::Number(val));
				}
				_ if ch.is_alphabetic() || ch == '_' => {
					let mut s = String::new();
					while let Some(&c) = self.chars.peek() {
						if c.is_alphanumeric() || c == '_' {
							s.push(self.chars.next().unwrap());
						} else if c == ':' {
							let mut clone = self.chars.clone();
							clone.next();
							if clone.peek() == Some(&':') {
								s.push(self.chars.next().unwrap());
								s.push(self.chars.next().unwrap());
							} else {
								break;
							}
						} else {
							break;
						}
					}

					let tok = match s.as_str() {
						"styles" => Token::Styles,
						"root" => Token::Root,
						"node" => Token::Node,
						_ => Token::Ident(s),
					};
					tokens.push(tok);
				}
				other => {
					return Err(format!("Unexpected character: '{other}'"));
				}
			}
		}

		tokens.push(Token::Eof);
		Ok(tokens)
	}

	fn peek_is_digit(&mut self) -> bool {
		let mut clone = self.chars.clone();
		clone.next(); // skip '-'
		clone.peek().map(|c| c.is_ascii_digit()).unwrap_or(false)
	}
}

pub struct Parser {
	tokens: Vec<Token>,
	pos: usize,
}

impl Parser {
	pub fn new(tokens: Vec<Token>) -> Self {
		Self { tokens, pos: 0 }
	}

	fn peek(&self) -> &Token {
		self.tokens.get(self.pos).unwrap_or(&Token::Eof)
	}

	fn next(&mut self) -> Token {
		let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
		if self.pos < self.tokens.len() {
			self.pos += 1;
		}
		tok
	}

	fn expect(&mut self, expected: Token) -> Result<(), String> {
		let tok = self.next();
		if tok == expected {
			Ok(())
		} else {
			Err(format!("Expected token {expected:?}, found {tok:?}"))
		}
	}

	pub fn parse_document(&mut self) -> Result<QdlDocument, String> {
		let mut doc = QdlDocument::default();

		while *self.peek() != Token::Eof {
			match self.peek() {
				Token::Styles => {
					self.next();
					self.expect(Token::BraceOpen)?;
					while *self.peek() != Token::BraceClose && *self.peek() != Token::Eof {
						let style_name = match self.next() {
							Token::Ident(s) | Token::StringLit(s) => s,
							other => return Err(format!("Expected style name, found {other:?}")),
						};
						self.expect(Token::BraceOpen)?;
						let props = self.parse_properties_until_brace()?;
						doc.styles.insert(style_name, props);
					}
					self.expect(Token::BraceClose)?;
				}
				Token::Root => {
					self.next();
					let node = self.parse_node_body(true)?;
					doc.root = Some(node);
				}
				other => return Err(format!("Unexpected top-level token: {other:?}")),
			}
		}

		Ok(doc)
	}

	fn parse_node_body(&mut self, _is_root: bool) -> Result<QdlNode, String> {
		let kind = match self.next() {
			Token::Ident(s) => s,
			other => return Err(format!("Expected node kind, found {other:?}")),
		};

		let name = if let Token::StringLit(_) = self.peek() {
			if let Token::StringLit(s) = self.next() {
				Some(s)
			} else {
				None
			}
		} else {
			None
		};

		self.expect(Token::BraceOpen)?;

		let mut styles = Vec::new();
		let mut props = HashMap::new();
		let mut children = Vec::new();

		while *self.peek() != Token::BraceClose && *self.peek() != Token::Eof {
			if *self.peek() == Token::Node {
				self.next();
				let child = self.parse_node_body(false)?;
				children.push(child);
				continue;
			}

			// Key-value property or style declaration
			let key = match self.next() {
				Token::Ident(k) => k,
				Token::StringLit(k) => k,
				other => return Err(format!("Expected property key or 'node', found {other:?}")),
			};

			if *self.peek() == Token::Colon {
				self.next();
				let val = self.parse_value()?;
				if key == "style" {
					if let Some(s) = val.as_str() {
						styles.push(s.to_string());
					}
				} else if key == "styles" {
					if let Some(arr) = val.as_array() {
						for item in arr {
							if let Some(s) = item.as_str() {
								styles.push(s.to_string());
							}
						}
					}
				} else {
					props.insert(key, val);
				}
			} else if *self.peek() == Token::BraceOpen {
				// inline block e.g. noise {}
				self.next();
				let sub_props = self.parse_properties_until_brace()?;
				props.insert(key, QdlValue::Object(sub_props));
			}

			if *self.peek() == Token::Comma {
				self.next();
			}
		}

		self.expect(Token::BraceClose)?;

		Ok(QdlNode {
			kind,
			name,
			styles,
			props,
			children,
		})
	}

	fn parse_properties_until_brace(&mut self) -> Result<HashMap<String, QdlValue>, String> {
		let mut props = HashMap::new();
		while *self.peek() != Token::BraceClose && *self.peek() != Token::Eof {
			let key = match self.next() {
				Token::Ident(s) | Token::StringLit(s) => s,
				other => return Err(format!("Expected property key, found {other:?}")),
			};

			if *self.peek() == Token::Colon {
				self.next();
				let val = self.parse_value()?;
				props.insert(key, val);
			}

			if *self.peek() == Token::Comma {
				self.next();
			}
		}
		self.expect(Token::BraceClose)?;
		Ok(props)
	}

	fn parse_value(&mut self) -> Result<QdlValue, String> {
		match self.next() {
			Token::StringLit(s) => Ok(QdlValue::String(s)),
			Token::Number(n) => Ok(QdlValue::Number(n)),
			Token::Ident(s) => match s.as_str() {
				"true" => Ok(QdlValue::Bool(true)),
				"false" => Ok(QdlValue::Bool(false)),
				_ => Ok(QdlValue::Ident(s)),
			},
			Token::BracketOpen => {
				let mut arr = Vec::new();
				while *self.peek() != Token::BracketClose && *self.peek() != Token::Eof {
					let val = self.parse_value()?;
					arr.push(val);
					if *self.peek() == Token::Comma {
						self.next();
					}
				}
				self.expect(Token::BracketClose)?;
				Ok(QdlValue::Array(arr))
			}
			Token::BraceOpen => {
				let obj = self.parse_properties_until_brace()?;
				Ok(QdlValue::Object(obj))
			}
			Token::At => {
				let mut anchor_str = "@".to_string();
				if *self.peek() == Token::Caret {
					self.next();
					anchor_str.push('^');
				} else if let Token::StringLit(s) = self.peek() {
					anchor_str.push_str(s);
					self.next();
				}
				if let Token::StringLit(dir) = self.peek() {
					anchor_str.push(' ');
					anchor_str.push_str(dir);
					self.next();
				}
				Ok(QdlValue::Ident(anchor_str))
			}
			other => Err(format!("Unexpected value token: {other:?}")),
		}
	}
}

pub fn parse(input: &str) -> Result<QdlDocument, String> {
	let lexer = Lexer::new(input);
	let tokens = lexer.tokenize()?;
	let mut parser = Parser::new(tokens);
	parser.parse_document()
}
