use std::{cell::RefCell, collections::HashMap, path::Path, rc::Rc};

use colored::Colorize;
use penguin::prelude::*;

#[derive(Clone, Default)]
pub struct PengErrorSources {
    sources: Rc<RefCell<HashMap<usize, String>>>,
}

impl PengErrorSources {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_path(&self, id: usize, path: &Path) {
        self.sources
            .borrow_mut()
            .insert(id, path.to_string_lossy().into_owned());
    }

    pub fn get_or_insert_path(&self, path: &Path) -> usize {
        let path = path.to_string_lossy().into_owned();

        {
            let sources = self.sources.borrow();

            if let Some((id, _)) = sources.iter().find(|(_, source)| **source == path) {
                return *id;
            }
        }

        let mut sources = self.sources.borrow_mut();
        let id = sources.keys().max().map(|id| id + 1).unwrap_or(0);

        sources.insert(id, path);

        id
    }

    fn get(&self, id: usize) -> Option<String> {
        self.sources.borrow().get(&id).cloned()
    }
}

pub fn format_peng_error(error: PengError) -> String {
    format_peng_error_impl(error, None)
}

pub fn format_peng_error_with_sources(error: PengError, sources: &PengErrorSources) -> String {
    format_peng_error_impl(error, Some(sources))
}

fn format_peng_error_impl(error: PengError, sources: Option<&PengErrorSources>) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "{} {}\n",
        "[error]".red().bold(),
        error_title(&error).bold()
    ));

    render_error(&error, &mut output, 0, sources);

    output.trim_end().to_string()
}

fn render_error(
    error: &PengError,
    output: &mut String,
    indent: usize,
    sources: Option<&PengErrorSources>,
) {
    match error {
        PengError::Stack(errors) => {
            push_line(output, indent, "trace:");

            for (index, error) in errors.iter().enumerate() {
                push_line(
                    output,
                    indent + 1,
                    &format!("{}. {}", index + 1, error_title(error)),
                );
                render_error_details(error, output, indent + 2, sources);
            }
        }

        PengError::PositionedError { error, position } => {
            push_line(
                output,
                indent,
                &format!("at {}", format_position(position, sources)),
            );
            render_error(error, output, indent, sources);
        }

        PengError::Raised(error) => {
            push_line(output, indent, "raised error:");
            render_error(error, output, indent + 1, sources);
        }

        _ => render_error_details(error, output, indent, sources),
    }
}

fn render_error_details(
    error: &PengError,
    output: &mut String,
    indent: usize,
    sources: Option<&PengErrorSources>,
) {
    match error {
        PengError::Stack(_) | PengError::PositionedError { .. } | PengError::Raised(_) => {
            render_error(error, output, indent, sources);
        }

        PengError::Position(position) => {
            push_line(
                output,
                indent,
                &format!("at {}", format_position(position, sources)),
            );
        }

        PengError::PositionedMessage { message, position } => {
            push_line(output, indent, message);
            push_line(
                output,
                indent,
                &format!("at {}", format_position(position, sources)),
            );
        }

        PengError::TypeMismatch { expected, found } => {
            push_line(output, indent, &format!("expected: {expected}"));
            push_line(output, indent, &format!("found: {found}"));
        }

        PengError::InvalidConversion { from, to } => {
            push_line(output, indent, &format!("from: {}", compact_debug(from)));
            push_line(output, indent, &format!("to: {}", compact_debug(to)));
        }

        PengError::InvalidUnaryOperation { operator, operand } => {
            push_line(output, indent, &format!("operator: {operator}"));
            push_line(output, indent, &format!("operand: {operand}"));
        }

        PengError::InvalidBinaryOperationCell {
            operator,
            left,
            right,
        } => {
            push_line(output, indent, &format!("operator: {operator}"));
            push_line(output, indent, &format!("left: {}", compact_debug(left)));
            push_line(output, indent, &format!("right: {}", compact_debug(right)));
        }

        PengError::InvalidBinaryOperationValue {
            operator,
            left,
            right,
        } => {
            push_line(output, indent, &format!("operator: {operator}"));
            push_line(output, indent, &format!("left: {}", compact_debug(left)));
            push_line(output, indent, &format!("right: {}", compact_debug(right)));
        }

        PengError::IndexOutOfBounds { index, len } => {
            push_line(output, indent, &format!("index: {index}"));
            push_line(output, indent, &format!("length: {len}"));
        }

        PengError::WrongArgumentCount { expected, found }
        | PengError::TooManyArguments { expected, found } => {
            push_line(output, indent, &format!("expected: {expected}"));
            push_line(output, indent, &format!("found: {found}"));
        }

        PengError::ProgramCounterOutOfBounds { pc, len } => {
            push_line(output, indent, &format!("program counter: {pc}"));
            push_line(output, indent, &format!("instruction count: {len}"));
        }

        PengError::ExpectedToken { expected, found } => {
            push_line(output, indent, &format!("expected: {expected}"));
            push_line(output, indent, &format!("found: {found}"));
        }

        PengError::UserError(values) => {
            if values.is_empty() {
                push_line(output, indent, "user error without values");
            } else {
                push_line(output, indent, "values:");
                for value in values {
                    push_line(output, indent + 1, &compact_debug(value));
                }
            }
        }

        _ => {
            push_line(output, indent, &error_message(error));
        }
    }
}

fn error_title(error: &PengError) -> String {
    match error {
        PengError::Stack(_) => "multiple errors".to_string(),
        PengError::Position(_) => "source position".to_string(),
        PengError::PositionedMessage { message, .. } => message.clone(),
        PengError::PositionedError { error, .. } => error_title(error),
        PengError::Raised(error) => format!("raised {}", error_title(error)),
        _ => error_message(error),
    }
}

fn error_message(error: &PengError) -> String {
    match error {
        PengError::Stack(errors) => format!("{} errors were reported", errors.len()),
        PengError::Position(position) => format!("position {}", format_position(position, None)),
        PengError::PositionedMessage { message, .. } => message.clone(),
        PengError::PositionedError { error, .. } => error_message(error),
        PengError::InternalError(message) => format!("internal error: {message}"),
        PengError::NotImplemented(message) => format!("not implemented: {message}"),
        PengError::InvalidState(message) => format!("invalid state: {message}"),
        PengError::NameNotFound(name) => format!("name not found: {}", format_name(*name)),
        PengError::NameAlreadyDefined(name) => {
            format!("name already defined: {}", format_name(*name))
        }
        PengError::LocalNotFound(index) => format!("local not found: #{index}"),
        PengError::GlobalNotFound(name) => format!("global not found: {}", format_name(*name)),
        PengError::AttributeNotFound(name) => {
            format!("attribute not found: {}", format_name(*name))
        }
        PengError::AttributeAlreadyDefined(name) => {
            format!("attribute already defined: {}", format_name(*name))
        }
        PengError::HeapValueNotFound(ptr) => format!("heap value not found: {}", format_ptr(*ptr)),
        PengError::InvalidReference(ptr) => format!("invalid reference: {}", format_ptr(*ptr)),
        PengError::ExpectedReference => "expected a reference".to_string(),
        PengError::DanglingReference(ptr) => format!("dangling reference: {}", format_ptr(*ptr)),
        PengError::CannotAssignImmutable => "cannot assign to immutable value".to_string(),
        PengError::CannotMutateImmutable => "cannot mutate immutable value".to_string(),
        PengError::CannotReadUninitialized => "cannot read uninitialized value".to_string(),
        PengError::CannotAssignUninitialized => "cannot assign uninitialized value".to_string(),
        PengError::ExpectedInitialized => "expected initialized value".to_string(),
        PengError::ExpectedMutable => "expected mutable value".to_string(),
        PengError::ExpectedImmutable => "expected immutable value".to_string(),
        PengError::TypeMismatch { .. } => "type mismatch".to_string(),
        PengError::InvalidConversion { .. } => "invalid conversion".to_string(),
        PengError::CannotInferType => "cannot infer type".to_string(),
        PengError::ExpectedType => "expected a type".to_string(),
        PengError::ExpectedValue => "expected a value".to_string(),
        PengError::ExpectedFunction => "expected a function".to_string(),
        PengError::ExpectedThread => "expected a thread".to_string(),
        PengError::ExpectedObject => "expected an object".to_string(),
        PengError::ExpectedVector => "expected a vector".to_string(),
        PengError::ExpectedModule => "expected a module".to_string(),
        PengError::ExpectedOperation => "expected an operation".to_string(),
        PengError::ExpectedNumber => "expected a number".to_string(),
        PengError::InvalidUnaryOperation { .. } => "invalid unary operation".to_string(),
        PengError::InvalidBinaryOperationCell { .. }
        | PengError::InvalidBinaryOperationValue { .. } => "invalid binary operation".to_string(),
        PengError::DivisionByZero => "division by zero".to_string(),
        PengError::ArithmeticOverflow => "arithmetic overflow".to_string(),
        PengError::ArithmeticUnderflow => "arithmetic underflow".to_string(),
        PengError::NegativeUnsignedResult => {
            "unsigned operation produced a negative result".to_string()
        }
        PengError::IndexOutOfBounds { .. } => "index out of bounds".to_string(),
        PengError::InvalidIndexTypeValue(value) => {
            format!("invalid index type: {}", compact_debug(value))
        }
        PengError::CannotIndexValue(value) => format!("cannot index value: {value}"),
        PengError::CannotSetIndex(value) => format!("cannot set index on value: {value}"),
        PengError::WrongArgumentCount { .. } => "wrong argument count".to_string(),
        PengError::TooManyArguments { .. } => "too many arguments".to_string(),
        PengError::CannotCallValue(value) => format!("cannot call value: {value}"),
        PengError::ReturnOutsideFunction => "return outside function".to_string(),
        PengError::MissingReturnValue => "missing return value".to_string(),
        PengError::StackUnderflow => "stack underflow".to_string(),
        PengError::StackOverflow => "stack overflow".to_string(),
        PengError::EmptyStack => "empty stack".to_string(),
        PengError::FrameNotFound => "frame not found".to_string(),
        PengError::EmptyFrameStack => "empty frame stack".to_string(),
        PengError::InvalidFrame => "invalid frame".to_string(),
        PengError::ProgramCounterOutOfBounds { .. } => "program counter out of bounds".to_string(),
        PengError::InstructionExpectedConstant => "instruction expected a constant".to_string(),
        PengError::InvalidInstruction(instruction) => {
            format!("invalid instruction: {}", compact_debug(instruction))
        }
        PengError::ThreadNotFound(ptr) => format!("thread not found: {}", format_ptr(*ptr)),
        PengError::CurrentThreadNotFound => "current thread not found".to_string(),
        PengError::SyntaxError(message) => format!("syntax error: {message}"),
        PengError::UnexpectedToken(token) => format!("unexpected token: {}", compact_debug(token)),
        PengError::ExpectedToken { .. } => "expected another token".to_string(),
        PengError::InvalidAssignmentTarget => "invalid assignment target".to_string(),
        PengError::InvalidLValue => "invalid left-hand value".to_string(),
        PengError::BreakOutsideLoop => "break outside loop".to_string(),
        PengError::ContinueOutsideLoop => "continue outside loop".to_string(),
        PengError::RaiseOutsideTry => "raise outside try".to_string(),
        PengError::Raised(error) => format!("raised {}", error_message(error)),
        PengError::UserError(values) => format!("user error with {} value(s)", values.len()),
    }
}

fn format_position(position: &PengPosition, sources: Option<&PengErrorSources>) -> String {
    let source = sources
        .and_then(|sources| sources.get(position.id))
        .unwrap_or_else(|| format!("source #{}", position.id));

    match position.column {
        Some(column) => format!("{source}, line {}, column {}", position.line, column),
        None => format!("{source}, line {}", position.line),
    }
}

fn format_ptr(ptr: PengHeapPtr) -> String {
    format!("#{}", ptr.0)
}

fn format_name(name: PengNamePoolPtr) -> String {
    format!("#{}", name.0)
}

fn compact_debug(value: &impl std::fmt::Debug) -> String {
    format!("{value:?}")
}

fn push_line(output: &mut String, indent: usize, line: &str) {
    output.push_str(&"  ".repeat(indent));
    output.push_str(line);
    output.push('\n');
}
