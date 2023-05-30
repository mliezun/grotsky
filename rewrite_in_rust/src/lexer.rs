#![allow(dead_code)]

use std::{collections::HashMap, vec};

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

#[derive(Debug, Clone, PartialEq)]
enum Literal {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
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
            lit.push(self.state.source.as_bytes()[self.current].into());
            self.advance();
        }

        if self.is_at_end() {
            self.state.set_error(InterpreterError {
                message: "Closing \" was expected".to_string(),
                line: self.line,
                pos: self.start,
            });
            return;
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

#[derive(Debug, PartialEq)]
struct FnExpr {
    params: Vec<TokenData>,
    body: Vec<Stmt>,
}

#[derive(Debug, PartialEq)]
struct VarExpr {
    name: Option<TokenData>,
}

#[derive(Debug, PartialEq)]
struct ListExpr {
    elements: Vec<Expr>,
    brace: TokenData,
}

#[derive(Debug, PartialEq)]
struct DictionaryExpr {
    elements: Vec<Expr>,
    curly_brace: TokenData,
}

#[derive(Debug, PartialEq)]
struct AssignExpr {
    name: TokenData,
    value: Box<Expr>,
}

#[derive(Debug, PartialEq)]
struct AccessExpr {
    object: Box<Expr>,
    brace: TokenData,
    first: Box<Expr>,
    first_colon: TokenData,
    second: Box<Expr>,
    second_colon: TokenData,
    third: Box<Expr>,
}

#[derive(Debug, PartialEq)]
struct BinaryExpr {
    left: Box<Expr>,
    operator: TokenData,
    right: Box<Expr>,
}

#[derive(Debug, PartialEq)]
struct CallExpr {
    callee: Box<Expr>,
    paren: TokenData,
    arguments: Vec<Expr>,
}

#[derive(Debug, PartialEq)]
struct GetExpr {
    object: Box<Expr>,
    name: TokenData,
}

#[derive(Debug, PartialEq)]
struct SetExpr {
    object: Box<Expr>,
    name: TokenData,
    value: Box<Expr>,
}

#[derive(Debug, PartialEq)]
struct SuperExpr {
    keyword: TokenData,
    method: TokenData,
}

#[derive(Debug, PartialEq)]
struct GroupingExpr {
    expression: Box<Expr>,
}

#[derive(Debug, PartialEq)]
struct LiteralExpr {
    value: Literal,
}

#[derive(Debug, PartialEq)]
struct LogicalExpr {
    left: Box<Expr>,
    operator: TokenData,
    right: Box<Expr>,
}

#[derive(Debug, PartialEq)]
struct ThisExpr {
    keyword: TokenData,
}

#[derive(Debug, PartialEq)]
struct UnaryExpr {
    operator: TokenData,
    right: Box<Expr>,
}

#[derive(Debug, PartialEq)]
enum Expr {
    Fn(FnExpr),
    Var(VarExpr),
    List(ListExpr),
    Dictionary(DictionaryExpr),
    Assign(AssignExpr),
    Access(AccessExpr),
    Binary(BinaryExpr),
    Call(CallExpr),
    Get(GetExpr),
    Set(SetExpr),
    Super(SuperExpr),
    Grouping(GroupingExpr),
    Literal(LiteralExpr),
    Logical(LogicalExpr),
    This(ThisExpr),
    Unary(UnaryExpr),
    Empty,
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

const MAX_FUNCTION_PARAMS: usize = 255;

#[derive(Debug, PartialEq)]
struct LetStmt {
    name: TokenData,
    initializer: Option<Expr>,
}

#[derive(Debug, PartialEq)]
struct ClassicForStmt {
    keyword: TokenData,
    initializer: Option<Box<Stmt>>,
    condition: Expr,
    increment: Expr,
    body: Box<Stmt>,
}

#[derive(Debug, PartialEq)]
struct EnhancedForStmt {
    keyword: TokenData,
    identifiers: Vec<TokenData>,
    collection: Expr,
    body: Box<Stmt>,
}

#[derive(Debug, PartialEq)]
struct TryCatchStmt {
    try_body: Box<Stmt>,
    name: TokenData,
    catch_body: Box<Stmt>,
}

#[derive(Debug, PartialEq)]
struct WhileStmt {
    keyword: TokenData,
    condition: Expr,
    body: Box<Stmt>,
}

#[derive(Debug, PartialEq)]
struct ReturnStmt {
    keyword: TokenData,
    value: Option<Expr>,
}

#[derive(Debug, PartialEq)]
struct BreakStmt {
    keyword: TokenData,
}

#[derive(Debug, PartialEq)]
struct ContinueStmt {
    keyword: TokenData,
}

#[derive(Debug, PartialEq)]
struct ElifBranch {
    condition: Expr,
    then_branch: Vec<Stmt>,
}

#[derive(Debug, PartialEq)]
struct IfStmt {
    keyword: TokenData,
    condition: Expr,
    then_branch: Vec<Stmt>,
    elifs: Vec<ElifBranch>,
    else_branch: Vec<Stmt>,
}

#[derive(Debug, PartialEq)]
struct FnStmt {
    name: TokenData,
    params: Vec<TokenData>,
    body: Vec<Stmt>,
}

#[derive(Debug, PartialEq)]
struct ClassStmt {
    name: Option<TokenData>,
    methods: Vec<FnStmt>,
    static_methods: Vec<FnStmt>,
    superclass: Option<VarExpr>,
}

#[derive(Debug, PartialEq)]
struct BlockStmt {
    stmts: Vec<Stmt>,
}

#[derive(Debug, PartialEq)]
struct ExprStmt {
    last: Option<TokenData>,
    expression: Expr,
}

#[derive(Debug, PartialEq)]
enum Stmt {
    Fn(FnStmt),
    Let(LetStmt),
    Block(BlockStmt),
    Class(ClassStmt),
    ClassicFor(ClassicForStmt),
    EnhancedFor(EnhancedForStmt),
    While(WhileStmt),
    If(IfStmt),
    Continue(ContinueStmt),
    Return(ReturnStmt),
    Break(BreakStmt),
    TryCatch(TryCatchStmt),
    Expr(ExprStmt),
}

impl Parser<'_> {
    fn new(state: &'_ mut InterpreterState) -> Parser<'_> {
        return Parser {
            current: 0,
            cls: vec![],
            state: state,
        };
    }

    fn get_parsing_context(&mut self) -> &mut CallStack {
        let len = self.cls.len();
        return &mut self.cls[len - 1];
    }

    fn enter_function(&mut self, name: &String) {
        self.cls.push(CallStack {
            function: name.to_string(),
            loop_count: 0,
        });
    }

    fn peek(&self) -> TokenData {
        return self.state.tokens[self.current].clone();
    }

    fn is_at_end(&self) -> bool {
        return self.peek().token == Token::EOF;
    }

    fn leave_function(&mut self, name: &String) {
        let pc = self.get_parsing_context();
        if pc.function != name.to_string() {
            self.state.fatal_error(InterpreterError {
                message: "Max number of parameters is 255".to_string(),
                line: self.peek().line,
                pos: 0,
            })
        }
        self.cls.pop();
    }

    fn enter_loop(&mut self) {
        let pc = self.get_parsing_context();
        pc.loop_count += 1;
    }

    fn leave_loop(&mut self) {
        let pc = self.get_parsing_context();
        pc.loop_count -= 1;
    }

    fn inside_loop(&mut self) -> bool {
        return self.get_parsing_context().loop_count != 0;
    }

    fn parse(&mut self) {
        self.cls = vec![];
        self.enter_function(&"".to_string());
        while !self.is_at_end() {
            // When multiple empty lines are encountered after a statement
            // the parser founds nil statements, we should avoid them to not
            // break the execution stage
            match self.parse_stmt() {
                None => (),
                Some(st) => self.state.stmts.push(st),
            }
        }
        self.leave_function(&"".to_string());
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        /* defer func(&mut self) -> {
            if r = recover(); r != nil {
                self.synchronize()
            }
        }() */
        return self.declaration(true);
    }

    fn check(&mut self, token: Token) -> bool {
        let old_current = self.current;
        while token != Token::Newline && !self.is_at_end() && self.peek().token == Token::Newline {
            self.current += 1;
        }
        let matchs = self.peek().token == token;
        if !matchs {
            self.current = old_current;
        }
        return matchs;
    }

    fn matches(&mut self, token: Token) -> bool {
        if self.check(token) {
            self.current += 1;
            return true;
        }
        return false;
    }

    fn matches_many(&mut self, tokens: Vec<Token>) -> bool {
        for token in tokens {
            if self.check(token) {
                self.current += 1;
                return true;
            }
        }
        return false;
    }

    fn consume(&mut self, token: Token, error_message: String) -> Option<TokenData> {
        if self.check(token) {
            return Some(self.advance());
        }

        self.state.set_error(InterpreterError {
            message: error_message,
            line: self.peek().line,
            pos: 0,
        });
        return None;
    }

    fn advance(&mut self) -> TokenData {
        if !self.is_at_end() {
            self.current += 1;
        }
        return self.previous();
    }

    fn previous(&mut self) -> TokenData {
        let mut i = 1;
        while i <= self.current {
            if self.state.tokens[self.current - i].token != Token::Newline {
                break;
            }
            i -= 1;
        }
        return self.state.tokens[self.current - 1].clone();
    }

    fn declaration(&mut self, expect_new_line: bool) -> Option<Stmt> {
        let s = if self.matches(Token::Class) {
            self.class()
        } else if self.matches(Token::Fn) {
            Some(self.fn_stmt())
        } else if self.matches(Token::Let) {
            Some(self.let_stmt())
        } else {
            Some(self.statement())
        };
        if expect_new_line {
            self.consume(Token::Newline, "Expected new line".to_string());
        }
        return s;
    }

    fn class(&mut self) -> Option<Stmt> {
        let name = self.consume(Token::Identifier, "Expected variable name".to_string());

        let superclass: Option<VarExpr> = if self.matches(Token::Less) {
            let class = self.consume(Token::Identifier, "Expected variable name".to_string());
            Some(VarExpr { name: class })
        } else {
            None
        };

        self.consume(
            Token::LeftCurlyBrace,
            "Expected '{' at this position".to_string(),
        );

        let mut methods: Vec<FnStmt> = vec![];
        let mut static_methods: Vec<FnStmt> = vec![];
        while !self.check(Token::RightCurlyBrace) && !self.is_at_end() {
            if self.matches(Token::Class) {
                if let Stmt::Fn(fn_stmt) = self.fn_stmt() {
                    static_methods.push(fn_stmt);
                } else {
                    panic!("Not a function");
                }
            } else {
                if let Stmt::Fn(fn_stmt) = self.fn_stmt() {
                    methods.push(fn_stmt);
                } else {
                    panic!("Not a function");
                }
            }
        }

        self.consume(
            Token::RightCurlyBrace,
            "Expected '}' at this position".to_string(),
        );

        return Some(Stmt::Class(ClassStmt {
            name: name,
            methods: methods,
            static_methods: static_methods,
            superclass: superclass,
        }));
    }

    fn fn_stmt(&mut self) -> Stmt {
        let name = self
            .consume(Token::Identifier, "Expected function name".to_string())
            .unwrap();

        self.enter_function(&name.lexeme);

        self.consume(
            Token::LeftParen,
            "Expect '(' after function name".to_string(),
        );

        let mut params: Vec<TokenData> = vec![];
        if !self.check(Token::RightParen) {
            loop {
                if params.len() > MAX_FUNCTION_PARAMS {
                    self.state.fatal_error(InterpreterError {
                        message: "Max number of parameters is 255".to_string(),
                        line: self.peek().line,
                        pos: 0,
                    });
                }
                params.push(
                    self.consume(Token::Identifier, "Expected function parameter".to_string())
                        .unwrap(),
                );
                if !self.matches(Token::Comma) {
                    break;
                }
            }
        }
        self.consume(Token::RightParen, "Expect ')' after expression".to_string());

        let mut body: Vec<Stmt> = vec![];
        if self.matches(Token::LeftCurlyBrace) {
            body.append(self.block().as_mut());
        } else {
            body.push(self.expression_stmt());
        }

        self.leave_function(&name.lexeme);

        return Stmt::Fn(FnStmt {
            name: name,
            params: params,
            body: body,
        });
    }

    fn fn_expr(&mut self) -> FnExpr {
        self.consume(
            Token::LeftParen,
            "Expected '(' after function name".to_string(),
        );

        let lambda_name: &String = &format!("lambda{}", self.cls.len());
        self.enter_function(lambda_name);

        let mut params: Vec<TokenData> = vec![];
        if !self.check(Token::RightParen) {
            loop {
                if params.len() > MAX_FUNCTION_PARAMS {
                    self.state.fatal_error(InterpreterError {
                        message: "Max number of parameters is 255".to_string(),
                        line: self.peek().line,
                        pos: 0,
                    });
                }
                params.push(
                    self.consume(Token::Identifier, "Expect function parameter".to_string())
                        .unwrap(),
                );
                if !self.matches(Token::Comma) {
                    break;
                }
            }
        }
        self.consume(Token::RightParen, "Expect ')' after expression".to_string());

        let mut body: Vec<Stmt> = vec![];
        if self.matches(Token::LeftCurlyBrace) {
            body.append(self.block().as_mut());
        } else {
            body.push(self.expression_stmt());
        }

        self.leave_function(lambda_name);

        return FnExpr {
            params: params,
            body: body,
        };
    }

    fn let_stmt(&mut self) -> Stmt {
        let name = self
            .consume(Token::Identifier, "Expected variable name".to_string())
            .unwrap();

        let init: Option<Expr> = if self.matches(Token::Equal) {
            Some(self.expression())
        } else {
            None
        };

        return Stmt::Let(LetStmt {
            name: name,
            initializer: init,
        });
    }

    fn statement(&mut self) -> Stmt {
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
            return Stmt::Block(BlockStmt {
                stmts: self.block(),
            });
        }
        return self.expression_stmt();
    }

    fn for_loop(&mut self) -> Stmt {
        let keyword = self.previous();

        self.enter_loop();

        if self.check(Token::Identifier) {
            // Enhanced for
            return self.enhanced_for(keyword);
        }
        // Classic for
        let init: Option<Box<Stmt>> = if self.matches(Token::Semicolon) {
            None
        } else if self.matches(Token::Let) {
            let aux = self.let_stmt();
            self.consume(Token::Semicolon, "Expected semicolon".to_string());
            Some(Box::new(aux))
        } else {
            self.state.set_error(InterpreterError {
                message: "Empty expression or let was expected at this position".to_string(),
                line: self.peek().line,
                pos: 0,
            });
            None
        };

        let cond = self.expression();
        self.consume(Token::Semicolon, "Expected semicolon".to_string());

        let inc = self.expression();

        let body = self.declaration(false);

        self.leave_loop();

        return Stmt::ClassicFor(ClassicForStmt {
            keyword: keyword,
            initializer: init,
            condition: cond,
            increment: inc,
            body: Box::new(body.unwrap()),
        });
    }

    fn enhanced_for(&mut self, keyword: TokenData) -> Stmt {
        let mut ids: Vec<TokenData> = vec![];
        while self.matches(Token::Identifier) {
            ids.push(self.previous());
            self.matches(Token::Comma);
        }
        self.consume(Token::In, "Expected 'in'".to_string());
        let collection: Expr = self.expression();
        let body = self.declaration(false).unwrap();
        return Stmt::EnhancedFor(EnhancedForStmt {
            keyword: keyword,
            identifiers: ids,
            body: Box::new(body),
            collection: collection,
        });
    }

    fn try_catch(&mut self) -> Stmt {
        let try_body = self.declaration(false).unwrap();
        self.consume(
            Token::Catch,
            "A catch block was expected at this position".to_string(),
        );
        let name = self
            .consume(Token::Identifier, "Expected variable name".to_string())
            .unwrap();
        let catch_body = self.declaration(false).unwrap();

        return Stmt::TryCatch(TryCatchStmt {
            try_body: Box::new(try_body),
            name: name,
            catch_body: Box::new(catch_body),
        });
    }

    fn if_stmt(&mut self) -> Stmt {
        let keyword = self.previous();
        let condition = self.expression();

        self.consume(
            Token::LeftCurlyBrace,
            "Expected '{' at this position".to_string(),
        );

        let mut then_branch: Vec<Stmt> = vec![];
        while !self.matches(Token::RightCurlyBrace) {
            then_branch.push(self.declaration(false).unwrap());
        }

        let mut elif_branches: Vec<ElifBranch> = vec![];
        while self.matches(Token::Elif) {
            let condition = self.expression();
            self.consume(
                Token::LeftCurlyBrace,
                "Expected '{' at this position".to_string(),
            );
            let mut then_branch: Vec<Stmt> = vec![];
            while !self.matches(Token::RightCurlyBrace) {
                then_branch.push(self.declaration(false).unwrap());
            }
            let elif = ElifBranch {
                condition: condition,
                then_branch: then_branch,
            };
            elif_branches.push(elif);
        }

        let mut else_branch: Vec<Stmt> = vec![];
        if self.matches(Token::Else) {
            self.consume(
                Token::LeftCurlyBrace,
                "Expected '{' at this position".to_string(),
            );
            while !self.check(Token::RightCurlyBrace) {
                else_branch.push(self.declaration(false).unwrap());
            }
            self.consume(
                Token::RightCurlyBrace,
                "Expected '}' at this position".to_string(),
            );
        }

        let st = IfStmt {
            keyword: keyword,
            condition: condition,
            then_branch: then_branch,
            elifs: elif_branches,
            else_branch: else_branch,
        };

        return Stmt::If(st);
    }

    fn ret(&mut self) -> Stmt {
        let keyword = self.previous();
        let value = if !self.check(Token::Newline) {
            Some(self.expression())
        } else {
            None
        };
        let rt = ReturnStmt {
            keyword: keyword,
            value: value,
        };
        return Stmt::Return(rt);
    }

    fn brk(&mut self) -> Stmt {
        let keyword = self.previous();
        if !self.inside_loop() {
            self.state.set_error(InterpreterError {
                message: "Statement only allowed for use inside loop".to_string(),
                line: keyword.line,
                pos: 0,
            });
        }
        let brk_stmt = BreakStmt { keyword };
        return Stmt::Break(brk_stmt);
    }

    fn cont(&mut self) -> Stmt {
        let keyword = self.previous();
        if !self.inside_loop() {
            self.state.set_error(InterpreterError {
                message: "Statement only allowed for use inside loop".to_string(),
                line: keyword.line,
                pos: 0,
            });
        }
        let continue_stmt = ContinueStmt { keyword };
        return Stmt::Continue(continue_stmt);
    }

    fn while_stmt(&mut self) -> Stmt {
        let keyword = self.previous();
        self.enter_loop();
        let condition = self.expression();
        let body = self.declaration(false).unwrap();
        self.leave_loop();
        let while_stmt = WhileStmt {
            keyword,
            condition,
            body: Box::new(body),
        };
        return Stmt::While(while_stmt);
    }

    fn block(&mut self) -> Vec<Stmt> {
        let mut stmts: Vec<Stmt> = vec![];
        while !self.matches(Token::RightCurlyBrace) {
            stmts.push(self.declaration(false).unwrap());
        }
        return stmts;
    }

    fn expression_stmt(&mut self) -> Stmt {
        let expr = self.expression();
        if expr != Expr::Empty {
            let expr_stmt = ExprStmt {
                last: Some(self.previous()),
                expression: expr,
            };
            return Stmt::Expr(expr_stmt);
        }
        // expr is empty when there are multiple empty lines
        let expr_stmt = ExprStmt {
            expression: Expr::Empty,
            last: None,
        };
        return Stmt::Expr(expr_stmt);
    }

    fn expression(&mut self) -> Expr {
        return self.assignment();
    }

    fn list(&mut self) -> Expr {
        let elements = self.arguments(Token::RightBrace);
        let brace = self
            .consume(Token::RightBrace, "Expected ']' at end of list".to_string())
            .unwrap();
        let list_expr = ListExpr { elements, brace };
        return Expr::List(list_expr);
    }

    fn dictionary(&mut self) -> Expr {
        let elements = self.dict_elements();
        let curly_brace = self
            .consume(
                Token::RightCurlyBrace,
                "Expected '}' at the end of dict".to_string(),
            )
            .unwrap();
        let dict_expr = DictionaryExpr {
            elements: elements,
            curly_brace: curly_brace,
        };
        return Expr::Dictionary(dict_expr);
    }

    // dict_elements returns array of keys & values where keys
    // are stored in even positions and values in odd positions
    fn dict_elements(&mut self) -> Vec<Expr> {
        let mut elements: Vec<Expr> = vec![];
        while !self.check(Token::RightCurlyBrace) {
            let key = self.expression();
            self.consume(Token::Colon, "Expected ':' after key".to_string());
            let value = self.expression();
            elements.push(key);
            elements.push(value);
            if !self.matches(Token::Comma) {
                break;
            }
        }
        return elements;
    }

    fn assignment(&mut self) -> Expr {
        let expr = self.or();
        if self.matches(Token::Equal) {
            let equal = self.previous();
            let value = self.assignment();

            // let mut object: Expr = Expr::Empty;
            // let access = match expr {
            //     Expr::Access(access) => {
            //         object = *access.object;
            //         loop {
            //             if let Expr::Access(inner_access) = object {
            //                 object = *inner_access.object;
            //             } else {
            //                 break;
            //             }
            //         }
            //         let access_expr = Box::new(Expr::Access(access));
            //         Some(access_expr)
            //     }
            //     _ => None,
            // };
            // if object != Expr::Empty {
            //     expr = object;
            // }

            match expr {
                Expr::Var(variable) => {
                    let assign = AssignExpr {
                        name: variable.name.unwrap(),
                        value: Box::new(value),
                    };
                    return Expr::Assign(assign);
                }
                Expr::Get(get) => {
                    let set = SetExpr {
                        name: get.name,
                        value: Box::new(value),
                        object: get.object,
                    };
                    return Expr::Set(set);
                }
                _ => self.state.fatal_error(InterpreterError {
                    message: "Undefined statement".to_string(),
                    line: equal.line,
                    pos: 0,
                }),
            }
        }
        return expr;
    }

    fn access(&mut self, object: Expr) -> Expr {
        let mut slice = AccessExpr {
            first: Box::new(Expr::Empty),
            first_colon: TokenData {
                token: Token::Nil,
                lexeme: "".to_string(),
                literal: None,
                line: 0,
            },
            object: Box::new(object),
            brace: self.previous(),
            second: Box::new(Expr::Empty),
            second_colon: TokenData {
                token: Token::Nil,
                lexeme: "".to_string(),
                literal: None,
                line: 0,
            },
            third: Box::new(Expr::Empty),
        };
        self.slice(&mut slice);
        self.consume(
            Token::RightBrace,
            "Expected ']' at the end of slice".to_string(),
        );
        return Expr::Access(slice);
    }

    fn slice(&mut self, slice: &mut AccessExpr) {
        if self.matches(Token::Colon) {
            slice.first_colon = self.previous();
            if self.matches(Token::Colon) {
                slice.second_colon = self.previous();
                slice.third = Box::new(self.expression());
            } else {
                slice.second = Box::new(self.expression());
                if self.matches(Token::Colon) {
                    slice.second_colon = self.previous();
                    slice.third = Box::new(self.expression());
                }
            }
        } else {
            slice.first = Box::new(self.expression());
            if self.matches(Token::Colon) {
                slice.first_colon = self.previous();
                if self.matches(Token::Colon) {
                    slice.second_colon = self.previous();
                    slice.third = Box::new(self.expression());
                } else if !self.check(Token::RightBrace) && !self.is_at_end() {
                    slice.second = Box::new(self.expression());
                    if self.matches(Token::Colon) {
                        slice.second_colon = self.previous();
                        if !self.check(Token::RightBrace) && !self.is_at_end() {
                            slice.third = Box::new(self.expression());
                        }
                    }
                }
            }
        }
    }

    fn or(&mut self) -> Expr {
        let mut expr = self.and();
        while self.matches(Token::Or) {
            let operator = self.previous();
            let right = self.and();
            let log_expr = LogicalExpr {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
            expr = Expr::Logical(log_expr);
        }
        return expr;
    }

    fn and(&mut self) -> Expr {
        let mut expr = self.equality();
        while self.matches(Token::And) {
            let operator = self.previous();
            let right = self.equality();
            let log_expr = LogicalExpr {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
            expr = Expr::Logical(log_expr);
        }
        return expr;
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        while self.matches_many(vec![Token::EqualEqual, Token::BangEqual]) {
            let operator = self.previous();
            let right = self.comparison();
            let bin_expr = BinaryExpr {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
            expr = Expr::Binary(bin_expr);
        }
        return expr;
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.addition();
        while self.matches_many(vec![
            Token::Greater,
            Token::GreaterEqual,
            Token::Less,
            Token::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.addition();
            let bin_expr = BinaryExpr {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
            expr = Expr::Binary(bin_expr);
        }
        return expr;
    }

    fn addition(&mut self) -> Expr {
        let mut expr = self.multiplication();
        while self.matches_many(vec![Token::Plus, Token::Minus]) {
            let operator = self.previous();
            let right = self.multiplication();
            let bin_expr = BinaryExpr {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
            expr = Expr::Binary(bin_expr);
        }
        return expr;
    }

    fn multiplication(&mut self) -> Expr {
        let mut expr = self.power();
        while self.matches_many(vec![Token::Slash, Token::Mod, Token::Star]) {
            let operator = self.previous();
            let right = self.power();
            let bin_expr = BinaryExpr {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
            expr = Expr::Binary(bin_expr);
        }
        return expr;
    }

    fn power(&mut self) -> Expr {
        let mut expr = self.unary();
        while self.matches(Token::Power) {
            let operator = self.previous();
            let right = self.unary();
            let bin_expr = BinaryExpr {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
            expr = Expr::Binary(bin_expr);
        }
        return expr;
    }

    fn unary(&mut self) -> Expr {
        if self.matches_many(vec![Token::Not, Token::Minus]) {
            let operator = self.previous();
            let right = self.unary();
            let unary_expr = UnaryExpr {
                operator: operator,
                right: Box::new(right),
            };
            return Expr::Unary(unary_expr);
        }
        return self.call();
    }

    fn call(&mut self) -> Expr {
        let mut expr = self.primary();
        loop {
            if self.matches(Token::LeftParen) {
                expr = self.finish_call(expr);
            } else if self.matches(Token::Dot) {
                let name = self
                    .consume(
                        Token::Identifier,
                        "Expected property name after '.'".to_string(),
                    )
                    .unwrap();
                let get_expr = GetExpr {
                    object: Box::new(expr),
                    name: name,
                };
                expr = Expr::Get(get_expr);
            } else if self.matches(Token::LeftBrace) {
                expr = self.access(expr);
            } else {
                break;
            }
        }
        return expr;
    }

    fn finish_call(&mut self, callee: Expr) -> Expr {
        let arguments = self.arguments(Token::RightParen);
        let paren = self
            .consume(Token::RightParen, "Expect ')' after arguments".to_string())
            .unwrap();
        let call_expr = CallExpr {
            callee: Box::new(callee),
            arguments: arguments,
            paren: paren,
        };
        return Expr::Call(call_expr);
    }

    fn arguments(&mut self, token_type: Token) -> Vec<Expr> {
        let mut arguments: Vec<Expr> = vec![];
        if !self.check(token_type) {
            loop {
                if token_type == Token::RightParen && arguments.len() >= MAX_FUNCTION_PARAMS {
                    self.state.fatal_error(InterpreterError {
                        message: "Max number of arguments is 255".to_string(),
                        line: self.peek().line,
                        pos: 0,
                    });
                }
                arguments.push(self.expression());
                if !self.matches(Token::Comma) || self.check(token_type) {
                    break;
                }
            }
        }
        return arguments;
    }

    fn primary(&mut self) -> Expr {
        if self.matches_many(vec![Token::Number, Token::String]) {
            let lit_expr = LiteralExpr {
                value: self.previous().literal.unwrap(),
            };
            return Expr::Literal(lit_expr);
        }
        if self.matches(Token::False) {
            let lit_expr = LiteralExpr {
                value: Literal::Boolean(false),
            };
            return Expr::Literal(lit_expr);
        }
        if self.matches(Token::True) {
            let lit_expr = LiteralExpr {
                value: Literal::Boolean(true),
            };
            return Expr::Literal(lit_expr);
        }
        if self.matches(Token::Nil) {
            let lit_expr = LiteralExpr {
                value: Literal::Nil,
            };
            return Expr::Literal(lit_expr);
        }
        if self.matches(Token::Identifier) {
            let var_expr = VarExpr {
                name: Some(self.previous()),
            };
            return Expr::Var(var_expr);
        }
        if self.matches(Token::LeftParen) {
            let expr = self.expression();
            self.consume(Token::RightParen, "Expect ')' after expression".to_string());
            let group_expr = GroupingExpr {
                expression: Box::new(expr),
            };
            return Expr::Grouping(group_expr);
        }
        if self.matches(Token::LeftBrace) {
            return self.list();
        }
        if self.matches(Token::LeftCurlyBrace) {
            return self.dictionary();
        }
        if self.matches(Token::Fn) {
            return Expr::Fn(self.fn_expr());
        }
        if self.matches(Token::This) {
            let this_expr = ThisExpr {
                keyword: self.previous(),
            };
            return Expr::This(this_expr);
        }
        if self.matches(Token::Super) {
            return self.super_expr();
        }
        if self.matches(Token::Newline) {
            return Expr::Empty;
        }

        self.state.fatal_error(InterpreterError {
            message: "Undefined expression".to_string(),
            line: self.peek().line,
            pos: 0,
        });
        return Expr::Empty;
    }

    fn super_expr(&mut self) -> Expr {
        let keyword = self.previous();
        let method: TokenData;
        if !self.check(Token::LeftParen) {
            self.consume(
                Token::Dot,
                "Keyword 'super' is only valid for property accessing".to_string(),
            );
            method = self
                .consume(Token::Identifier, "Expected variable name".to_string())
                .unwrap();
        } else {
            method = TokenData {
                token: Token::Identifier,
                lexeme: "init".to_string(),
                line: keyword.line,
                literal: None,
            };
        }
        let super_expr = SuperExpr {
            keyword: keyword,
            method: method,
        };
        return Expr::Super(super_expr);
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            match self.peek().token {
                Token::Class => return,
                Token::Fn => return,
                Token::Let => return,
                Token::For => return,
                Token::If => return,
                Token::While => return,
                Token::Return => return,
                _ => (),
            }
            self.advance();
        }
    }
}

#[derive(Debug)]
enum Result {
    Literal(Literal),
    Empty,
}

trait StmtVisitor {
    fn visit_expr_stmt(&self, stmt: &ExprStmt) -> Result;
    fn visit_try_catch_stmt(&self, stmt: &TryCatchStmt) -> Result;
    fn visit_classic_for_stmt(&self, stmt: &ClassicForStmt) -> Result;
    fn visit_enhanced_for_stmt(&self, stmt: &EnhancedForStmt) -> Result;
    fn visit_let_stmt(&self, stmt: &LetStmt) -> Result;
    fn visit_block_stmt(&self, stmt: &BlockStmt) -> Result;
    fn visit_while_stmt(&self, stmt: &WhileStmt) -> Result;
    fn visit_return_stmt(&self, stmt: &ReturnStmt) -> Result;
    fn visit_break_stmt(&self, stmt: &BreakStmt) -> Result;
    fn visit_continue_stmt(&self, stmt: &ContinueStmt) -> Result;
    fn visit_if_stmt(&self, stmt: &IfStmt) -> Result;
    fn visit_fn_stmt(&self, stmt: &FnStmt) -> Result;
    fn visit_class_stmt(&self, stmt: &ClassStmt) -> Result;
}

trait ExprVisitor {
    fn visit_function_expr(&self, expr: &FnExpr) -> Result;
    fn visit_variable_expr(&self, expr: &VarExpr) -> Result;
    fn visit_list_expr(&self, expr: &ListExpr) -> Result;
    fn visit_dictionary_expr(&self, expr: &DictionaryExpr) -> Result;
    fn visit_assign_expr(&self, expr: &AssignExpr) -> Result;
    fn visit_access_expr(&self, expr: &AccessExpr) -> Result;
    fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result;
    fn visit_call_expr(&self, expr: &CallExpr) -> Result;
    fn visit_get_expr(&self, expr: &GetExpr) -> Result;
    fn visit_set_expr(&self, expr: &SetExpr) -> Result;
    fn visit_super_expr(&self, expr: &SuperExpr) -> Result;
    fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result;
    fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result;
    fn visit_logical_expr(&self, expr: &LogicalExpr) -> Result;
    fn visit_this_expr(&self, expr: &ThisExpr) -> Result;
    fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result;
}

impl Expr {
    fn accept(&self, visitor: &dyn ExprVisitor) -> Result {
        match self {
            Expr::Fn(expr) => visitor.visit_function_expr(&expr),
            Expr::Var(expr) => visitor.visit_variable_expr(&expr),
            Expr::List(expr) => visitor.visit_list_expr(&expr),
            Expr::Dictionary(expr) => visitor.visit_dictionary_expr(&expr),
            Expr::Assign(expr) => visitor.visit_assign_expr(&expr),
            Expr::Access(expr) => visitor.visit_access_expr(&expr),
            Expr::Binary(expr) => visitor.visit_binary_expr(&expr),
            Expr::Call(expr) => visitor.visit_call_expr(&expr),
            Expr::Get(expr) => visitor.visit_get_expr(&expr),
            Expr::Set(expr) => visitor.visit_set_expr(&expr),
            Expr::Super(expr) => visitor.visit_super_expr(&expr),
            Expr::Grouping(expr) => visitor.visit_grouping_expr(&expr),
            Expr::Literal(expr) => visitor.visit_literal_expr(&expr),
            Expr::Logical(expr) => visitor.visit_logical_expr(&expr),
            Expr::This(expr) => visitor.visit_this_expr(&expr),
            Expr::Unary(expr) => visitor.visit_unary_expr(&expr),
            Expr::Empty => unreachable!(),
        }
    }
}

impl Stmt {
    fn accept(&self, visitor: &dyn StmtVisitor) -> Result {
        match self {
            Stmt::Fn(stmt) => visitor.visit_fn_stmt(&stmt),
            Stmt::Let(stmt) => visitor.visit_let_stmt(&stmt),
            Stmt::Block(stmt) => visitor.visit_block_stmt(&stmt),
            Stmt::Class(stmt) => visitor.visit_class_stmt(&stmt),
            Stmt::ClassicFor(stmt) => visitor.visit_classic_for_stmt(&stmt),
            Stmt::EnhancedFor(stmt) => visitor.visit_enhanced_for_stmt(&stmt),
            Stmt::While(stmt) => visitor.visit_while_stmt(&stmt),
            Stmt::If(stmt) => visitor.visit_if_stmt(&stmt),
            Stmt::Continue(stmt) => visitor.visit_continue_stmt(&stmt),
            Stmt::Return(stmt) => visitor.visit_return_stmt(&stmt),
            Stmt::Break(stmt) => visitor.visit_break_stmt(&stmt),
            Stmt::TryCatch(stmt) => visitor.visit_try_catch_stmt(&stmt),
            Stmt::Expr(stmt) => visitor.visit_expr_stmt(&stmt),
        }
    }
}

struct Exec<'a> {
    state: &'a InterpreterState,
}

impl StmtVisitor for Exec<'_> {
    fn visit_expr_stmt(&self, stmt: &ExprStmt) -> Result {
        stmt.expression.accept(self)
    }
    fn visit_try_catch_stmt(&self, stmt: &TryCatchStmt) -> Result {
        return Result::Empty;
    }
    fn visit_classic_for_stmt(&self, stmt: &ClassicForStmt) -> Result {
        return Result::Empty;
    }
    fn visit_enhanced_for_stmt(&self, stmt: &EnhancedForStmt) -> Result {
        return Result::Empty;
    }
    fn visit_let_stmt(&self, stmt: &LetStmt) -> Result {
        return Result::Empty;
    }
    fn visit_block_stmt(&self, stmt: &BlockStmt) -> Result {
        return Result::Empty;
    }
    fn visit_while_stmt(&self, stmt: &WhileStmt) -> Result {
        return Result::Empty;
    }
    fn visit_return_stmt(&self, stmt: &ReturnStmt) -> Result {
        return Result::Empty;
    }
    fn visit_break_stmt(&self, stmt: &BreakStmt) -> Result {
        return Result::Empty;
    }
    fn visit_continue_stmt(&self, stmt: &ContinueStmt) -> Result {
        return Result::Empty;
    }
    fn visit_if_stmt(&self, stmt: &IfStmt) -> Result {
        return Result::Empty;
    }
    fn visit_fn_stmt(&self, stmt: &FnStmt) -> Result {
        return Result::Empty;
    }
    fn visit_class_stmt(&self, stmt: &ClassStmt) -> Result {
        return Result::Empty;
    }
}

impl ExprVisitor for Exec<'_> {
    fn visit_function_expr(&self, expr: &FnExpr) -> Result {
        return Result::Empty;
    }
    fn visit_variable_expr(&self, expr: &VarExpr) -> Result {
        return Result::Empty;
    }
    fn visit_list_expr(&self, expr: &ListExpr) -> Result {
        return Result::Empty;
    }
    fn visit_dictionary_expr(&self, expr: &DictionaryExpr) -> Result {
        return Result::Empty;
    }
    fn visit_assign_expr(&self, expr: &AssignExpr) -> Result {
        return Result::Empty;
    }
    fn visit_access_expr(&self, expr: &AccessExpr) -> Result {
        return Result::Empty;
    }
    fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result {
        return Result::Empty;
    }
    fn visit_call_expr(&self, expr: &CallExpr) -> Result {
        return Result::Empty;
    }
    fn visit_get_expr(&self, expr: &GetExpr) -> Result {
        return Result::Empty;
    }
    fn visit_set_expr(&self, expr: &SetExpr) -> Result {
        return Result::Empty;
    }
    fn visit_super_expr(&self, expr: &SuperExpr) -> Result {
        return Result::Empty;
    }
    fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result {
        return Result::Empty;
    }
    fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result {
        Result::Literal(expr.value.clone())
    }
    fn visit_logical_expr(&self, expr: &LogicalExpr) -> Result {
        return Result::Empty;
    }
    fn visit_this_expr(&self, expr: &ThisExpr) -> Result {
        return Result::Empty;
    }
    fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result {
        return Result::Empty;
    }
}

impl Exec<'_> {
    fn new(state: &'_ InterpreterState) -> Exec<'_> {
        return Exec { state: state };
    }

    fn interpret(&self) {
        for s in &self.state.stmts {
            println!("Executing {:#?}", s.accept(self));
        }
    }
}

pub fn scan(source: String) {
    let state = &mut InterpreterState::new(source);
    let mut lex = Lexer::new(state);
    lex.scan();
    let mut parser = Parser::new(state);
    parser.parse();
    println!("{:#?}", state.tokens);
    println!("{:#?}", state.stmts);
    let exec = Exec::new(state);
    exec.interpret();
}
