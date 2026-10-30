#[derive(Debug, Clone)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,

    pub stdin: Option<String>,
    pub stdout: Option<Redirect>,
}

#[derive(Debug, Clone)]
pub enum Redirect {
    Write(String),  // >
    Append(String), // >>
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Command(Command),

    Pipeline(Pipeline),

    Sequence(Vec<Statement>),

    And(Box<Statement>, Box<Statement>),

    Or(Box<Statement>, Box<Statement>),

    If {
        branches: Vec<(Statement, Statement)>,
        else_branch: Option<Box<Statement>>,
    },
}

#[derive(Debug, Clone)]
pub struct Word {
    pub value: String,

    pub expand: bool,
}
