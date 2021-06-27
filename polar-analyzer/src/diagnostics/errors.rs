use polar_core::error::{ErrorKind, ParseError};

pub type ErrorData = (String, usize, usize);

pub fn find_parse_errors(src: &str) -> Vec<ErrorData> {
    let parse_result = polar_core::parser::parse_file_with_errors(0, src);

    match parse_result {
        Ok((_, errors)) => errors,
        Err(e) => match e.kind {
            ErrorKind::Parse(e) => match e {
                ParseError::IntegerOverflow { loc, .. }
                | ParseError::InvalidTokenCharacter { loc, .. }
                | ParseError::InvalidToken { loc, .. }
                | ParseError::UnrecognizedEOF { loc }
                | ParseError::UnrecognizedToken { loc, .. }
                | ParseError::ExtraToken { loc, .. }
                | ParseError::WrongValueType { loc, .. }
                | ParseError::ReservedWord { loc, .. } => {
                    vec![(e.to_string(), loc, loc)]
                }
                _ => {
                    vec![(e.to_string(), 0, 0)]
                }
            },
            _ => vec![(e.to_string(), 0, 0)],
        },
    }
}
