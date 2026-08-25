use std::fmt;
use std::path::PathBuf;

/// A stable identifier for one validation failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DiagnosticCode {
    // RDOC001-RDOC003 are reserved after retiring the standalone JSON configuration.
    /// Reserved after removal of the standalone configuration schema.
    UnsupportedSchema,
    /// Reserved after removal of the standalone configuration file.
    ConfigRead,
    /// Reserved after removal of the standalone configuration parser.
    ConfigParse,
    /// The root Registry manifest could not be read.
    RegistryManifestRead,
    /// The root Registry manifest is malformed.
    RegistryManifestParse,
    /// The root Registry manifest has no members.
    EmptyRegistry,
    /// A Registry member or invocation-relative path is invalid.
    InvalidMemberPath,
    /// Two Registry members resolve to the same Rust module name.
    DuplicateMemberModule,
    /// A Component manifest could not be read.
    ComponentManifestRead,
    /// A Component manifest is malformed.
    ComponentManifestParse,
    /// A Component install/page name is invalid.
    InvalidComponentName,
    /// Component source could not be read.
    ComponentSourceRead,
    /// Component source is malformed Rust.
    ComponentSourceParse,
    /// Component source has no public `#[component]` function.
    MissingPublicComponent,
    /// A Component README could not be read.
    ReadmeRead,
    /// A Component README lacks the required heading and introduction.
    MalformedReadme,
    /// A declared Example source file could not be read.
    ExampleRead,
    /// A stable Component group ID is invalid.
    InvalidGroupId,
    /// A stable Component group ID is declared more than once.
    DuplicateGroupId,
    /// A Component page declares no Examples.
    EmptyExamples,
    /// An Example slug is empty.
    EmptyExampleSlug,
    /// An Example slug is declared more than once on a page.
    DuplicateExampleSlug,
    /// A macro invocation source file could not be read.
    InvocationRead,
    /// A macro invocation source file or invocation is malformed.
    InvocationParse,
    /// The expected macro invocation is absent.
    MissingInvocation,
    /// A Component uses a group ID absent from the catalog mapping.
    UnknownGroup,
    /// A Component manifest does not exclude all authoring-only files.
    MissingExclude,
    /// A Component retains the superseded documentation layout.
    SupersededLayout,
    /// A Component has no `docs/mod.rs` invocation module.
    MissingDocsModule,
    /// A Component has no `docs/examples` directory.
    MissingExamplesDirectory,
    /// A custom page does not name a zero-prop Component.
    InvalidPage,
    /// The configured default Component is not a Registry member.
    InvalidDefault,
    /// Two Component manifests use the same page ID.
    DuplicatePageId,
    // RDOC034-RDOC035 are reserved after retiring section-pattern validation.
    /// Reserved after removal of required-section validation.
    MissingRequiredSection,
    /// Reserved after removal of section-pattern validation.
    SectionPatternMismatch,
    /// The catalog declares no stable group-ID mappings.
    EmptyGroupMapping,
    /// No Registry repository is available for installation instructions.
    MissingRegistryRepository,
    /// The Registry repository is not an accepted absolute URL.
    InvalidRegistryRepository,
    /// The Registry revision is unsafe for command generation.
    InvalidRegistryRevision,
    /// The root Registry manifest has an empty name.
    InvalidRegistryName,
}

impl DiagnosticCode {
    /// The stable, greppable external representation of this diagnostic.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedSchema => "RDOC001",
            Self::ConfigRead => "RDOC002",
            Self::ConfigParse => "RDOC003",
            Self::RegistryManifestRead => "RDOC004",
            Self::RegistryManifestParse => "RDOC005",
            Self::EmptyRegistry => "RDOC006",
            Self::InvalidMemberPath => "RDOC007",
            Self::DuplicateMemberModule => "RDOC008",
            Self::ComponentManifestRead => "RDOC009",
            Self::ComponentManifestParse => "RDOC010",
            Self::InvalidComponentName => "RDOC011",
            Self::ComponentSourceRead => "RDOC012",
            Self::ComponentSourceParse => "RDOC013",
            Self::MissingPublicComponent => "RDOC014",
            Self::ReadmeRead => "RDOC015",
            Self::MalformedReadme => "RDOC016",
            Self::ExampleRead => "RDOC017",
            Self::InvalidGroupId => "RDOC018",
            Self::DuplicateGroupId => "RDOC019",
            Self::EmptyExamples => "RDOC020",
            Self::EmptyExampleSlug => "RDOC021",
            Self::DuplicateExampleSlug => "RDOC022",
            Self::InvocationRead => "RDOC023",
            Self::InvocationParse => "RDOC024",
            Self::MissingInvocation => "RDOC025",
            Self::UnknownGroup => "RDOC026",
            Self::MissingExclude => "RDOC027",
            Self::SupersededLayout => "RDOC028",
            Self::MissingDocsModule => "RDOC029",
            Self::MissingExamplesDirectory => "RDOC030",
            Self::InvalidPage => "RDOC031",
            Self::InvalidDefault => "RDOC032",
            Self::DuplicatePageId => "RDOC033",
            Self::MissingRequiredSection => "RDOC034",
            Self::SectionPatternMismatch => "RDOC035",
            Self::EmptyGroupMapping => "RDOC036",
            Self::MissingRegistryRepository => "RDOC037",
            Self::InvalidRegistryRepository => "RDOC038",
            Self::InvalidRegistryRevision => "RDOC039",
            Self::InvalidRegistryName => "RDOC040",
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// The macro input token that should receive a core diagnostic.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum DiagnosticTarget {
    /// The complete macro invocation.
    #[default]
    Invocation,
    /// The `component_root` value.
    ComponentRoot,
    /// The Component's `group` value.
    ComponentGroup,
    /// One Example's slug, identified by declaration index.
    ExampleSlug(usize),
    /// The catalog's root manifest path.
    CatalogManifest,
    /// One catalog group mapping, identified by declaration index.
    CatalogGroup(usize),
    /// The catalog's default Component.
    CatalogDefault,
    /// The catalog's Registry repository.
    CatalogRepository,
    /// The catalog's Registry revision.
    CatalogRevision,
}

/// One validation failure with a stable code and an optional source path.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Diagnostic {
    /// The stable external failure code.
    pub code: DiagnosticCode,
    /// An actionable description of the failure.
    pub message: String,
    /// The authoring file associated with the failure, when known.
    pub path: Option<PathBuf>,
    /// The macro token that should receive the diagnostic.
    pub target: DiagnosticTarget,
}

impl Diagnostic {
    /// Creates a diagnostic without a source path.
    pub fn new(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            path: None,
            target: DiagnosticTarget::Invocation,
        }
    }

    /// Associates this diagnostic with an authoring path.
    #[must_use]
    pub fn at_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub(crate) fn for_target(mut self, target: DiagnosticTarget) -> Self {
        self.target = target;
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = &self.path {
            write!(formatter, "{}: ", path.display())?;
        }

        write!(formatter, "error[{}]: {}", self.code, self.message)
    }
}

impl std::error::Error for Diagnostic {}

/// A possibly partial value and every diagnostic observed while producing it.
///
/// Full-site loading may retain a partial value alongside diagnostics so callers can apply
/// additional policy. Component and catalog loaders return no value after a validation failure.
#[must_use = "validation diagnostics and any partial value must be inspected"]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Validation<T> {
    /// The complete or partial value produced by the operation.
    pub value: Option<T>,
    /// Every independent validation failure observed by the operation.
    pub diagnostics: Vec<Diagnostic>,
}

impl<T> Validation<T> {
    pub(crate) fn success(value: T) -> Self {
        Self {
            value: Some(value),
            diagnostics: Vec::new(),
        }
    }

    pub(crate) fn failure(diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            value: None,
            diagnostics,
        }
    }
}
