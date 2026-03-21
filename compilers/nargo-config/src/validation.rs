/// Configuration validation error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Package name is missing or invalid.
    MissingPackageName,

    /// Package version is missing or invalid.
    MissingPackageVersion,

    /// Invalid version format.
    InvalidVersion(String),

    /// Invalid dependency specification.
    InvalidDependency(String),

    /// Circular dependency detected.
    CircularDependency(String),

    /// Path dependency does not exist.
    PathNotFound(String),

    /// Git dependency without URL.
    GitWithoutUrl(String),

    /// Conflicting dependency sources.
    ConflictingSources(String),

    /// Invalid package name format.
    InvalidPackageName(String),

    /// Unknown registry reference.
    UnknownRegistry(String),

    /// Invalid target specification.
    InvalidTarget(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MissingPackageName => write!(f, "Package name is required"),
            ValidationError::MissingPackageVersion => write!(f, "Package version is required"),
            ValidationError::InvalidVersion(v) => write!(f, "Invalid version format: {}", v),
            ValidationError::InvalidDependency(d) => write!(f, "Invalid dependency: {}", d),
            ValidationError::CircularDependency(d) => {
                write!(f, "Circular dependency detected: {}", d)
            }
            ValidationError::PathNotFound(p) => write!(f, "Dependency path not found: {}", p),
            ValidationError::GitWithoutUrl(d) => write!(f, "Git dependency missing URL: {}", d),
            ValidationError::ConflictingSources(d) => {
                write!(f, "Dependency has conflicting sources: {}", d)
            }
            ValidationError::InvalidPackageName(n) => write!(f, "Invalid package name: {}", n),
            ValidationError::UnknownRegistry(r) => write!(f, "Unknown registry: {}", r),
            ValidationError::InvalidTarget(t) => write!(f, "Invalid target specification: {}", t),
        }
    }
}

impl std::error::Error for ValidationError {}
