use crate::expr::*;
use crate::state::*;
use crate::stmt::*;
use crate::token::*;

#[derive(Debug, Clone)]
pub struct CallStack {
    pub function: String,
    pub loop_count: usize,
}

pub struct Parser<'a> {
    pub current: usize,
    pub cls: Vec<CallStack>,
    pub state: &'a mut InterpreterState,
}

const MAX_FUNCTION_PARAMS: usize = 255;

impl Parser<'_> {
    pub fn new(state: &'_ mut InterpreterState) -> Parser<'_> {
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

    fn enter_function(&mut self, name: String) {
        self.cls.push(CallStack {
            function: name,
            loop_count: 0,
        });
    }

    fn peek(&self) -> TokenData {
        return self.state.tokens[self.current].clone();
    }

    fn is_at_end(&self) -> bool {
        return self.peek().token == Token::EOF;
    }

    fn leave_function(&mut self, name: String) {
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

    pub fn parse(&mut self) {
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

        self.enter_function(name.lexeme.clone());

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

        self.leave_function(name.lexeme.clone());

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

        let lambda_name: String = format!("lambda{}", self.cls.len());
        self.enter_function(lambda_name.clone());

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
        let mut expr = self.or();
        if self.matches(Token::Equal) {
            let equal = self.previous();
            let value = self.assignment();

            let access = match expr {
                Expr::Access(a) => {
                    let mut obj = a.clone().object;
                    loop {
                        if let Expr::Access(_a) = *obj {
                            obj = _a.object;
                        } else {
                            break;
                        }
                    }
                    expr = obj.as_ref().clone();
                    Some(a)
                }
                _ => None,
            };

            match expr {
                Expr::Var(variable) => {
                    let assign = AssignExpr {
                        name: variable.name.unwrap(),
                        value: Box::new(value),
                        access: access,
                    };
                    return Expr::Assign(assign);
                }
                Expr::Get(get) => {
                    let set = SetExpr {
                        name: get.name,
                        value: Box::new(value),
                        object: get.object,
                        access: access,
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

    // fn synchronize(&mut self) {
    //     self.advance();
    //     while !self.is_at_end() {
    //         match self.peek().token {
    //             Token::Class => return,
    //             Token::Fn => return,
    //             Token::Let => return,
    //             Token::For => return,
    //             Token::If => return,
    //             Token::While => return,
    //             Token::Return => return,
    //             _ => (),
    //         }
    //         self.advance();
    //     }
    // }
}
