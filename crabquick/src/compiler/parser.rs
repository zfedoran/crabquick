//! JavaScript parser
//!
//! Recursive descent parser for JavaScript

use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use super::lexer::{Lexer, Token, TokenKind, SourceLocation};
use super::ast::*;

/// Parse error
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub location: SourceLocation,
}

impl ParseError {
    fn new(message: String, location: SourceLocation) -> Self {
        ParseError { message, location }
    }
}

/// Parse result
pub type ParseResult<T> = Result<T, ParseError>;

/// JavaScript parser
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
    peeked: Option<Token>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Lexer::new(source);
        let current = lexer.next_token();

        Parser {
            lexer,
            current,
            peeked: None,
        }
    }

    /// Parses the source code into a Program
    pub fn parse(mut self) -> ParseResult<Program> {
        let mut body = Vec::new();

        while !self.is_eof() {
            body.push(self.parse_statement()?);
        }

        Ok(Program::new(body))
    }

    /// Returns the current token
    fn current_token(&self) -> &Token {
        &self.current
    }

    /// Returns the current token kind
    fn current_kind(&self) -> &TokenKind {
        &self.current.kind
    }

    /// Peeks at the next token
    fn peek(&mut self) -> &Token {
        if self.peeked.is_none() {
            self.peeked = Some(self.lexer.next_token());
        }
        self.peeked.as_ref().unwrap()
    }

    /// Advances to the next token
    fn advance(&mut self) {
        self.current = if let Some(peeked) = self.peeked.take() {
            peeked
        } else {
            self.lexer.next_token()
        };
    }

    /// Checks if we're at EOF
    fn is_eof(&self) -> bool {
        matches!(self.current.kind, TokenKind::Eof)
    }

    /// Expects a specific token kind and advances
    fn expect(&mut self, expected: TokenKind) -> ParseResult<()> {
        if self.current.kind == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::new(
                format!("Expected {:?}, found {:?}", expected, self.current.kind),
                self.current.location,
            ))
        }
    }

    /// Consumes a token if it matches, returns true if consumed
    fn consume_if(&mut self, kind: &TokenKind) -> bool {
        if core::mem::discriminant(&self.current.kind) == core::mem::discriminant(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Parses a statement
    fn parse_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.current.location;

        match self.current.kind {
            TokenKind::Var | TokenKind::Let | TokenKind::Const => {
                self.parse_var_statement()
            }
            TokenKind::Function => {
                self.parse_function_declaration()
            }
            TokenKind::If => {
                self.parse_if_statement()
            }
            TokenKind::While => {
                self.parse_while_statement()
            }
            TokenKind::Do => {
                self.parse_do_while_statement()
            }
            TokenKind::For => {
                self.parse_for_statement()
            }
            TokenKind::Return => {
                self.parse_return_statement()
            }
            TokenKind::Break => {
                self.parse_break_statement()
            }
            TokenKind::Continue => {
                self.parse_continue_statement()
            }
            TokenKind::Throw => {
                self.parse_throw_statement()
            }
            TokenKind::Try => {
                self.parse_try_statement()
            }
            TokenKind::Switch => {
                self.parse_switch_statement()
            }
            TokenKind::LBrace => {
                self.parse_block_statement()
            }
            TokenKind::Semicolon => {
                self.advance();
                Ok(Stmt::Empty { loc })
            }
            TokenKind::Identifier(_) => {
                // Check if this is a labeled statement (identifier followed by colon)
                let peeked_kind = self.peek().kind.clone();
                if matches!(peeked_kind, TokenKind::Colon) {
                    self.parse_labeled_statement()
                } else {
                    // Expression statement
                    let expr = self.parse_expression()?;
                    self.consume_semicolon();
                    Ok(Stmt::Expression { expr, loc })
                }
            }
            _ => {
                // Expression statement
                let expr = self.parse_expression()?;
                self.consume_semicolon();
                Ok(Stmt::Expression { expr, loc })
            }
        }
    }

    /// Parses a variable declaration statement
    fn parse_var_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.current.location;
        let kind = match &self.current.kind {
            TokenKind::Var => VarKind::Var,
            TokenKind::Let => VarKind::Let,
            TokenKind::Const => VarKind::Const,
            _ => return Err(ParseError::new("Expected var/let/const".to_string(), loc)),
        };

        self.advance();

        let mut declarations = Vec::new();

        loop {
            let name = self.parse_identifier()?;

            let init = if self.consume_if(&TokenKind::Assign) {
                Some(self.parse_assignment_expression()?)
            } else {
                None
            };

            declarations.push(VarDeclarator { name, init });

            if !self.consume_if(&TokenKind::Comma) {
                break;
            }
        }

        self.consume_semicolon();

        Ok(Stmt::VarDecl { kind, declarations, loc })
    }

    /// Parses a function declaration
    fn parse_function_declaration(&mut self) -> ParseResult<Stmt> {
        let loc = self.current.location;
        self.expect(TokenKind::Function)?;

        let name = self.parse_identifier()?;

        self.expect(TokenKind::LParen)?;
        let params = self.parse_parameter_list()?;
        self.expect(TokenKind::RParen)?;

        self.expect(TokenKind::LBrace)?;
        let body = self.parse_statement_list()?;
        self.expect(TokenKind::RBrace)?;

        Ok(Stmt::FunctionDecl { name, params, body, loc })
    }

    /// Parses an if statement
    fn parse_if_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.current.location;
        self.expect(TokenKind::If)?;

        self.expect(TokenKind::LParen)?;
        let test = self.parse_expression()?;
        self.expect(TokenKind::RParen)?;

        let consequent = Box::new(self.parse_statement()?);

        let alternate = if self.consume_if(&TokenKind::Else) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Ok(Stmt::If { test, consequent, alternate, loc })
    }

    /// Parses a while statement
    fn parse_while_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.current.location;
        self.expect(TokenKind::While)?;

        self.expect(TokenKind::LParen)?;
        let test = self.parse_expression()?;
        self.expect(TokenKind::RParen)?;

        let body = Box::new(self.parse_statement()?);

        Ok(Stmt::While { test, body, loc })
    }

    /// Parses a do-while statement
    fn parse_do_while_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.current.location;
        self.expect(TokenKind::Do)?;

        let body = Box::new(self.parse_statement()?);

        self.expect(TokenKind::While)?;
        self.expect(TokenKind::LParen)?;
        let test = self.parse_expression()?;
        self.expect(TokenKind::RParen)?;

        self.consume_semicolon();

        Ok(Stmt::DoWhile { body, test, loc })
    }

    /// Parses a for statement
    fn parse_for_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.current.location;
        self.expect(TokenKind::For)?;

        self.expect(TokenKind::LParen)?;

        // Parse init
        let init = if self.consume_if(&TokenKind::Semicolon) {
            None
        } else if matches!(self.current.kind, TokenKind::Var | TokenKind::Let | TokenKind::Const) {
            let kind = match &self.current.kind {
                TokenKind::Var => VarKind::Var,
                TokenKind::Let => VarKind::Let,
                TokenKind::Const => VarKind::Const,
                _ => unreachable!(),
            };
            self.advance();

            let mut declarations = Vec::new();
            loop {
                let name = self.parse_identifier()?;
                let init_expr = if self.consume_if(&TokenKind::Assign) {
                    Some(self.parse_assignment_expression()?)
                } else {
                    None
                };
                declarations.push(VarDeclarator { name, init: init_expr });

                if !self.consume_if(&TokenKind::Comma) {
                    break;
                }
            }

            // Check for for-in
            if self.consume_if(&TokenKind::In) {
                let right = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                let body = Box::new(self.parse_statement()?);

                return Ok(Stmt::ForIn {
                    left: ForInit::VarDecl { kind, declarations },
                    right,
                    body,
                    loc,
                });
            }

            // Check for for-of
            if self.consume_if(&TokenKind::Of) {
                let right = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                let body = Box::new(self.parse_statement()?);

                return Ok(Stmt::ForOf {
                    left: ForInit::VarDecl { kind, declarations },
                    right,
                    body,
                    loc,
                });
            }

            self.expect(TokenKind::Semicolon)?;
            Some(ForInit::VarDecl { kind, declarations })
        } else if self.current.kind != TokenKind::Semicolon {
            // For for-in/for-of, we need to parse LHS without treating 'in' as binary operator
            // First, try to parse as a simple LHS (identifier or member expression)
            let expr = self.parse_left_hand_side_expression()?;

            // Check for for-in
            if self.consume_if(&TokenKind::In) {
                let right = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                let body = Box::new(self.parse_statement()?);

                return Ok(Stmt::ForIn {
                    left: ForInit::Expr(expr),
                    right,
                    body,
                    loc,
                });
            }

            // Check for for-of
            if self.consume_if(&TokenKind::Of) {
                let right = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                let body = Box::new(self.parse_statement()?);

                return Ok(Stmt::ForOf {
                    left: ForInit::Expr(expr),
                    right,
                    body,
                    loc,
                });
            }

            // Not for-in/for-of, need to continue parsing as full expression
            // The expr we parsed is just the LHS, we need to handle binary ops
            let full_expr = self.continue_parsing_expression(expr)?;

            self.expect(TokenKind::Semicolon)?;
            Some(ForInit::Expr(full_expr))
        } else {
            None
        };

        // Parse test
        let test = if self.consume_if(&TokenKind::Semicolon) {
            None
        } else {
            let t = Some(self.parse_expression()?);
            self.expect(TokenKind::Semicolon)?;
            t
        };

        // Parse update
        let update = if self.consume_if(&TokenKind::RParen) {
            None
        } else {
            let u = Some(self.parse_expression()?);
            self.expect(TokenKind::RParen)?;
            u
        };

        let body = Box::new(self.parse_statement()?);

        Ok(Stmt::For { init, test, update, body, loc })
    }

    /// Parses a return statement
    fn parse_return_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.current.location;
        self.expect(TokenKind::Return)?;

        // ASI: if there's a newline after return, or we have semicolon/rbrace/eof, don't parse expression
        let argument = if self.consume_if(&TokenKind::Semicolon)
            || self.current.had_newline
            || matches!(self.current.kind, TokenKind::RBrace | TokenKind::Eof) {
            None
        } else {
            let arg = Some(self.parse_expression()?);
            self.consume_semicolon();
            arg
        };

        Ok(Stmt::Return { argument, loc })
    }

    /// Parses a break statement
    fn parse_break_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.current.location;
        self.expect(TokenKind::Break)?;

        // Only parse label if there was no newline before the identifier (ASI)
        let label = if !self.current.had_newline && matches!(self.current.kind, TokenKind::Identifier(_)) {
            Some(self.parse_identifier()?)
        } else {
            None
        };

        self.consume_semicolon();

        Ok(Stmt::Break { label, loc })
    }

    /// Parses a continue statement
    fn parse_continue_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.current.location;
        self.expect(TokenKind::Continue)?;

        // Only parse label if there was no newline before the identifier (ASI)
        let label = if !self.current.had_newline && matches!(self.current.kind, TokenKind::Identifier(_)) {
            Some(self.parse_identifier()?)
        } else {
            None
        };

        self.consume_semicolon();

        Ok(Stmt::Continue { label, loc })
    }

    /// Parses a labeled statement (label: statement)
    fn parse_labeled_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.current.location;

        // Get the label name
        let label = self.parse_identifier()?;

        // Expect colon
        self.expect(TokenKind::Colon)?;

        // Parse the body statement
        let body = Box::new(self.parse_statement()?);

        Ok(Stmt::Labeled { label, body, loc })
    }

    /// Parses a throw statement
    fn parse_throw_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.current.location;
        self.expect(TokenKind::Throw)?;

        let argument = self.parse_expression()?;
        self.consume_semicolon();

        Ok(Stmt::Throw { argument, loc })
    }

    /// Parses a try statement
    fn parse_try_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.current.location;
        self.expect(TokenKind::Try)?;

        self.expect(TokenKind::LBrace)?;
        let block = self.parse_statement_list()?;
        self.expect(TokenKind::RBrace)?;

        let handler = if self.consume_if(&TokenKind::Catch) {
            let param = if self.consume_if(&TokenKind::LParen) {
                let p = Some(self.parse_identifier()?);
                self.expect(TokenKind::RParen)?;
                p
            } else {
                None
            };

            self.expect(TokenKind::LBrace)?;
            let body = self.parse_statement_list()?;
            self.expect(TokenKind::RBrace)?;

            Some(CatchClause { param, body })
        } else {
            None
        };

        let finalizer = if self.consume_if(&TokenKind::Finally) {
            self.expect(TokenKind::LBrace)?;
            let stmts = self.parse_statement_list()?;
            self.expect(TokenKind::RBrace)?;
            Some(stmts)
        } else {
            None
        };

        if handler.is_none() && finalizer.is_none() {
            return Err(ParseError::new(
                "Try statement must have catch or finally".to_string(),
                loc,
            ));
        }

        Ok(Stmt::Try { block, handler, finalizer, loc })
    }

    /// Parses a switch statement
    fn parse_switch_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.current.location;
        self.expect(TokenKind::Switch)?;

        self.expect(TokenKind::LParen)?;
        let discriminant = self.parse_expression()?;
        self.expect(TokenKind::RParen)?;

        self.expect(TokenKind::LBrace)?;

        let mut cases = Vec::new();

        while !self.consume_if(&TokenKind::RBrace) {
            let test = if self.consume_if(&TokenKind::Case) {
                Some(self.parse_expression()?)
            } else if self.consume_if(&TokenKind::Default) {
                None
            } else {
                return Err(ParseError::new(
                    "Expected 'case' or 'default'".to_string(),
                    self.current.location,
                ));
            };

            self.expect(TokenKind::Colon)?;

            let mut consequent = Vec::new();
            while !matches!(self.current.kind, TokenKind::Case | TokenKind::Default | TokenKind::RBrace) {
                consequent.push(self.parse_statement()?);
            }

            cases.push(SwitchCase { test, consequent });
        }

        Ok(Stmt::Switch { discriminant, cases, loc })
    }

    /// Parses a block statement
    fn parse_block_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.current.location;
        self.expect(TokenKind::LBrace)?;
        let stmts = self.parse_statement_list()?;
        self.expect(TokenKind::RBrace)?;

        Ok(Stmt::Block { stmts, loc })
    }

    /// Parses a list of statements (until })
    fn parse_statement_list(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut stmts = Vec::new();

        while !matches!(self.current.kind, TokenKind::RBrace | TokenKind::Eof) {
            stmts.push(self.parse_statement()?);
        }

        Ok(stmts)
    }

    /// Consumes a semicolon (or allows ASI)
    fn consume_semicolon(&mut self) {
        self.consume_if(&TokenKind::Semicolon);
        // In a full implementation, we'd handle automatic semicolon insertion here
    }

    /// Parses an identifier
    fn parse_identifier(&mut self) -> ParseResult<String> {
        match &self.current.kind {
            TokenKind::Identifier(name) => {
                let result = name.clone();
                self.advance();
                Ok(result)
            }
            _ => Err(ParseError::new(
                format!("Expected identifier, found {:?}", self.current.kind),
                self.current.location,
            )),
        }
    }

    /// Parses a property name (identifier or reserved word)
    /// Used for member access after dot and object property keys
    fn parse_property_name(&mut self) -> ParseResult<String> {
        // First try as identifier
        if let TokenKind::Identifier(name) = &self.current.kind {
            let result = name.clone();
            self.advance();
            return Ok(result);
        }

        // Try as reserved word
        if self.current.kind.is_reserved_word() {
            if let Some(name) = self.current.kind.as_property_name() {
                self.advance();
                return Ok(name);
            }
        }

        Err(ParseError::new(
            format!("Expected property name, found {:?}", self.current.kind),
            self.current.location,
        ))
    }

    /// Parses a parameter list
    fn parse_parameter_list(&mut self) -> ParseResult<Vec<String>> {
        let mut params = Vec::new();

        if matches!(self.current.kind, TokenKind::RParen) {
            return Ok(params);
        }

        loop {
            params.push(self.parse_identifier()?);

            if !self.consume_if(&TokenKind::Comma) {
                break;
            }
        }

        Ok(params)
    }

    // ===== Expression Parsing =====

    /// Parses an expression
    fn parse_expression(&mut self) -> ParseResult<Expr> {
        self.parse_sequence_expression()
    }

    /// Parses a left-hand-side expression (for for-in/for-of loop variable)
    /// This parses identifier, member access, call expressions but NOT binary operators
    fn parse_left_hand_side_expression(&mut self) -> ParseResult<Expr> {
        self.parse_postfix_expression()
    }

    /// Continues parsing an expression from an already-parsed LHS
    /// This handles assignment and binary operators
    fn continue_parsing_expression(&mut self, left: Expr) -> ParseResult<Expr> {
        // Check for assignment operators first
        let assign_op = match &self.current.kind {
            TokenKind::Assign => Some(AssignOp::Assign),
            TokenKind::PlusAssign => Some(AssignOp::AddAssign),
            TokenKind::MinusAssign => Some(AssignOp::SubAssign),
            TokenKind::StarAssign => Some(AssignOp::MulAssign),
            TokenKind::SlashAssign => Some(AssignOp::DivAssign),
            TokenKind::PercentAssign => Some(AssignOp::ModAssign),
            TokenKind::AmpersandAssign => Some(AssignOp::BitAndAssign),
            TokenKind::PipeAssign => Some(AssignOp::BitOrAssign),
            TokenKind::CaretAssign => Some(AssignOp::BitXorAssign),
            TokenKind::LtLtAssign => Some(AssignOp::LeftShiftAssign),
            TokenKind::GtGtAssign => Some(AssignOp::RightShiftAssign),
            TokenKind::GtGtGtAssign => Some(AssignOp::UnsignedRightShiftAssign),
            _ => None,
        };

        if let Some(op) = assign_op {
            let loc = self.current.location;
            self.advance();
            let right = self.parse_assignment_expression()?;
            return Ok(Expr::Assignment {
                op,
                left: Box::new(left),
                right: Box::new(right),
                loc,
            });
        }

        // Otherwise, continue with binary operators
        // We need to handle ternary, logical, etc.
        self.continue_binary_expression(left)
    }

    /// Continues parsing binary operators from an already-parsed LHS
    fn continue_binary_expression(&mut self, left: Expr) -> ParseResult<Expr> {
        let loc = left.location();
        let mut result = left;

        // Handle binary operators in precedence order
        // This is a simplified version - for full correctness we'd need to integrate
        // with the existing precedence parsing
        loop {
            let op = match &self.current.kind {
                // Logical OR
                TokenKind::LogicalOr => { self.advance(); BinaryOp::LogicalOr }
                // Logical AND
                TokenKind::LogicalAnd => { self.advance(); BinaryOp::LogicalAnd }
                // Bitwise OR
                TokenKind::Pipe => { self.advance(); BinaryOp::BitOr }
                // Bitwise XOR
                TokenKind::Caret => { self.advance(); BinaryOp::BitXor }
                // Bitwise AND
                TokenKind::Ampersand => { self.advance(); BinaryOp::BitAnd }
                // Equality
                TokenKind::Eq => { self.advance(); BinaryOp::Eq }
                TokenKind::NotEq => { self.advance(); BinaryOp::NotEq }
                TokenKind::StrictEq => { self.advance(); BinaryOp::StrictEq }
                TokenKind::StrictNotEq => { self.advance(); BinaryOp::StrictNotEq }
                // Relational (but NOT 'in' - that's why we're here!)
                TokenKind::Lt => { self.advance(); BinaryOp::Lt }
                TokenKind::LtEq => { self.advance(); BinaryOp::LtEq }
                TokenKind::Gt => { self.advance(); BinaryOp::Gt }
                TokenKind::GtEq => { self.advance(); BinaryOp::GtEq }
                TokenKind::InstanceOf => { self.advance(); BinaryOp::InstanceOf }
                // In operator is NOT included here - that's the whole point
                // Shift
                TokenKind::LtLt => { self.advance(); BinaryOp::LeftShift }
                TokenKind::GtGt => { self.advance(); BinaryOp::RightShift }
                TokenKind::GtGtGt => { self.advance(); BinaryOp::UnsignedRightShift }
                // Additive
                TokenKind::Plus => { self.advance(); BinaryOp::Add }
                TokenKind::Minus => { self.advance(); BinaryOp::Sub }
                // Multiplicative
                TokenKind::Star => { self.advance(); BinaryOp::Mul }
                TokenKind::Slash => { self.advance(); BinaryOp::Div }
                TokenKind::Percent => { self.advance(); BinaryOp::Mod }
                TokenKind::StarStar => { self.advance(); BinaryOp::Pow }
                _ => break,
            };

            let right = self.parse_unary_expression()?;
            result = Expr::Binary {
                op,
                left: Box::new(result),
                right: Box::new(right),
                loc,
            };
        }

        // Handle ternary
        if self.consume_if(&TokenKind::Question) {
            let consequent = self.parse_assignment_expression()?;
            self.expect(TokenKind::Colon)?;
            let alternate = self.parse_assignment_expression()?;
            result = Expr::Conditional {
                test: Box::new(result),
                consequent: Box::new(consequent),
                alternate: Box::new(alternate),
                loc,
            };
        }

        Ok(result)
    }

    /// Parses a sequence expression (comma operator)
    fn parse_sequence_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;
        let mut exprs = vec![self.parse_assignment_expression()?];

        while self.consume_if(&TokenKind::Comma) {
            exprs.push(self.parse_assignment_expression()?);
        }

        if exprs.len() == 1 {
            Ok(exprs.into_iter().next().unwrap())
        } else {
            Ok(Expr::Sequence { exprs, loc })
        }
    }

    /// Parses an assignment expression
    fn parse_assignment_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;

        // Try arrow function
        if matches!(self.current.kind, TokenKind::Identifier(_)) {
            let checkpoint_pos = self.lexer.pc();
            let checkpoint_current = self.current.clone();
            let checkpoint_peeked = self.peeked.clone();

            if let Ok(name) = self.parse_identifier() {
                if self.consume_if(&TokenKind::Arrow) {
                    let body = self.parse_arrow_body()?;
                    return Ok(Expr::Arrow {
                        params: alloc::vec![name],
                        body,
                        loc,
                    });
                }
            }

            // Restore state
            self.lexer.set_pc(checkpoint_pos);
            self.current = checkpoint_current;
            self.peeked = checkpoint_peeked;
        }

        // Try conditional
        let expr = self.parse_conditional_expression()?;

        // Check for assignment operator
        let op = match &self.current.kind {
            TokenKind::Assign => AssignOp::Assign,
            TokenKind::PlusAssign => AssignOp::AddAssign,
            TokenKind::MinusAssign => AssignOp::SubAssign,
            TokenKind::StarAssign => AssignOp::MulAssign,
            TokenKind::SlashAssign => AssignOp::DivAssign,
            TokenKind::PercentAssign => AssignOp::ModAssign,
            TokenKind::LtLtAssign => AssignOp::LeftShiftAssign,
            TokenKind::GtGtAssign => AssignOp::RightShiftAssign,
            TokenKind::GtGtGtAssign => AssignOp::UnsignedRightShiftAssign,
            TokenKind::AmpersandAssign => AssignOp::BitAndAssign,
            TokenKind::PipeAssign => AssignOp::BitOrAssign,
            TokenKind::CaretAssign => AssignOp::BitXorAssign,
            _ => return Ok(expr),
        };

        self.advance();
        let right = Box::new(self.parse_assignment_expression()?);

        Ok(Expr::Assignment {
            op,
            left: Box::new(expr),
            right,
            loc,
        })
    }

    /// Parses a conditional expression (ternary)
    fn parse_conditional_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;
        let test = self.parse_logical_or_expression()?;

        if self.consume_if(&TokenKind::Question) {
            let consequent = Box::new(self.parse_assignment_expression()?);
            self.expect(TokenKind::Colon)?;
            let alternate = Box::new(self.parse_assignment_expression()?);

            Ok(Expr::Conditional {
                test: Box::new(test),
                consequent,
                alternate,
                loc,
            })
        } else {
            Ok(test)
        }
    }

    /// Parses a logical OR expression
    fn parse_logical_or_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;
        let mut left = self.parse_logical_and_expression()?;

        while matches!(self.current.kind, TokenKind::LogicalOr | TokenKind::NullishCoalescing) {
            let op = if self.consume_if(&TokenKind::LogicalOr) {
                BinaryOp::LogicalOr
            } else {
                self.advance(); // NullishCoalescing
                BinaryOp::NullishCoalescing
            };

            let right = Box::new(self.parse_logical_and_expression()?);
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right,
                loc,
            };
        }

        Ok(left)
    }

    /// Parses a logical AND expression
    fn parse_logical_and_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;
        let mut left = self.parse_bitwise_or_expression()?;

        while self.consume_if(&TokenKind::LogicalAnd) {
            let right = Box::new(self.parse_bitwise_or_expression()?);
            left = Expr::Binary {
                op: BinaryOp::LogicalAnd,
                left: Box::new(left),
                right,
                loc,
            };
        }

        Ok(left)
    }

    /// Parses a bitwise OR expression
    fn parse_bitwise_or_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;
        let mut left = self.parse_bitwise_xor_expression()?;

        while self.consume_if(&TokenKind::Pipe) {
            let right = Box::new(self.parse_bitwise_xor_expression()?);
            left = Expr::Binary {
                op: BinaryOp::BitOr,
                left: Box::new(left),
                right,
                loc,
            };
        }

        Ok(left)
    }

    /// Parses a bitwise XOR expression
    fn parse_bitwise_xor_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;
        let mut left = self.parse_bitwise_and_expression()?;

        while self.consume_if(&TokenKind::Caret) {
            let right = Box::new(self.parse_bitwise_and_expression()?);
            left = Expr::Binary {
                op: BinaryOp::BitXor,
                left: Box::new(left),
                right,
                loc,
            };
        }

        Ok(left)
    }

    /// Parses a bitwise AND expression
    fn parse_bitwise_and_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;
        let mut left = self.parse_equality_expression()?;

        while self.consume_if(&TokenKind::Ampersand) {
            let right = Box::new(self.parse_equality_expression()?);
            left = Expr::Binary {
                op: BinaryOp::BitAnd,
                left: Box::new(left),
                right,
                loc,
            };
        }

        Ok(left)
    }

    /// Parses an equality expression
    fn parse_equality_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;
        let mut left = self.parse_relational_expression()?;

        loop {
            let op = match &self.current.kind {
                TokenKind::Eq => BinaryOp::Eq,
                TokenKind::NotEq => BinaryOp::NotEq,
                TokenKind::StrictEq => BinaryOp::StrictEq,
                TokenKind::StrictNotEq => BinaryOp::StrictNotEq,
                _ => break,
            };

            self.advance();
            let right = Box::new(self.parse_relational_expression()?);
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right,
                loc,
            };
        }

        Ok(left)
    }

    /// Parses a relational expression
    fn parse_relational_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;
        let mut left = self.parse_shift_expression()?;

        loop {
            let op = match &self.current.kind {
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::LtEq => BinaryOp::LtEq,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::GtEq => BinaryOp::GtEq,
                TokenKind::In => BinaryOp::In,
                TokenKind::InstanceOf => BinaryOp::InstanceOf,
                _ => break,
            };

            self.advance();
            let right = Box::new(self.parse_shift_expression()?);
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right,
                loc,
            };
        }

        Ok(left)
    }

    /// Parses a shift expression
    fn parse_shift_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;
        let mut left = self.parse_additive_expression()?;

        loop {
            let op = match &self.current.kind {
                TokenKind::LtLt => BinaryOp::LeftShift,
                TokenKind::GtGt => BinaryOp::RightShift,
                TokenKind::GtGtGt => BinaryOp::UnsignedRightShift,
                _ => break,
            };

            self.advance();
            let right = Box::new(self.parse_additive_expression()?);
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right,
                loc,
            };
        }

        Ok(left)
    }

    /// Parses an additive expression
    fn parse_additive_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;
        let mut left = self.parse_multiplicative_expression()?;

        loop {
            let op = match &self.current.kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => break,
            };

            self.advance();
            let right = Box::new(self.parse_multiplicative_expression()?);
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right,
                loc,
            };
        }

        Ok(left)
    }

    /// Parses a multiplicative expression
    fn parse_multiplicative_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;
        let mut left = self.parse_exponentiation_expression()?;

        loop {
            let op = match &self.current.kind {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                _ => break,
            };

            self.advance();
            let right = Box::new(self.parse_exponentiation_expression()?);
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right,
                loc,
            };
        }

        Ok(left)
    }

    /// Parses an exponentiation expression
    fn parse_exponentiation_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;
        let left = self.parse_unary_expression()?;

        if self.consume_if(&TokenKind::StarStar) {
            let right = Box::new(self.parse_exponentiation_expression()?); // Right associative
            Ok(Expr::Binary {
                op: BinaryOp::Pow,
                left: Box::new(left),
                right,
                loc,
            })
        } else {
            Ok(left)
        }
    }

    /// Parses a unary expression
    fn parse_unary_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;

        match &self.current.kind {
            TokenKind::Plus => {
                self.advance();
                Ok(Expr::Unary {
                    op: UnaryOp::Plus,
                    arg: Box::new(self.parse_unary_expression()?),
                    prefix: true,
                    loc,
                })
            }
            TokenKind::Minus => {
                self.advance();
                Ok(Expr::Unary {
                    op: UnaryOp::Minus,
                    arg: Box::new(self.parse_unary_expression()?),
                    prefix: true,
                    loc,
                })
            }
            TokenKind::Bang => {
                self.advance();
                Ok(Expr::Unary {
                    op: UnaryOp::LogicalNot,
                    arg: Box::new(self.parse_unary_expression()?),
                    prefix: true,
                    loc,
                })
            }
            TokenKind::Tilde => {
                self.advance();
                Ok(Expr::Unary {
                    op: UnaryOp::BitwiseNot,
                    arg: Box::new(self.parse_unary_expression()?),
                    prefix: true,
                    loc,
                })
            }
            TokenKind::TypeOf => {
                self.advance();
                Ok(Expr::Unary {
                    op: UnaryOp::TypeOf,
                    arg: Box::new(self.parse_unary_expression()?),
                    prefix: true,
                    loc,
                })
            }
            TokenKind::Void => {
                self.advance();
                Ok(Expr::Unary {
                    op: UnaryOp::Void,
                    arg: Box::new(self.parse_unary_expression()?),
                    prefix: true,
                    loc,
                })
            }
            TokenKind::Delete => {
                self.advance();
                Ok(Expr::Unary {
                    op: UnaryOp::Delete,
                    arg: Box::new(self.parse_unary_expression()?),
                    prefix: true,
                    loc,
                })
            }
            TokenKind::PlusPlus => {
                self.advance();
                Ok(Expr::Update {
                    op: UpdateOp::Inc,
                    arg: Box::new(self.parse_unary_expression()?),
                    prefix: true,
                    loc,
                })
            }
            TokenKind::MinusMinus => {
                self.advance();
                Ok(Expr::Update {
                    op: UpdateOp::Dec,
                    arg: Box::new(self.parse_unary_expression()?),
                    prefix: true,
                    loc,
                })
            }
            _ => self.parse_postfix_expression(),
        }
    }

    /// Parses a postfix expression
    fn parse_postfix_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;
        let expr = self.parse_call_expression()?;

        match &self.current.kind {
            TokenKind::PlusPlus => {
                self.advance();
                Ok(Expr::Update {
                    op: UpdateOp::Inc,
                    arg: Box::new(expr),
                    prefix: false,
                    loc,
                })
            }
            TokenKind::MinusMinus => {
                self.advance();
                Ok(Expr::Update {
                    op: UpdateOp::Dec,
                    arg: Box::new(expr),
                    prefix: false,
                    loc,
                })
            }
            _ => Ok(expr),
        }
    }

    /// Parses a call expression
    fn parse_call_expression(&mut self) -> ParseResult<Expr> {
        let mut expr = if self.consume_if(&TokenKind::New) {
            self.parse_new_expression()?
        } else {
            self.parse_member_expression()?
        };

        loop {
            match &self.current.kind {
                TokenKind::LParen => {
                    let loc = self.current.location;
                    self.advance();
                    let args = self.parse_argument_list()?;
                    self.expect(TokenKind::RParen)?;

                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                        loc,
                    };
                }
                TokenKind::Dot => {
                    let loc = self.current.location;
                    self.advance();
                    let property = self.parse_property_name()?;

                    expr = Expr::Member {
                        object: Box::new(expr),
                        property: Box::new(Expr::Identifier(property, loc)),
                        computed: false,
                        loc,
                    };
                }
                TokenKind::LBracket => {
                    let loc = self.current.location;
                    self.advance();
                    let property = self.parse_expression()?;
                    self.expect(TokenKind::RBracket)?;

                    expr = Expr::Member {
                        object: Box::new(expr),
                        property: Box::new(property),
                        computed: true,
                        loc,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Parses a new expression
    fn parse_new_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;
        let callee = Box::new(self.parse_member_expression()?);

        let args = if self.consume_if(&TokenKind::LParen) {
            let args = self.parse_argument_list()?;
            self.expect(TokenKind::RParen)?;
            args
        } else {
            Vec::new()
        };

        Ok(Expr::New { callee, args, loc })
    }

    /// Parses a member expression
    fn parse_member_expression(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_primary_expression()?;

        loop {
            match &self.current.kind {
                TokenKind::Dot => {
                    let loc = self.current.location;
                    self.advance();
                    let property = self.parse_property_name()?;

                    expr = Expr::Member {
                        object: Box::new(expr),
                        property: Box::new(Expr::Identifier(property, loc)),
                        computed: false,
                        loc,
                    };
                }
                TokenKind::LBracket => {
                    let loc = self.current.location;
                    self.advance();
                    let property = self.parse_expression()?;
                    self.expect(TokenKind::RBracket)?;

                    expr = Expr::Member {
                        object: Box::new(expr),
                        property: Box::new(property),
                        computed: true,
                        loc,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Parses a primary expression
    fn parse_primary_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;

        match &self.current.kind.clone() {
            TokenKind::Number(n) => {
                let value = *n;
                self.advance();
                Ok(Expr::Literal(Literal::Number(value), loc))
            }
            TokenKind::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(Expr::Literal(Literal::String(value), loc))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Boolean(true), loc))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Literal(Literal::Boolean(false), loc))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Expr::Literal(Literal::Null, loc))
            }
            TokenKind::Undefined => {
                self.advance();
                Ok(Expr::Literal(Literal::Undefined, loc))
            }
            TokenKind::This => {
                self.advance();
                Ok(Expr::This(loc))
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Identifier(name, loc))
            }
            TokenKind::LParen => {
                // Try to parse as arrow function: () => ... or (a, b) => ...
                let checkpoint_pos = self.lexer.pc();
                let checkpoint_current = self.current.clone();
                let checkpoint_peeked = self.peeked.clone();

                self.advance(); // consume (

                // Try parsing as parameter list
                let mut params = Vec::new();
                let mut valid_arrow = true;

                // Check for empty params: () =>
                if !matches!(self.current.kind, TokenKind::RParen) {
                    // Try to parse comma-separated identifiers
                    loop {
                        if let TokenKind::Identifier(name) = &self.current.kind {
                            params.push(name.clone());
                            self.advance();
                        } else {
                            valid_arrow = false;
                            break;
                        }

                        if matches!(self.current.kind, TokenKind::RParen) {
                            break;
                        }

                        if !self.consume_if(&TokenKind::Comma) {
                            valid_arrow = false;
                            break;
                        }
                    }
                }

                // Check for ) =>
                if valid_arrow && self.consume_if(&TokenKind::RParen) {
                    if self.consume_if(&TokenKind::Arrow) {
                        let body = self.parse_arrow_body()?;
                        return Ok(Expr::Arrow {
                            params,
                            body,
                            loc,
                        });
                    }
                }

                // Not an arrow function, restore and parse as expression
                self.lexer.set_pc(checkpoint_pos);
                self.current = checkpoint_current;
                self.peeked = checkpoint_peeked;

                self.advance(); // consume ( again
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::LBracket => {
                self.parse_array_literal()
            }
            TokenKind::LBrace => {
                self.parse_object_literal()
            }
            TokenKind::Function => {
                self.parse_function_expression()
            }
            _ => Err(ParseError::new(
                format!("Unexpected token: {:?}", self.current.kind),
                loc,
            )),
        }
    }

    /// Parses an array literal
    fn parse_array_literal(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;
        self.expect(TokenKind::LBracket)?;

        let mut elements = Vec::new();

        while !self.consume_if(&TokenKind::RBracket) {
            if self.consume_if(&TokenKind::Comma) {
                elements.push(None); // Hole
            } else {
                elements.push(Some(self.parse_assignment_expression()?));

                if !self.consume_if(&TokenKind::Comma) {
                    self.expect(TokenKind::RBracket)?;
                    break;
                }
            }
        }

        Ok(Expr::Array { elements, loc })
    }

    /// Parses an object literal
    fn parse_object_literal(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;
        self.expect(TokenKind::LBrace)?;

        let mut properties = Vec::new();

        while !self.consume_if(&TokenKind::RBrace) {
            let prop = self.parse_object_property()?;
            properties.push(prop);

            if !self.consume_if(&TokenKind::Comma) {
                self.expect(TokenKind::RBrace)?;
                break;
            }
        }

        Ok(Expr::Object { properties, loc })
    }

    /// Parses a single object property (handles get/set/method/regular)
    fn parse_object_property(&mut self) -> ParseResult<Property> {
        let prop_loc = self.current.location;

        // Check for getter/setter: get/set followed by property name
        if let TokenKind::Identifier(name) = &self.current.kind.clone() {
            let is_get = name == "get";
            let is_set = name == "set";

            if is_get || is_set {
                // Peek at next token to determine if this is getter/setter or regular property
                let next_kind = self.peek().kind.clone();

                // If followed by : , } ( then it's a regular property or method named "get"/"set"
                let is_regular = matches!(next_kind,
                    TokenKind::Colon | TokenKind::Comma | TokenKind::RBrace | TokenKind::LParen);

                if !is_regular {
                    // It's a getter or setter
                    self.advance(); // consume 'get' or 'set'

                    // Parse the property name
                    let key = self.parse_property_key()?;

                    // Parse parameter list
                    self.expect(TokenKind::LParen)?;
                    let params = if is_set {
                        // Setter has exactly one parameter
                        let param = self.parse_identifier()?;
                        self.expect(TokenKind::RParen)?;
                        vec![param]
                    } else {
                        // Getter has no parameters
                        self.expect(TokenKind::RParen)?;
                        vec![]
                    };

                    // Parse function body
                    self.expect(TokenKind::LBrace)?;
                    let body = self.parse_statement_list()?;
                    self.expect(TokenKind::RBrace)?;

                    let value = Expr::Function {
                        name: None,
                        params,
                        body,
                        loc: prop_loc,
                    };

                    return Ok(Property {
                        key,
                        value,
                        kind: if is_get { PropertyKind::Get } else { PropertyKind::Set },
                    });
                }
            }
        }

        // Parse regular property key
        let key = self.parse_property_key()?;

        // Check what follows the key
        match &self.current.kind {
            TokenKind::LParen => {
                // Shorthand method: { f() { ... } }
                self.advance(); // consume '('
                let params = self.parse_parameter_list()?;
                self.expect(TokenKind::RParen)?;
                self.expect(TokenKind::LBrace)?;
                let body = self.parse_statement_list()?;
                self.expect(TokenKind::RBrace)?;

                let value = Expr::Function {
                    name: None,
                    params,
                    body,
                    loc: prop_loc,
                };

                Ok(Property {
                    key,
                    value,
                    kind: PropertyKind::Init,
                })
            }
            TokenKind::Colon => {
                // Regular property: { x: value }
                self.advance(); // consume ':'
                let value = self.parse_assignment_expression()?;
                Ok(Property {
                    key,
                    value,
                    kind: PropertyKind::Init,
                })
            }
            _ => Err(ParseError::new(
                format!("Expected ':' or '(' after property key, found {:?}", self.current.kind),
                self.current.location,
            )),
        }
    }

    /// Parses a property key (identifier, string, number, computed, or reserved word)
    fn parse_property_key(&mut self) -> ParseResult<PropertyKey> {
        match &self.current.kind.clone() {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(PropertyKey::Identifier(name))
            }
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(PropertyKey::Literal(Literal::String(s)))
            }
            TokenKind::Number(n) => {
                let n = *n;
                self.advance();
                Ok(PropertyKey::Literal(Literal::Number(n)))
            }
            TokenKind::LBracket => {
                self.advance();
                let expr = self.parse_assignment_expression()?;
                self.expect(TokenKind::RBracket)?;
                Ok(PropertyKey::Computed(Box::new(expr)))
            }
            kind if kind.is_reserved_word() => {
                let name = kind.as_property_name().unwrap_or_default();
                self.advance();
                Ok(PropertyKey::Identifier(name))
            }
            _ => Err(ParseError::new(
                "Expected property key".to_string(),
                self.current.location,
            )),
        }
    }

    /// Parses a function expression
    fn parse_function_expression(&mut self) -> ParseResult<Expr> {
        let loc = self.current.location;
        self.expect(TokenKind::Function)?;

        let name = if matches!(self.current.kind, TokenKind::Identifier(_)) {
            Some(self.parse_identifier()?)
        } else {
            None
        };

        self.expect(TokenKind::LParen)?;
        let params = self.parse_parameter_list()?;
        self.expect(TokenKind::RParen)?;

        self.expect(TokenKind::LBrace)?;
        let body = self.parse_statement_list()?;
        self.expect(TokenKind::RBrace)?;

        Ok(Expr::Function { name, params, body, loc })
    }

    /// Parses an arrow function body
    fn parse_arrow_body(&mut self) -> ParseResult<ArrowBody> {
        if self.consume_if(&TokenKind::LBrace) {
            let stmts = self.parse_statement_list()?;
            self.expect(TokenKind::RBrace)?;
            Ok(ArrowBody::Block(stmts))
        } else {
            let expr = self.parse_assignment_expression()?;
            Ok(ArrowBody::Expr(Box::new(expr)))
        }
    }

    /// Parses an argument list
    fn parse_argument_list(&mut self) -> ParseResult<Vec<Expr>> {
        let mut args = Vec::new();

        if matches!(self.current.kind, TokenKind::RParen) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_assignment_expression()?);

            if !self.consume_if(&TokenKind::Comma) {
                break;
            }
        }

        Ok(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let parser = Parser::new("42");
        let program = parser.parse().unwrap();

        assert_eq!(program.body.len(), 1);
        match &program.body[0] {
            Stmt::Expression { expr, .. } => {
                match expr {
                    Expr::Literal(Literal::Number(n), _) => assert_eq!(*n, 42.0),
                    _ => panic!("Expected number literal"),
                }
            }
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_parse_binary_expr() {
        let parser = Parser::new("2 + 3");
        let program = parser.parse().unwrap();

        assert_eq!(program.body.len(), 1);
        match &program.body[0] {
            Stmt::Expression { expr, .. } => {
                match expr {
                    Expr::Binary { op, .. } => assert_eq!(*op, BinaryOp::Add),
                    _ => panic!("Expected binary expression"),
                }
            }
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_parse_var_statement() {
        let parser = Parser::new("var x = 10;");
        let program = parser.parse().unwrap();

        assert_eq!(program.body.len(), 1);
        match &program.body[0] {
            Stmt::VarDecl { kind, declarations, .. } => {
                assert_eq!(*kind, VarKind::Var);
                assert_eq!(declarations.len(), 1);
                assert_eq!(declarations[0].name, "x");
            }
            _ => panic!("Expected var declaration"),
        }
    }

    #[test]
    fn test_parse_function() {
        let parser = Parser::new("function add(a, b) { return a + b; }");
        let program = parser.parse().unwrap();

        assert_eq!(program.body.len(), 1);
        match &program.body[0] {
            Stmt::FunctionDecl { name, params, body, .. } => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 2);
                assert_eq!(params[0], "a");
                assert_eq!(params[1], "b");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let parser = Parser::new("if (x > 0) return x;");
        let program = parser.parse().unwrap();

        assert_eq!(program.body.len(), 1);
        match &program.body[0] {
            Stmt::If { .. } => {}
            _ => panic!("Expected if statement"),
        }
    }
}
