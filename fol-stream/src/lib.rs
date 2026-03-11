// FOL Stream - Sophisticated file/folder to character stream conversion
// Handles .mod directories specially and creates unified character streams

use colored::Colorize;
use fol_types::*;
use regex::Regex;
use std::ffi::OsStr;
use std::path::Path;

/// Character provider trait for streaming characters with location info
pub trait CharacterProvider {
    fn next_char(&mut self) -> Option<(char, Location)>;
}

/// Stream source trait for converting paths to character providers
pub trait StreamSource {
    type Provider: CharacterProvider;
    fn into_provider(self) -> Result<Self::Provider, Box<dyn Glitch>>;
}

/// Location tracking for characters in source files
#[derive(Debug, Clone)]
pub struct Location {
    pub row: usize,
    pub col: usize,
    pub file: Option<String>,
}

/// Source file representation with namespace information
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Source {
    pub call: String,      // Original call path
    pub path: String,      // Absolute file path
    pub data: String,      // File contents
    pub namespace: String, // Namespace path (e.g., "math::vector")
    pub package: String,   // Package name (root namespace)
}

/// Source type for path validation
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SourceType {
    File,
    Folder,
}

impl Source {
    /// Initialize sources from input path with package name
    pub fn init(input: &str, source_type: SourceType) -> Result<Vec<Self>, Box<dyn Glitch>> {
        let package_name = detect_package_name(input)?;
        source(input, source_type, &package_name)
    }

    /// Initialize sources with explicit package name
    pub fn init_with_package(
        input: &str,
        source_type: SourceType,
        package_name: &str,
    ) -> Result<Vec<Self>, Box<dyn Glitch>> {
        source(input, source_type, package_name)
    }

    /// Get the full or relative path
    pub fn path(&self, abs: bool) -> String {
        if abs {
            self.path.clone()
        } else {
            self.rel_path()
        }
    }

    fn rel_path(&self) -> String {
        std::fs::canonicalize(&self.path)
            .unwrap_or_else(|_| self.path.clone().into())
            .as_path()
            .to_str()
            .unwrap_or(&self.path)
            .trim_start_matches(&self.abs_path())
            .to_string()
    }

    fn abs_path(&self) -> String {
        std::fs::canonicalize(&self.call)
            .unwrap_or_else(|_| self.call.clone().into())
            .as_path()
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_str()
            .unwrap_or(".")
            .to_string()
    }

    pub fn module(&self) -> String {
        std::fs::canonicalize(&self.path)
            .unwrap_or_else(|_| self.path.clone().into())
            .as_path()
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_str()
            .unwrap_or(".")
            .trim_start_matches(&self.abs_path())
            .to_string()
    }
}

/// Multi-source file stream for handling complex folder structures
pub struct FileStream {
    sources: Vec<Source>,
    current_source: usize,
    position: usize,
    current_chars: Vec<char>,
    location: Location,
}

impl FileStream {
    /// Create from a single file path
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Glitch>> {
        let sources = Source::init(path, SourceType::File)?;
        Self::from_sources(sources)
    }

    /// Create from a folder path (with sophisticated .mod handling)
    pub fn from_folder(path: &str) -> Result<Self, Box<dyn Glitch>> {
        let sources = Source::init(path, SourceType::Folder)?;
        Self::from_sources(sources)
    }

    /// Create from a list of sources
    pub fn from_sources(mut sources: Vec<Source>) -> Result<Self, Box<dyn Glitch>> {
        if sources.is_empty() {
            return Err(Box::new(BasicError {
                message: "No sources provided".to_string(),
            }));
        }

        // Load file contents for all sources
        for source in &mut sources {
            source.data =
                std::fs::read_to_string(&source.path).map_err(|e| -> Box<dyn Glitch> {
                    Box::new(BasicError {
                        message: format!("Failed to read file {}: {}", source.path, e),
                    })
                })?;
        }

        let first_file = sources[0].path.clone();
        let first_chars: Vec<char> = sources[0].data.chars().collect();
        Ok(Self {
            sources,
            current_source: 0,
            position: 0,
            current_chars: first_chars,
            location: Location {
                row: 1,
                col: 1,
                file: Some(first_file),
            },
        })
    }

    /// Get current source being processed
    pub fn current_source(&self) -> Option<&Source> {
        self.sources.get(self.current_source)
    }

    /// Get all sources
    pub fn sources(&self) -> &[Source] {
        &self.sources
    }
}

impl CharacterProvider for FileStream {
    fn next_char(&mut self) -> Option<(char, Location)> {
        // Check if we need to move to next source
        while self.current_source < self.sources.len() {
            if self.position >= self.current_chars.len() {
                // Move to next source
                self.current_source += 1;
                self.position = 0;

                if self.current_source < self.sources.len() {
                    self.current_chars = self.sources[self.current_source].data.chars().collect();
                    // Update location for new file
                    self.location = Location {
                        row: 1,
                        col: 1,
                        file: Some(self.sources[self.current_source].path.clone()),
                    };
                }
                continue;
            }

            // Get character from current position
            let ch = self.current_chars[self.position];
            let loc = self.location.clone();

            self.position += 1;
            if ch == '\n' {
                self.location.row += 1;
                self.location.col = 1;
            } else {
                self.location.col += 1;
            }

            return Some((ch, loc));
        }

        None
    }
}

/// Create sources from input path and type with namespace support
fn source(
    input: &str,
    source_type: SourceType,
    package_name: &str,
) -> Result<Vec<Source>, Box<dyn Glitch>> {
    let mut sources = Vec::new();
    let validated_path = check_validity(input, source_type.clone())?;

    match source_type {
        SourceType::File => {
            let namespace = compute_namespace(&validated_path, &validated_path, package_name)?;
            sources.push(Source {
                call: input.to_string(),
                path: validated_path,
                data: String::new(), // Will be loaded later
                namespace,
                package: package_name.to_string(),
            });
        }
        SourceType::Folder => {
            for file_path in from_dir(&validated_path)? {
                let namespace = compute_namespace(&file_path, &validated_path, package_name)?;
                sources.push(Source {
                    call: input.to_string(),
                    path: file_path,
                    data: String::new(), // Will be loaded later
                    namespace,
                    package: package_name.to_string(),
                });
            }
        }
    }

    Ok(sources)
}

/// Sophisticated directory traversal with .mod handling
///
/// Key innovation: .mod directories are SKIPPED during traversal!
/// This allows for modular organization where .mod directories contain
/// mixed file types and module-specific code that's handled separately.
fn from_dir(directory: &str) -> Result<Vec<String>, Box<dyn Glitch>> {
    let paths = std::fs::read_dir(directory).map_err(|e| -> Box<dyn Glitch> {
        Box::new(BasicError {
            message: format!("Cannot read directory {}: {}", directory, e),
        })
    })?;
    let mut entries = paths
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| -> Box<dyn Glitch> {
            Box::new(BasicError {
                message: format!("Cannot read directory entry: {}", e),
            })
        })?;
    entries.sort_by(|left, right| {
        left.file_name()
            .to_string_lossy()
            .cmp(&right.file_name().to_string_lossy())
    });

    let mut files = Vec::new();

    for entry in entries {
        let filepath = entry.path().to_string_lossy().to_string();
        let filename = entry.file_name().to_string_lossy().to_string();

        if entry.path().is_dir() {
            // CRITICAL: Skip .mod directories - this is the key innovation!
            // .mod directories contain module-specific mixed content
            if Regex::new(r"(\.mod)$").unwrap().is_match(&filename) {
                continue;
            }

            // Recursively process non-.mod directories
            if let Ok(recursive_files) = from_dir(&filepath) {
                files.extend(recursive_files);
            }
        } else {
            // Only include .fol files
            if let Some(extension) = entry.path().extension().and_then(OsStr::to_str) {
                if extension == "fol" {
                    files.push(filepath);
                }
            }
        }
    }

    if files.is_empty() {
        let msg = format!("{}", "No .fol files found".red());
        Err(Box::new(BasicError { message: msg }))
    } else {
        Ok(files)
    }
}

/// Validate input path and return canonical path
fn check_validity(input: &str, source_type: SourceType) -> Result<String, Box<dyn Glitch>> {
    let path = Path::new(input);

    if !path.exists() {
        let msg = format!("Path {} does not exist", input.red());
        return Err(Box::new(BasicError { message: msg }));
    }

    let canonical_path = std::fs::canonicalize(path)
        .map_err(|e| -> Box<dyn Glitch> {
            Box::new(BasicError {
                message: format!("Cannot resolve path {}: {}", input, e),
            })
        })?
        .to_string_lossy()
        .to_string();

    match (path.is_dir(), source_type) {
        (true, SourceType::Folder) => Ok(canonical_path),
        (false, SourceType::File) => Ok(canonical_path),
        (false, SourceType::Folder) => {
            // If file provided but folder expected, use parent directory
            if let Some(parent) = path.parent() {
                let parent_canonical = std::fs::canonicalize(parent)
                    .map_err(|e| -> Box<dyn Glitch> {
                        Box::new(BasicError {
                            message: format!("Cannot resolve parent path: {}", e),
                        })
                    })?
                    .to_string_lossy()
                    .to_string();
                Ok(parent_canonical)
            } else {
                let msg = format!("Invalid path for folder source: {}", input.red());
                Err(Box::new(BasicError { message: msg }))
            }
        }
        (true, SourceType::File) => {
            let msg = format!("Expected file but got directory: {}", input.red());
            Err(Box::new(BasicError { message: msg }))
        }
    }
}

/// Create iterator over sources from input path
pub fn sources(input: String, source_type: SourceType) -> impl Iterator<Item = Source> {
    let sources = Source::init(&input, source_type).unwrap_or_default();
    sources.into_iter()
}

/// Detect package name from path by looking for Cargo.toml
fn detect_package_name(input_path: &str) -> Result<String, Box<dyn Glitch>> {
    let path = std::path::Path::new(input_path);
    let fallback_root = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    let fallback_name = fallback_root
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .or_else(|| {
            path.file_stem()
                .and_then(|name| name.to_str())
                .filter(|name| !name.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "unknown".to_string());
    let mut current = fallback_root;

    // Walk up the directory tree looking for Cargo.toml
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            // Parse Cargo.toml to get package name
            let content = std::fs::read_to_string(&cargo_toml).map_err(|e| -> Box<dyn Glitch> {
                Box::new(BasicError {
                    message: format!("Cannot read Cargo.toml: {}", e),
                })
            })?;

            // Simple parsing - look for name = "..." line
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("name") && line.contains('=') {
                    if let Some(name_part) = line.split('=').nth(1) {
                        let name = name_part.trim().trim_matches('"').trim_matches('\'');
                        return Ok(name.to_string());
                    }
                }
            }

            return Err(Box::new(BasicError {
                message: "Could not find package name in Cargo.toml".to_string(),
            }));
        }

        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }

    Ok(fallback_name)
}

/// Compute namespace from file path relative to project root
fn compute_namespace(
    file_path: &str,
    root_path: &str,
    package_name: &str,
) -> Result<String, Box<dyn Glitch>> {
    let file_path = std::path::Path::new(file_path);
    let root_path = std::path::Path::new(root_path);

    // Get the directory containing the file (not the file itself)
    let file_dir = file_path.parent().unwrap_or(root_path);

    // Compute relative path from root to file directory
    let relative_path = if file_dir == root_path {
        // File is in root directory
        None
    } else {
        // Try to get relative path
        file_dir.strip_prefix(root_path).ok()
    };

    match relative_path {
        None => {
            // File is in root directory -> root namespace (package name)
            Ok(package_name.to_string())
        }
        Some(rel_path) => {
            // File is in subdirectory -> build namespace path
            let mut namespace_parts = vec![package_name.to_string()];

            for component in rel_path.components() {
                if let Some(name) = component.as_os_str().to_str() {
                    // Skip .mod directories in namespace (they were already filtered out)
                    // Also validate that component names are valid identifiers
                    if !name.ends_with(".mod") && is_valid_namespace_component(name) {
                        namespace_parts.push(name.to_string());
                    }
                }
            }

            Ok(namespace_parts.join("::"))
        }
    }
}

/// Check if a name is a valid namespace component (no dots, valid identifier)
fn is_valid_namespace_component(name: &str) -> bool {
    // Namespace components should be valid identifiers (no dots or special chars except underscore)
    !name.is_empty()
        && !name.contains('.')
        && name.chars().all(|c| c.is_alphanumeric() || c == '_')
        && !name.chars().next().unwrap_or('0').is_ascii_digit() // Don't start with digit
}
