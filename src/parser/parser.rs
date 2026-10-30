use super::ast::*;
use super::lexer::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Incomplete,
    Syntax(String),
}

pub fn parse(tokens: &[Token]) -> Result<Option<Statement>, ParseError> {
    let mut parser = Parser {
        tokens,
        position: 0,
    };
    let statement = parser.parse_sequence(&[])?;
    parser.skip_separators();

    if parser.position < tokens.len() {
        return Err(ParseError::Syntax("unexpected token".to_string()));
    }

    Ok(statement)
}

struct Parser<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl Parser<'_> {
    fn parse_sequence(&mut self, stops: &[&str]) -> Result<Option<Statement>, ParseError> {
        let mut statements = Vec::new();
        self.skip_separators();

        while self.position < self.tokens.len() {
            if self.current_word_is_any(stops) {
                break;
            }

            let statement = if self.current_word_is("if") {
                self.parse_if()?
            } else {
                self.parse_simple_statement()?
            };
            statements.push(statement);

            if self.position < self.tokens.len()
                && !self.at_separator()
                && !self.current_word_is_any(stops)
            {
                return Err(ParseError::Syntax(
                    "expected a command separator".to_string(),
                ));
            }

            self.skip_separators();
        }

        Ok(match statements.len() {
            0 => None,
            1 => statements.pop(),
            _ => Some(Statement::Sequence(statements)),
        })
    }

    fn parse_if(&mut self) -> Result<Statement, ParseError> {
        self.position += 1; // if
        let mut branches = Vec::new();
        let (condition, body) = self.parse_if_branch()?;
        branches.push((condition, body));

        while self.current_word_is("elif") {
            self.position += 1;
            let (condition, body) = self.parse_if_branch()?;
            branches.push((condition, body));
        }

        let else_branch = if self.current_word_is("else") {
            self.position += 1;
            let body = self.parse_sequence(&["end"])?;
            Some(Box::new(body.unwrap_or_else(empty_sequence)))
        } else {
            None
        };

        if self.current_word_is("end") {
            self.position += 1;
        } else if self.position >= self.tokens.len() {
            return Err(ParseError::Incomplete);
        } else {
            return Err(ParseError::Syntax("expected 'end'".to_string()));
        }

        Ok(Statement::If {
            branches,
            else_branch,
        })
    }

    fn parse_if_branch(&mut self) -> Result<(Statement, Statement), ParseError> {
        let mut condition_tokens = Vec::new();

        while self.position < self.tokens.len() {
            if self.current_word_is("then") {
                break;
            }

            let token = &self.tokens[self.position];
            if !matches!(token, Token::Newline | Token::Semicolon) {
                condition_tokens.push(token.clone());
            }
            self.position += 1;
        }

        if self.position >= self.tokens.len() {
            return Err(ParseError::Incomplete);
        }
        if condition_tokens.is_empty() {
            return Err(ParseError::Syntax("if requires a condition".to_string()));
        }

        self.position += 1; // then
        let condition_tokens: Vec<_> = condition_tokens
            .into_iter()
            .filter(|token| !matches!(token, Token::Newline | Token::Semicolon))
            .collect();
        let condition = parse_logic(&condition_tokens)
            .ok_or_else(|| ParseError::Syntax("invalid if condition".to_string()))?;

        let body = self.parse_sequence(&["elif", "else", "end"])?;
        Ok((condition, body.unwrap_or_else(empty_sequence)))
    }

    fn parse_simple_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self.position;
        while self.position < self.tokens.len() && !self.at_separator() {
            self.position += 1;
        }

        if matches!(
            self.tokens.get(self.position.saturating_sub(1)),
            Some(
                Token::Pipe
                    | Token::And
                    | Token::Or
                    | Token::RedirectIn
                    | Token::RedirectOut
                    | Token::AppendOut
            )
        ) {
            return Err(ParseError::Incomplete);
        }

        parse_logic(&self.tokens[start..self.position])
            .ok_or_else(|| ParseError::Syntax("expected a command".to_string()))
    }

    fn at_separator(&self) -> bool {
        matches!(
            self.tokens.get(self.position),
            Some(Token::Semicolon | Token::Newline)
        )
    }

    fn skip_separators(&mut self) {
        while self.at_separator() {
            self.position += 1;
        }
    }

    fn current_word_is(&self, expected: &str) -> bool {
        matches!(
            self.tokens.get(self.position),
            Some(Token::Word { value, quoted: false, .. }) if value == expected
        )
    }

    fn current_word_is_any(&self, expected: &[&str]) -> bool {
        expected.iter().any(|word| self.current_word_is(word))
    }
}

fn empty_sequence() -> Statement {
    Statement::Sequence(Vec::new())
}

fn parse_pipeline(tokens: &[Token]) -> Option<Statement> {
    let mut commands = Vec::new();
    let mut start = 0;

    for (index, token) in tokens.iter().enumerate() {
        if matches!(token, Token::Pipe) {
            commands.push(parse_command(&tokens[start..index])?);
            start = index + 1;
        }
    }

    commands.push(parse_command(&tokens[start..])?);

    if commands.len() == 1 {
        Some(Statement::Command(commands.remove(0)))
    } else {
        Some(Statement::Pipeline(Pipeline { commands }))
    }
}

fn parse_command(tokens: &[Token]) -> Option<Command> {
    let mut command = Command {
        program: String::new(),
        args: Vec::new(),
        stdin: None,
        stdout: None,
    };

    let mut index = 0;
    while index < tokens.len() {
        match &tokens[index] {
            Token::Word { value, expand, .. } => {
                let words = if *expand {
                    crate::expand::expand_word(value)
                } else {
                    vec![value.clone()]
                };

                for word in words {
                    if command.program.is_empty() {
                        command.program = word;
                    } else {
                        command.args.push(word);
                    }
                }
            }
            Token::RedirectOut => {
                index += 1;
                if let Some(Token::Word { value, .. }) = tokens.get(index) {
                    command.stdout = Some(Redirect::Write(value.clone()));
                }
            }
            Token::AppendOut => {
                index += 1;
                if let Some(Token::Word { value, .. }) = tokens.get(index) {
                    command.stdout = Some(Redirect::Append(value.clone()));
                }
            }
            Token::RedirectIn => {
                index += 1;
                if let Some(Token::Word { value, .. }) = tokens.get(index) {
                    command.stdin = Some(value.clone());
                }
            }
            _ => {}
        }
        index += 1;
    }

    (!command.program.is_empty()).then_some(command)
}

fn parse_logic(tokens: &[Token]) -> Option<Statement> {
    if let Some(index) = tokens.iter().rposition(|token| matches!(token, Token::Or)) {
        let left = parse_logic(&tokens[..index])?;
        let right = parse_logic(&tokens[index + 1..])?;
        return Some(Statement::Or(Box::new(left), Box::new(right)));
    }

    if let Some(index) = tokens.iter().rposition(|token| matches!(token, Token::And)) {
        let left = parse_logic(&tokens[..index])?;
        let right = parse_logic(&tokens[index + 1..])?;
        return Some(Statement::And(Box::new(left), Box::new(right)));
    }

    parse_pipeline(tokens)
}
