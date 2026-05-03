use crate::ast;
use crate::lexer;

pub struct Parser<'a> {
    tokens: &'a Vec<lexer::Token>,
    pos: usize,
}
impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<lexer::Token>) -> Self {
        Self { tokens, pos: 0 }
    }
    fn cur(&self) -> &lexer::Token {
        &self.tokens[self.pos]
    }
    fn peek(&self, offset: usize) -> &lexer::Token {
        &self.tokens[self.pos + offset]
    }
    fn match_ok(&mut self, t: lexer::TokenType) -> bool {
        self.cur().t == t
    }
    fn peek_match_ok(&mut self, t: lexer::TokenType) -> bool {
        self.peek(1).t == t
    }
    fn error(&self, l: usize, c: usize, msg: &String) {
        panic!("Error at {}, {}:\t{}", l, c, msg);
    }
    #[allow(dead_code)]
    fn expect(&mut self, t: lexer::TokenType) {
        if !self.match_ok(t.clone()) {
            self.error(
                self.cur().l,
                self.cur().c,
                &format!("Expected token `{:?}`, but found `{:?}`", t, self.cur().t),
            );
        }
    }
    fn advance(&mut self) {
        self.pos += 1;
    }
    fn consume(&mut self, t: lexer::TokenType) -> bool {
        if self.match_ok(t.clone()) {
            self.advance();
            true
        } else {
            self.error(
                self.cur().l,
                self.cur().c,
                &format!("Expected token `{:?}`, but found `{:?}`", t, self.cur().t),
            );
            false
        }
    }
    #[allow(dead_code)]
    fn consume_or(&mut self, t1: lexer::TokenType, t2: lexer::TokenType) -> bool {
        if self.match_ok(t1.clone()) {
            self.advance();
            true
        } else if self.match_ok(t2.clone()) {
            self.advance();
            true
        } else {
            self.error(
                self.cur().l,
                self.cur().c,
                &format!(
                    "Expected token `{:?}` or `{:?}`, but found `{:?}`",
                    t1,
                    t2,
                    self.cur().t
                ),
            );
            false
        }
    }

    /* experssions tree
     * expr ::= assign
     * assign ::= ident '=' assign | logical
     * logical ::= equality (('&&' | '||') equality)*
     * equality ::= relational (('==' | '!=') relational)*
     * relational ::= additive (('>' | '<' | '>=' | '<=') additive)*
     * additive ::= multiplicative (('+' | '-') multiplicative)*
     * multiplicative ::= expon (('*' | '/' | '%') expon)*
     * expon ::= factor ('^' factor)*
     * factor ::= func arg... (primary ([], (), . primary )) | primary
     * primary ::= number | ident | '(' expr ')' |
     */
    fn parse_expr(&mut self) -> ast::Expr {
        self.parse_logical()
    }

    #[allow(dead_code)]
    fn parse_assign(&mut self) -> ast::Expr {
        let mut node = self.parse_logical();
        if let ast::Expr::Ident(name) = &node {
            if self.match_ok(lexer::TokenType::Assign) {
                self.advance();
                node = ast::Expr::Assign(name.clone(), Box::new(self.parse_assign()));
            }
        }
        node
    }
    fn parse_logical(&mut self) -> ast::Expr {
        let mut node = self.parse_equality();
        while self.match_ok(lexer::TokenType::KWAnd) || self.match_ok(lexer::TokenType::KWOr) {
            let op = self.cur().v.clone();
            self.advance();
            node = ast::Expr::BinaryOp(Box::new(node), op, Box::new(self.parse_equality()));
        }
        node
    }
    fn parse_equality(&mut self) -> ast::Expr {
        let mut node = self.parse_relational();
        while self.match_ok(lexer::TokenType::OpEq) || self.match_ok(lexer::TokenType::OpNe) {
            let op = self.cur().v.clone();
            self.advance();
            node = ast::Expr::BinaryOp(Box::new(node), op, Box::new(self.parse_relational()));
        }
        node
    }
    fn parse_relational(&mut self) -> ast::Expr {
        let mut node = self.parse_additive();
        while self.match_ok(lexer::TokenType::OpGt)
            || self.match_ok(lexer::TokenType::OpLt)
            || self.match_ok(lexer::TokenType::OpGe)
            || self.match_ok(lexer::TokenType::OpLe)
        {
            let op = self.cur().v.clone();
            self.advance();
            node = ast::Expr::BinaryOp(Box::new(node), op, Box::new(self.parse_additive()));
        }
        node
    }
    fn parse_additive(&mut self) -> ast::Expr {
        let mut node = self.parse_multi();
        while self.match_ok(lexer::TokenType::OpAdd) || self.match_ok(lexer::TokenType::OpSub) {
            let op = self.cur().v.clone();
            self.advance();
            node = ast::Expr::BinaryOp(Box::new(node), op, Box::new(self.parse_multi()));
        }
        node
    }
    fn parse_multi(&mut self) -> ast::Expr {
        let mut node = self.parse_expon();
        while self.match_ok(lexer::TokenType::OpMul)
            || self.match_ok(lexer::TokenType::OpDiv)
            || self.match_ok(lexer::TokenType::OpMod)
        {
            let op = self.cur().v.clone();
            self.advance();
            node = ast::Expr::BinaryOp(Box::new(node), op, Box::new(self.parse_expon()));
        }
        node
    }
    fn parse_expon(&mut self) -> ast::Expr {
        let mut node = self.parse_factor();
        if self.match_ok(lexer::TokenType::OpPow) {
            let op = self.cur().v.clone();
            self.advance();
            node = ast::Expr::BinaryOp(Box::new(node), op, Box::new(self.parse_expon()));
        }
        node
    }
    fn parse_factor(&mut self) -> ast::Expr {
        if self.match_ok(lexer::TokenType::Ident) {
            let name = self.cur().v.clone();
            self.advance();
            if self.match_ok(lexer::TokenType::LParen) {
                // 带括号的函数调用
                self.advance();
                let mut args = Vec::new();
                while self.pos < self.tokens.len() && !self.match_ok(lexer::TokenType::RParen) {
                    args.push(self.parse_expr());
                    if self.match_ok(lexer::TokenType::RParen)
                        || self.match_ok(lexer::TokenType::EOF)
                        || self.pos >= self.tokens.len()
                        || self.match_ok(lexer::TokenType::SemiColon)
                    {
                        break;
                    }
                    self.consume(lexer::TokenType::Comma);
                }
                self.consume(lexer::TokenType::RParen);
                ast::Expr::Call(name, args)
            } else if self.is_primary_start() && !self.match_ok(lexer::TokenType::EOF) {
                // 无括号的函数调用
                let mut args = Vec::new();
                while self.is_primary_start() {
                    args.push(self.parse_primary());
                }
                ast::Expr::Call(name, args)
            } else {
                ast::Expr::Ident(name)
            }
        } else {
            let mut result = self.parse_primary();
            if self.match_ok(lexer::TokenType::LParen) {
                self.advance();
                let mut args = Vec::new();
                while self.pos < self.tokens.len()
                    && !self.match_ok(lexer::TokenType::RParen)
                    && self.match_ok(lexer::TokenType::EOF)
                    && self.match_ok(lexer::TokenType::SemiColon)
                {
                    args.push(self.parse_expr());
                    if self.match_ok(lexer::TokenType::RParen)
                        || self.match_ok(lexer::TokenType::EOF)
                        || self.pos >= self.tokens.len()
                        || self.match_ok(lexer::TokenType::SemiColon)
                    {
                        break;
                    }
                    self.consume(lexer::TokenType::Comma);
                }
                self.consume(lexer::TokenType::RParen);
                result = ast::Expr::DynCall(Box::new(result), args);
            }
            result
        }
    }
    fn is_primary_start(&self) -> bool {
        self.tk_is_primary_start(&self.cur())
    }
    fn tk_is_primary_start(&self, tk: &lexer::Token) -> bool {
        tk.t == lexer::TokenType::Digit
            || tk.t == lexer::TokenType::Ident
            || tk.t == lexer::TokenType::LParen
            || tk.t == lexer::TokenType::LBrace
            || tk.t == lexer::TokenType::OpUnarySub
            || tk.t == lexer::TokenType::OpUnaryNot
    }
    fn parse_primary(&mut self) -> ast::Expr {
        if self.match_ok(lexer::TokenType::Digit) {
            let num = self.cur().v.clone();
            self.advance();
            if self.match_ok(lexer::TokenType::Dot) && self.peek_match_ok(lexer::TokenType::Digit) {
                let mut float_str = num.clone();
                float_str.push('.');
                self.advance();
                float_str.push_str(&self.cur().v);
                // println!("Parsed float: {}", float_str);
                self.advance();
                ast::Expr::Float(float_str)
            } else {
                ast::Expr::Number(num.parse().unwrap())
            }
        } else if (self.match_ok(lexer::TokenType::OpUnarySub)
            || self.match_ok(lexer::TokenType::OpUnaryNot))
            && self.peek_match_ok(lexer::TokenType::LParen)
        {
            let op = self.cur().v.clone();
            self.advance();
            let expr = self.parse_expr();
            self.consume(lexer::TokenType::RParen);
            ast::Expr::UnaryOp(op, Box::new(expr))
        } else if self.match_ok(lexer::TokenType::LParen) {
            self.advance();
            let expr = self.parse_expr();
            self.consume(lexer::TokenType::RParen);
            expr
        } else if self.match_ok(lexer::TokenType::Ident) {
            let name = self.cur().v.clone();
            self.advance();
            ast::Expr::Ident(name)
        } else if self.match_ok(lexer::TokenType::LBrace) {
            self.advance();
            let mut stmts = Vec::new();
            while self.pos < self.tokens.len() && !self.match_ok(lexer::TokenType::RBrace) {
                if self.match_ok(lexer::TokenType::EOF) {
                    break;
                }
                stmts.push(self.parse_stmt());
            }
            if let Some(expr) = stmts.last_mut() {
                if let ast::Stmt::Expr(e) = &expr.stmt {
                    *expr = ast::Node {
                        stmt: ast::Stmt::TailReturn(e.clone()),
                        l: expr.l,
                        c: expr.c,
                    };
                }
            }
            self.consume(lexer::TokenType::RBrace);
            ast::Expr::Block(stmts)
        } else if self.match_ok(lexer::TokenType::SemiColon) || self.match_ok(lexer::TokenType::EOF)
        {
            ast::Expr::Null
        } else if self.match_ok(lexer::TokenType::KWIf) {
            self.advance();
            self.parse_if_expr()
        } else if self.match_ok(lexer::TokenType::KWFn) {
            self.advance();
            self.parse_lambda_expr()
        } else if self.match_ok(lexer::TokenType::StringLiteral) {
            let str = self.cur().v.clone();
            self.advance();
            ast::Expr::String(str)
        } else {
            self.error(
                self.cur().l,
                self.cur().c,
                &format!("Unexpected token `{:?}`", self.cur().t),
            );
            ast::Expr::Null
        }
    }

    fn parse_if_expr(&mut self) -> ast::Expr {
        let cond = self.parse_expr();
        self.consume(lexer::TokenType::KWThen);
        let then = self.parse_expr();
        let else_branch = if self.match_ok(lexer::TokenType::KWElse) {
            self.advance();
            Some(Box::new(self.parse_expr()))
        } else {
            None
        };
        ast::Expr::If(Box::new(cond), Box::new(then), else_branch)
    }

    fn parse_lambda_expr(&mut self) -> ast::Expr {
        let mut params = Vec::new();
        while self.pos < self.tokens.len() && !self.match_ok(lexer::TokenType::Arrow) {
            if self.match_ok(lexer::TokenType::EOF) {
                break;
            }
            if self.match_ok(lexer::TokenType::Ident) {
                params.push(self.cur().v.clone());
                self.advance();
            }
            if self.match_ok(lexer::TokenType::Arrow) {
                break;
            }
        }
        self.consume(lexer::TokenType::Arrow);
        ast::Expr::Lambda(params, Box::new(self.parse_expr()))
    }
    pub fn parse(&mut self) -> ast::Program {
        let mut body = Vec::new();
        while self.cur().t != lexer::TokenType::EOF && self.pos < self.tokens.len() {
            body.push(self.parse_stmt());
            if self.cur().t == lexer::TokenType::EOF {
                break;
            }
            self.consume(lexer::TokenType::SemiColon);
        }
        ast::Program::new(body)
    }
    fn parse_stmt(&mut self) -> ast::Node {
        let line = self.cur().l;
        let col = self.cur().c;
        let stmt = if self.match_ok(lexer::TokenType::KWLet) {
            self.advance();
            if !self.match_ok(lexer::TokenType::Ident) {
                self.error(line, col, &"Expected identifier after 'let'".to_string());
                return ast::Node {
                    stmt: ast::Stmt::Expr(Box::new(ast::Expr::Null)),
                    l: line,
                    c: col,
                };
            }
            let name = self.cur().v.clone();
            let mut lhs = ast::Expr::Ident(name.clone());
            // let lhs_str = format!("{:?}", lhs);
            self.advance();
            if self.match_ok(lexer::TokenType::Ident) {
                let mut params = Vec::new();
                params.push(name.clone());
                params.push(self.cur().v.clone());
                self.advance();
                while self.match_ok(lexer::TokenType::Ident) {
                    params.push(self.cur().v.clone());
                    self.advance();
                    if self.match_ok(lexer::TokenType::Assign) {
                        break;
                    }
                }
                lhs = ast::Expr::IdentList(params);
            }
            let _ = self.consume(lexer::TokenType::Assign);
            let rhs = self.parse_expr();
            ast::Node {
                stmt: ast::Stmt::Let(Box::new(lhs), Box::new(rhs)),
                l: line,
                c: col,
            }
        } else {
            let expr = self.parse_expr();
            ast::Node {
                stmt: ast::Stmt::Expr(Box::new(expr)),
                l: line,
                c: col,
            }
        };
        stmt
    }
}
