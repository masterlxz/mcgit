#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JavaSource {
    Managed,
    Detected,
    Manual,
}

impl JavaSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            JavaSource::Managed => "managed",
            JavaSource::Detected => "detected",
            JavaSource::Manual => "manual",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "managed" => Some(JavaSource::Managed),
            "detected" => Some(JavaSource::Detected),
            "manual" => Some(JavaSource::Manual),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaInstallation {
    pub id: i64,
    pub major_version: u32,
    pub vendor: String,
    pub path: String,
    pub source: JavaSource,
    pub is_default: bool,
    pub created_at: String,
}

/// What's needed to insert (or, if `path` already exists, update) a row —
/// `id`, `is_default`, and `created_at` are assigned by the database.
#[derive(Debug, Clone)]
pub struct NewJavaInstallation {
    pub major_version: u32,
    pub vendor: String,
    pub path: String,
    pub source: JavaSource,
}
