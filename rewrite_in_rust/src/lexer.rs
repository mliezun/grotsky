#![allow(dead_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Token {
    EOF,
    Newline,
    // Single-character tokens.
    // (, ), [, ], {, } ',', ., -, +, ;, /, %, *, ^, :, ;
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    RightCurlyBrace,
    LeftCurlyBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Slash,
    Mod,
    Star,
    Power,
    Colon,
    Semicolon,
    // One or two character tokens.
    // !=, =, ==, >, >=, <, <=
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    // *variable*, string, int,
    Identifier,
    String,
    Number,
    // Keywords.
    // and, class, else, false, fn, for, if, elif, nil, or,
    // return, break, continue, super, this, true, let, while, not, in, begin, end
    And,
    Class,
    Else,
    False,
    Fn,
    For,
    If,
    Elif,
    Nil,
    Or,
    Return,
    Break,
    Continue,
    Super,
    This,
    True,
    Let,
    While,
    Not,
    In,
    Try,
    Catch,
}

#[derive(Debug)]
enum Literal {
    String(String),
    Number(f64),
}

#[derive(Debug)]
struct TokenData {
    token: Token,
    lexeme: String,
    literal: Option<Literal>,
    line: i32,
}

struct InterpreterError {
    message: String,
    line: i32,
    pos: usize,
}

struct InterpreterState {
    source: String,
    tokens: Vec<TokenData>,
    errors: Vec<InterpreterError>,
    stmts: Vec<Stmt>,
}

impl InterpreterState {
    fn new(source: String) -> Self {
        InterpreterState {
            source: source,
            tokens: vec![],
            errors: vec![],
            stmts: vec![],
        }
    }

    fn set_error(&mut self, err: InterpreterError) {
        self.errors.push(err);
    }

    fn fatal_error(&mut self, err: InterpreterError) {
        self.errors.push(err);
        panic!();
    }
}

struct Lexer<'a> {
    start: usize,
    current: usize,
    line: i32,

    state: &'a mut InterpreterState,
}

fn keywords() -> HashMap<&'static str, Token> {
    HashMap::from([
        ("and", Token::And),
        ("class", Token::Class),
        ("else", Token::Else),
        ("false", Token::False),
        ("fn", Token::Fn),
        ("for", Token::For),
        ("if", Token::If),
        ("elif", Token::Elif),
        ("nil", Token::Nil),
        ("or", Token::Or),
        ("return", Token::Return),
        ("break", Token::Break),
        ("continue", Token::Continue),
        ("super", Token::Super),
        ("this", Token::This),
        ("true", Token::True),
        ("let", Token::Let),
        ("while", Token::While),
        ("not", Token::Not),
        ("in", Token::In),
        ("try", Token::Try),
        ("catch", Token::Catch),
    ])
}

impl Lexer<'_> {
    fn new(state: &'_ mut InterpreterState) -> Lexer<'_> {
        Lexer {
            start: 0,
            current: 0,
            line: 0,
            state: state,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.state.source.len()
    }

    fn next(&self) -> u8 {
        self.state.source.as_bytes()[self.current]
    }

    fn advance(&mut self) -> u8 {
        let c = self.next();
        self.current += 1;
        return c;
    }

    fn matches(&self, c: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        return char::from(self.next()) == c;
    }

    fn is_digit(&self, c: u8) -> bool {
        let zero: u8 = 48;
        let nine: u8 = 57;
        c >= zero && c <= nine
    }

    fn is_alpha(&self, c: u8) -> bool {
        let a: u8 = 97;
        let z: u8 = 122;
        let big_a: u8 = 65;
        let big_z: u8 = 90;
        let underscore: u8 = 95;
        (c >= a && c <= z) || (c >= big_a && c <= big_z) || c == underscore
    }

    fn emit(&mut self, token: Token, l: Option<Literal>) {
        let lexeme =
            String::from_utf8_lossy(&self.state.source.as_bytes()[self.start..self.current])
                .to_string();
        self.state.tokens.push(TokenData {
            token: token,
            lexeme: lexeme,
            literal: l,
            line: self.line,
        })
    }

    fn scan(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        let token_count = self.state.tokens.len();
        if token_count > 0
            && self.state.tokens.get(token_count - 1).unwrap().token != Token::Newline
        {
            self.state.tokens.push(TokenData {
                token: Token::Newline,
                lexeme: "".to_string(),
                literal: None,
                line: self.line,
            })
        }
        self.state.tokens.push(TokenData {
            token: Token::EOF,
            lexeme: "".to_string(),
            literal: None,
            line: self.line,
        })
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match char::from(c) {
            '[' => self.emit(Token::LeftBrace, None),
            ']' => self.emit(Token::RightBrace, None),
            '{' => self.emit(Token::LeftCurlyBrace, None),
            '}' => self.emit(Token::RightCurlyBrace, None),
            '(' => self.emit(Token::LeftParen, None),
            ')' => self.emit(Token::RightParen, None),
            ',' => self.emit(Token::Comma, None),
            '.' => self.emit(Token::Dot, None),
            '-' => self.emit(Token::Minus, None),
            '+' => self.emit(Token::Plus, None),
            '/' => self.emit(Token::Slash, None),
            '%' => self.emit(Token::Mod, None),
            '*' => self.emit(Token::Star, None),
            '^' => self.emit(Token::Power, None),
            ':' => self.emit(Token::Colon, None),
            ';' => self.emit(Token::Semicolon, None),
            '#' => {
                while !self.matches('\n') && !self.is_at_end() {
                    self.advance();
                }
            }
            '!' => {
                if self.matches('=') {
                    self.advance();
                    self.emit(Token::BangEqual, None);
                } else {
                    self.state.set_error(InterpreterError {
                        message: "'!' cannot be used here".to_string(),
                        line: self.line,
                        pos: self.start,
                    });
                }
            }
            '=' => {
                if self.matches('=') {
                    self.advance();
                    self.emit(Token::EqualEqual, None);
                } else {
                    self.emit(Token::Equal, None);
                }
            }
            '<' => {
                if self.matches('=') {
                    self.advance();
                    self.emit(Token::LessEqual, None);
                } else {
                    self.emit(Token::Less, None);
                }
            }
            '>' => {
                if self.matches('=') {
                    self.advance();
                    self.emit(Token::GreaterEqual, None);
                } else {
                    self.emit(Token::Greater, None);
                }
            }

            // Ignore whitespace
            ' ' => (),
            '\r' => (),
            '\t' => (),

            '\n' => {
                self.line += 1;
                self.emit(Token::Newline, None);
            }

            '"' => self.string(),

            _ => {
                if self.is_digit(c) {
                    self.number();
                } else if self.is_alpha(c) {
                    self.identifier();
                } else {
                    self.state.set_error(InterpreterError {
                        message: "Illegal character".to_string(),
                        line: self.line,
                        pos: self.start,
                    })
                }
            }
        }
    }

    fn string(&mut self) {
        let mut lit = "".to_string();
        while !self.is_at_end() && !self.matches('"') {
            if self.matches('\n') {
                self.line += 1;
            }
            if self.matches('\\') {
                self.advance();
                let unescaped = self.unescape_sequence();
                if unescaped == "".to_string() {
                    self.state.set_error(InterpreterError {
                        message: "Closing \" was expected".to_string(),
                        line: self.line,
                        pos: self.start,
                    });
                    return;
                }
                lit.push_str(&unescaped);
                continue;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.state.set_error(InterpreterError {
                message: "Closing \" was expected".to_string(),
                line: self.line,
                pos: self.start,
            });
        }

        // Consume ending "
        self.advance();

        self.emit(Token::String, Some(Literal::String(lit)));
    }

    fn unescape_sequence(&mut self) -> String {
        if self.is_at_end() {
            return "".to_string();
        }
        let c = self.advance();
        match char::from(c) {
            // 'a' => "\a".to_string(),
            // 'b' => "\b".to_string(),
            // 'f' => "\f".to_string(),
            // 'v' => "\v".to_string(),
            'n' => "\n".to_string(),
            'r' => "\r".to_string(),
            't' => "\t".to_string(),
            '\\' => "\\".to_string(),
            '"' => "\"".to_string(),
            _ => "".to_string(),
        }
    }

    fn number(&mut self) {
        while !self.is_at_end() && self.is_digit(self.next()) {
            self.advance();
        }

        if self.matches('.') {
            self.advance();
            while self.is_digit(self.next()) {
                self.advance();
            }
        }

        let lit = String::from_utf8_lossy(&self.state.source.as_bytes()[self.start..self.current])
            .parse::<f64>()
            .unwrap();

        self.emit(Token::Number, Some(Literal::Number(lit)));
    }

    fn identifier(&mut self) {
        while !self.is_at_end() && self.is_alpha(self.next()) {
            self.advance();
        }

        let identifier =
            String::from_utf8_lossy(&self.state.source.as_bytes()[self.start..self.current])
                .to_string();

        let kwds: HashMap<&str, Token> = keywords();
        let token = match kwds.get(identifier.as_str()) {
            Some(tok) => tok,
            None => &Token::Identifier,
        };

        self.emit(*token, None);
    }
}

struct FnExpr {
    params: Vec<TokenData>,
    body:   Vec<Stmt>,
}

struct VarExpr {
    name: Option<TokenData>,
}

enum Expr {
    Fn(FnExpr),
    Var(VarExpr),
}

struct CallStack {
    function: String,
    loop_count: usize,
}

struct Parser<'a> {
    current: usize,
    cls: Vec<CallStack>,
    state: &'a mut InterpreterState,
}

const max_function_params: usize = 255;

struct LetStmt { 
    name:        TokenData,
    initializer: Option<Expr>,
}


struct FnStmt {
    name: TokenData,
	params: Vec<TokenData>,
	body: Vec<Stmt>,
}

struct ClassStmt {
    name:          Option<TokenData>,
    methods:       Vec<FnStmt>,
    static_methods: Vec<FnStmt>,
    superclass:    Option<VarExpr>,
}

struct BlockStmt {
    stmts: Vec<Stmt>,
}

enum Stmt {
    Fn(FnStmt),
    Let(LetStmt),
    Block(BlockStmt),
    Class(ClassStmt),
}

impl Parser<'_> {
    fn  get_parsing_context(&self) -> &CallStack {
        return &self.cls[self.cls.len()-1];
    }

    fn  enter_function(&mut self, name: String) {
        self.cls.push(CallStack{
            function:  name,
            loop_count: 0,
        });
    }

    fn peek(&self) -> TokenData {
        return self.state.tokens[self.current];
    }

    fn  is_at_end(&self) -> bool {
        return self.peek().token == Token::EOF
    }

    fn  leave_function(&mut self, name: String) {
        let pc = self.get_parsing_context();
        if pc.function != name {
            self.state.fatal_error(InterpreterError{
                message:"Max number of parameters is 255".to_string(),
                line:self.peek().line,
                pos:0,
            })
        }
        self.cls.pop();
    }

    fn  enter_loop(&mut self)  {
        let pc = self.get_parsing_context();
        pc.loop_count += 1;
    }

    fn  leave_loop(&mut self) {
        let pc = self.get_parsing_context();
        pc.loop_count -= 1;
    }

    fn  inside_loop(&mut self) -> bool {
        return self.get_parsing_context().loop_count != 0;
    }


    fn  parse(&mut self)  {
        self.cls = vec![];
        self.enter_function("".to_string());
        while !self.is_at_end() {
            // When multiple empty lines are encountered after a statement
            // the parser founds nil statements, we should avoid them to not
            // break the execution stage
            match self.parse_stmt() {
                None => (),
                Some(st) => self.state.stmts.push(st),
            }
        }
        self.leave_function("".to_string());
    }

    fn  parse_stmt(&mut self) -> Option<Stmt> {
        /* defer func(&mut self) -> {
            if r = recover(); r != nil {
                self.synchronize()
            }
        }() */
        return self.declaration(true);
    }

    fn  check(&mut self, token: Token) -> bool {
        let oldCurrent = self.current;
        while token != Token::Newline && !self.is_at_end() && self.peek().token == Token::Newline {
            self.current += 1;
        }
        let matchs = self.peek().token == token;
        if !matchs {
            self.current = oldCurrent
        }
        return matchs
    }

    fn  matches(&mut self, token: Token)-> bool {
        if self.check(token) {
            self.current += 1;
            return true;
        }
        return false
    }

    fn  matches_many(&mut self, tokens: Vec<Token>) -> bool {
        for token in tokens {
            if self.check(token) {
                self.current += 1;
                return true;
            }
        }
        return false;
    }

    fn  consume(&mut self,token: Token, error_message: String)-> Option<TokenData> {
        if self.check(token) {
            return Some(self.advance());
        }

        self.state.set_error(InterpreterError { message: error_message, line: self.peek().line, pos: 0 });
        return None;
    }

    fn  advance(&mut self) -> TokenData {
        if !self.is_at_end() {
            self.current += 1;
        }
        return self.previous();
    }

    fn  previous(&mut self) -> TokenData {
        let mut i = 1;
        while  i <= self.current {
            if self.state.tokens[self.current-i].token != Token::Newline {
                break
            }
            i -= 1;
        }
        return self.state.tokens[self.current-1];
    }

    fn  declaration(&mut self, expect_new_line: bool) -> Option<Stmt> {
        let s = if self.matches(Token::Class) {
            self.class()
        } else if self.matches(Token::Fn) {
            self.fn_stmt()
        } else if self.matches(Token::Let) {
             self.let_stmt()
        } else {
             self.statement()
        };
        if expect_new_line {
            self.consume(Token::Newline, "Expected new line".to_string());
        }
        return s;
    }

    fn  class(&mut self) -> Option<Stmt> {
        let name = self.consume(Token::Identifier, "Expected variable name".to_string());

        let superclass: Option<VarExpr> = if self.matches(Token::Less) {
            let class = self.consume(Token::Identifier, "Expected variable name".to_string());
            Some(VarExpr{
                name: class,
            })
        } else {
            None
        };

        self.consume(Token::LeftCurlyBrace, "Expected '{' at this position".to_string());

        let mut methods: Vec<FnStmt> = vec![];
        let mut static_methods: Vec<FnStmt> = vec![];
        while !self.check(Token::RightCurlyBrace) && !self.is_at_end() {
            if self.matches(Token::Class) {
                static_methods.push(self.fn_stmt());
            } else {
                methods.push(self.fn_stmt());
            }
        }

        self.consume(Token::RightCurlyBrace, "Expected '}' at this position".to_string());

        return Some(Stmt::Class(ClassStmt{
            name:          name,
            methods:       methods,
            static_methods: static_methods,
            superclass:    superclass,
        }));
    }

    fn  fn_stmt(&mut self) -> FnStmt {
        let name = self.consume(Token::Identifier, "Expected function name".to_string()).unwrap();

        self.enter_function(name.lexeme);

        self.consume(Token::LeftParen, "Expect '(' after function name".to_string());

        let params: Vec<TokenData> = vec![];
        if !self.check(Token::RightParen) {
            loop {
                if params.len() > max_function_params {
                    self.state.fatal_error(InterpreterError { message: "Max number of parameters is 255".to_string(), line:self.peek().line, pos:0});
                }
                params.push(self.consume(Token::Identifier, "Expected function parameter".to_string()).unwrap());
                if !self.matches(Token::Comma) {
                    break;
                }
            }
        }
        self.consume(Token::RightParen, "Expect ')' after expression".to_string());

        let body: Vec<Stmt> = vec![];
        if self.matches(Token::LeftCurlyBrace) {
            body.append(self.block());
        } else {
            body.push(self.expression_stmt());
        }

        self.leave_function(name.lexeme);

        return FnStmt{
            name:   name,
            params: params,
            body:   body,
        }
    }

    fn  fn_expr(&mut self) -> FnExpr {
        self.consume(Token::LeftParen, "Expected '(' after function name".to_string());

        let lambda_name = format!("lambda{}", self.cls.len());
        self.enter_function(lambda_name);

        let params: Vec<TokenData> = vec![];
        if !self.check(Token::RightParen) {
            loop {
                if params.len() > max_function_params {
                    self.state.fatal_error(InterpreterError { message: "Max number of parameters is 255".to_string(), line:self.peek().line, pos:0 });
                }
                params.push(self.consume(Token::Identifier, "Expect function parameter".to_string()).unwrap());
                if !self.matches(Token::Comma) {
                    break
                }
            }
        }
        self.consume(Token::RightParen, "Expect ')' after expression".to_string());

        let body: Vec<Stmt> = vec![];
        if self.matches(Token::LeftCurlyBrace) {
            body.append(self.block());
        } else {
            body.append(self.expression_stmt());
        }

        self.leave_function(lambda_name);

        return FnExpr { params:params, body:body };
    }

    fn  let_stmt(&mut self) -> Stmt {
        let name = self.consume(Token::Identifier, "Expected variable name".to_string()).unwrap();

        let init: Option<Expr> = if self.matches(Token::Equal) {
            Some(self.expression())
        } else {
            None
        };

        return Stmt::Let(LetStmt{name:name, initializer:init});
    }

    fn  statement(&mut self) -> Stmt {
        if self.matches(Token::For) {
            return self.for_loop();
        }
        if self.matches(Token::Try) {
            return self.try_catch();
        }
        if self.matches(Token::If) {
            return self.if_stmt();
        }
        if self.matches(Token::Return) {
            return self.ret();
        }
        if self.matches(Token::Break) {
            return self.brk();
        }
        if self.matches(Token::Continue) {
            return self.cont();
        }
        if self.matches(Token::While) {
            return self.while_stmt();
        }
        if self.matches(Token::LeftCurlyBrace) {
            return BlockStmt{stmts: self.block()};
        }
        return self.expression_stmt();
    }

    fn  for_loop(&mut self) -> Stmt {
        let keyword = self.previous();

        self.enter_loop();

        if self.check(Token::Identifier) {
            // Enhanced for
            return self.enhancedFor(keyword)
        }
        // Classic for
        let init =  if self.matches(Token::Semicolon) {
            init = nil;
        } else if self.matches(Token::Let) {
            init = self.let_stmt();
            self.consume(Token::Semicolon, errExpectedSemicolon)
        } else {
            self.state.setError(errExpectedInit, self.peek().line, 0);
            None
        };

        let cond = self.expression();
        self.consume(Token::Semicolon, errExpectedSemicolon);

        let inc = self.expression();

        let body = self.declaration(false);

        self.leave_loop();

        return &classicForStmt[T]{
            keyword:     keyword,
            initializer: init,
            condition:   cond,
            increment:   inc,
            body:        body,
        }
    }

    fn  enhancedFor(keyword *token) stmt[T] {
        var ids []*token
        for self.matches(Token::Identifier) {
            ids = append(ids, self.previous())
            self.matches(Token::Comma)
        }
        self.consume(Token::In, errExpectedIn)
        collection = self.expression()
        body = self.declaration(false)
        return &enhancedForStmt[T]{
            keyword:     keyword,
            identifiers: ids,
            body:        body,
            collection:  collection,
        }
    }

    fn  try_catch(&mut self) -> stmt[T] {
        tryBody = self.declaration(false)
        self.consume(Token::Catch, errExpectedCatch)
        name = self.consume(Token::Identifier, errExpectedIdentifier)
        catchBody = self.declaration(false)

        return &try_catchStmt[T]{
            tryBody:   tryBody,
            name:      name,
            catchBody: catchBody,
        }
    }

    fn  if_stmt(&mut self) -> stmt[T] {
        st = &if_stmt[T]{
            keyword: self.previous(),
        }

        st.condition = self.expression()

        self.consume(Token::LeftCurlyBrace, errExpectedOpeningCurlyBrace)

        for !self.matches(Token::RightCurlyBrace) {
            st.thenBranch = append(st.thenBranch, self.declaration(false))
        }

        for self.matches(Token::Elif) {
            elif = &struct {
                condition  expr[T]
                thenBranch []stmt[T]
            }{
                condition: self.expression(),
            }
            self.consume(Token::LeftCurlyBrace, errExpectedOpeningCurlyBrace)
            for !self.matches(Token::RightCurlyBrace) {
                elif.thenBranch = append(elif.thenBranch, self.declaration(false))
            }
            st.elifs = append(st.elifs, elif)
        }

        if self.matches(Token::Else) {
            self.consume(Token::LeftCurlyBrace, errExpectedOpeningCurlyBrace)
            for !self.check(Token::RightCurlyBrace) {
                st.elseBranch = append(st.elseBranch, self.declaration(false))
            }
            self.consume(Token::RightCurlyBrace, errExpectedClosingCurlyBrace)
        }

        return st
    }

    fn  ret(&mut self) -> stmt[T] {
        var value expr[T]
        keyword = self.previous()
        if !self.check(Token::Newline) {
            value = self.expression()
        }
        return &returnStmt[T]{
            keyword: keyword,
            value:   value,
        }
    }

    fn  brk(&mut self) -> stmt[T] {
        keyword = self.previous()
        if !self.inside_loop(&mut self) -> {
            self.state.setError(errOnlyAllowedInsideLoop, keyword.line, 0)
        }
        return &breakStmt[T]{
            keyword: keyword,
        }
    }

    fn  cont(&mut self) -> stmt[T] {
        keyword = self.previous()
        if !self.inside_loop(&mut self) -> {
            self.state.setError(errOnlyAllowedInsideLoop, keyword.line, 0)
        }
        return &continueStmt[T]{
            keyword: keyword,
        }
    }

    fn  while_stmt(&mut self) -> stmt[T] {
        keyword = self.previous()
        self.enter_loop()
        defer self.leave_loop()
        cond = self.expression()
        body = self.declaration(false)
        return &whileStmt[T]{
            keyword:   keyword,
            condition: cond,
            body:      body,
        }
    }

    fn  block(&mut self) -> Vec<Stmt> {
        let stmts: Vec<Stmt> = vec![];
        while !self.matches(Token::RightCurlyBrace) {
            stmts.push(self.declaration(false));
        }
        return stmts;
    }

    fn  expression_stmt(&mut self) -> Stmt {
        let expr = self.expression();
        if expr != nil {
            return &exprStmt[T]{
                last:       self.previous(),
                expression: expr,
            }
        }
        // expr is nil when there are multiple empty lines
        return nil
    }

    fn  expression(&mut self) -> Expr] {
        return self.assignment();
    }

    fn  list(&mut self) -> expr[T] {
        elements = self.arguments(Token::RightBrace)
        brace = self.consume(Token::RightBrace, errUnclosedBracket)
        return &listExpr[T]{
            elements: elements,
            brace:    brace,
        }
    }

    fn  dictionary(&mut self) -> expr[T] {
        elements = self.dictElements()
        curlyBrace = self.consume(Token::RightCurlyBrace, errUnclosedCurlyBrace)
        return &dictionaryExpr[T]{
            elements:   elements,
            curlyBrace: curlyBrace,
        }
    }

    // dictElements returns array of keys & values where keys
    // are stored in even positions and values in odd positions
    fn  dictElements(&mut self) -> []expr[T] {
        elements = make([]expr[T], 0)
        for !self.check(Token::RightCurlyBrace) {
            key = self.expression()
            self.consume(Token::Colon, errExpectedColon)
            value = self.expression()
            elements = append(elements, key, value)
            if !self.matches(Token::Comma) {
                break
            }
        }
        return elements
    }

    fn  assignment(&mut self) -> expr[T] {
        expr = self.or()
        if self.matches(Token::Equal) {
            equal = self.previous()
            value = self.assignment()

            access, isAccess = expr.(*accessExpr[T])
            if isAccess {
                object = access.object
                for {
                    _, ok = object.(*accessExpr[T])
                    if !ok {
                        break
                    }
                    object = object.(*accessExpr[T]).object
                }
                expr = object
            }

            if variable, isVar = expr.(*variableExpr[T]); isVar {
                assign = &assignExpr[T]{
                    name:  variable.name,
                    value: value,
                }
                if access != nil {
                    assign.access = access
                }
                return assign
            } else if get, isGet = expr.(*getExpr[T]); isGet {
                set = &setExpr[T]{
                    name:   get.name,
                    object: get.object,
                    value:  value,
                }
                if access != nil {
                    set.access = access
                }
                return set
            }

            self.state.fatalError(errUndefinedStmt, equal.line, 0)
        }
        return expr
    }

    fn  access(object expr[T]) expr[T] {
        slice = &accessExpr[T]{
            object: object,
            brace:  self.previous(),
        }
        self.slice(slice)
        self.consume(Token::RightBrace, errors.New("Expected ']' at the end of slice"))
        return slice
    }

    fn  slice(slice *accessExpr[T]) {
        if self.matches(Token::Colon) {
            slice.firstColon = self.previous()
            if self.matches(Token::Colon) {
                slice.secondColon = self.previous()
                slice.third = self.expression()
            } else {
                slice.second = self.expression()
                if self.matches(Token::Colon) {
                    slice.secondColon = self.previous()
                    slice.third = self.expression()
                }
            }
        } else {
            slice.first = self.expression()
            if self.matches(Token::Colon) {
                slice.firstColon = self.previous()
                if self.matches(Token::Colon) {
                    slice.secondColon = self.previous()
                    slice.third = self.expression()
                } else if !self.check(Token::RightBrace) && !self.is_at_end(&mut self) -> {
                    slice.second = self.expression()
                    if self.matches(Token::Colon) {
                        slice.secondColon = self.previous()
                        if !self.check(Token::RightBrace) && !self.is_at_end(&mut self) -> {
                            slice.third = self.expression()
                        }
                    }
                }
            }
        }
    }

    fn  or(&mut self) -> expr[T] {
        expr = self.and()
        for self.matches(Token::Or) {
            operator = self.previous()
            right = self.and()
            expr = &logicalExpr[T]{
                left:     expr,
                operator: operator,
                right:    right,
            }
        }
        return expr
    }

    fn  and(&mut self) -> expr[T] {
        expr = self.equality()
        for self.matches(Token::And) {
            operator = self.previous()
            right = self.equality()
            expr = &logicalExpr[T]{
                left:     expr,
                operator: operator,
                right:    right,
            }
        }
        return expr
    }

    fn  equality(&mut self) -> expr[T] {
        expr = self.comparison()
        for self.matches(Token::EqualEqual, Token::BangEqual) {
            operator = self.previous()
            right = self.comparison()
            expr = &binaryExpr[T]{
                left:     expr,
                operator: operator,
                right:    right,
            }
        }
        return expr
    }

    fn  comparison(&mut self) -> expr[T] {
        expr = self.addition()
        for self.matches(Token::Greater, Token::GreaterEqual, Token::Less, Token::LessEqual) {
            operator = self.previous()
            right = self.addition()
            expr = &binaryExpr[T]{
                left:     expr,
                operator: operator,
                right:    right,
            }
        }
        return expr
    }

    fn  addition(&mut self) -> expr[T] {
        expr = self.multiplication()
        for self.matches(Token::Plus, Token::Minus) {
            operator = self.previous()
            right = self.multiplication()
            expr = &binaryExpr[T]{
                left:     expr,
                operator: operator,
                right:    right,
            }
        }
        return expr
    }

    fn  multiplication(&mut self) -> expr[T] {
        expr = self.power()
        for self.matches(Token::Slash, Token::Mod, Token::Star) {
            operator = self.previous()
            right = self.power()
            expr = &binaryExpr[T]{
                left:     expr,
                operator: operator,
                right:    right,
            }
        }
        return expr
    }

    fn  power(&mut self) -> expr[T] {
        expr = self.unary()
        for self.matches(Token::Power) {
            operator = self.previous()
            right = self.unary()
            expr = &binaryExpr[T]{
                left:     expr,
                operator: operator,
                right:    right,
            }
        }
        return expr
    }

    fn  unary(&mut self) -> expr[T] {
        if self.matches(Token::Not, Token::Minus) {
            operator = self.previous()
            right = self.unary()
            return &unaryExpr[T]{
                operator: operator,
                right:    right,
            }
        }
        return self.call()
    }

    fn  call(&mut self) -> expr[T] {
        expr = self.primary()
        for {
            if self.matches(Token::LeftParen) {
                expr = self.finishCall(expr)
            } else if self.matches(Token::Dot) {
                name = self.consume(Token::Identifier, errExpectedProp)
                expr = &getExpr[T]{
                    object: expr,
                    name:   name,
                }
            } else if self.matches(Token::LeftBrace) {
                expr = self.access(expr)
            } else {
                break
            }
        }
        return expr
    }

    fn  finishCall(callee expr[T]) expr[T] {
        arguments = self.arguments(Token::RightParen)
        paren = self.consume(Token::RightParen, errors.New("Expect ')' after arguments"))
        return &callExpr[T]{
            callee:    callee,
            arguments: arguments,
            paren:     paren,
        }
    }

    fn  arguments(Token:: tokenType) []expr[T] {
        arguments = make([]expr[T], 0)
        if !self.check(Token::) {
            for {
                if Token:: == Token::RightParen && len(arguments) >= max_function_params {
                    self.state.fatalError(errMaxArguments, self.peek().line, 0)
                }
                arguments = append(arguments, self.expression())
                if !self.matches(Token::Comma) || self.check(Token::) {
                    break
                }
            }
        }
        return arguments
    }

    fn  primary(&mut self) -> expr[T] {
        if self.matches(Token::Number, Token::String) {
            return &literalExpr[T]{value: self.previous().literal}
        }
        if self.matches(Token::False) {
            return &literalExpr[T]{value: grotskyBool(false)}
        }
        if self.matches(Token::True) {
            return &literalExpr[T]{value: grotskyBool(true)}
        }
        if self.matches(Token::Nil) {
            return &literalExpr[T]{value: nil}
        }
        if self.matches(Token::Identifier) {
            return &variableExpr[T]{name: self.previous()}
        }
        if self.matches(Token::LeftParen) {
            expr = self.expression()
            self.consume(Token::RightParen, errUnclosedParen)
            return &groupingExpr[T]{expression: expr}
        }
        if self.matches(Token::LeftBrace) {
            return self.list()
        }
        if self.matches(Token::LeftCurlyBrace) {
            return self.dictionary()
        }
        if self.matches(Token::Fn) {
            return self.fn_expr()
        }
        if self.matches(Token::This) {
            return &thisExpr[T]{keyword: self.previous()}
        }
        if self.matches(Token::Super) {
            return self.superExpr()
        }
        if self.matches(Token::Newline) {
            return nil
        }

        self.state.fatalError(errUndefinedExpr, self.peek().line, 0)
        return &literalExpr[T]{}
    }

    fn  superExpr(&mut self) -> expr[T] {
        super = &superExpr[T]{
            keyword: self.previous(),
        }
        if !self.check(Token::LeftParen) {
            self.consume(Token::Dot, errExpectedDot)
            super.method = self.consume(Token::Identifier, errExpectedIdentifier)
        } else {
            super.method = &token{
                token:  Token::Identifier,
                lexeme: "init",
                line:   super.keyword.line,
            }
        }
        return super
    }


    

    


    

    fn  synchronize(&mut self) -> {
        self.advance()
        for !self.is_at_end(&mut self) -> {
            switch self.peek().token {
            case Token::Class:
                return
            case Token::Fn:
                return
            case Token::Let:
                return
            case Token::For:
                return
            case Token::If:
                return
            case Token::While:
                return
            case Token::Return:
                return
            default:
            }

            self.advance()
        }
    }
}

pub fn scan(source: String) {
    let state = &mut InterpreterState::new(source);
    let mut lex = Lexer::new(state);
    lex.scan();
    println!("{:#?}", state.tokens);
}
