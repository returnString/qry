use super::StackFrame;
use crate::lang::SourceLocation;

#[derive(Debug, Clone)]
pub struct Exception {
	pub message: String,
	pub location: SourceLocation,
	pub stack: Vec<StackFrame>,
}

fn location_for_stacktrace(location: &SourceLocation) -> String {
	match location {
		SourceLocation::User { line, file } => format!("{}:{}", file, line),
		SourceLocation::Native { line, file } => format!("native: {}:{}", file, line),
		SourceLocation::Unknown => "unknown".into(),
	}
}

impl std::fmt::Display for Exception {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		writeln!(f, "exception stacktrace:")?;

		for frame in &self.stack {
			writeln!(
				f,
				"  in {} ({})",
				frame.name,
				location_for_stacktrace(&frame.location)
			)?;
		}

		writeln!(
			f,
			"{} ({})",
			&self.message,
			location_for_stacktrace(&self.location)
		)?;

		Ok(())
	}
}
