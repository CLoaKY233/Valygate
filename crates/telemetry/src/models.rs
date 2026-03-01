/// The output format for log messages.
///
/// | Format    | Best For                        |
/// |-----------|---------------------------------|
/// | `Json`    | Production / log aggregators    |
/// | `Compact` | Development terminals           |
/// | `Pretty`  | Debugging with full context     |
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogFormat {
    Json,
    Compact,
    Pretty,
}

pub struct LogConfig {
    pub format: LogFormat,
    pub filter: String,
}
