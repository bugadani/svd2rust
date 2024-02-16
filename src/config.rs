use anyhow::{bail, Result};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

#[cfg_attr(feature = "serde", derive(serde::Deserialize), serde(default))]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
#[non_exhaustive]
pub struct Config {
    pub target: Target,
    pub atomics: bool,
    pub atomics_feature: Option<String>,
    pub generic_mod: bool,
    pub make_mod: bool,
    pub skip_crate_attributes: bool,
    pub ignore_groups: bool,
    pub keep_list: bool,
    pub strict: bool,
    pub feature_group: bool,
    pub feature_peripheral: bool,
    pub max_cluster_size: bool,
    pub impl_debug: bool,
    pub impl_debug_feature: Option<String>,
    pub impl_defmt: Option<String>,
    pub output_dir: Option<PathBuf>,
    pub input: Option<PathBuf>,
    pub source_type: SourceType,
    pub log_level: Option<String>,
    pub interrupt_link_section: Option<String>,
    pub reexport_core_peripherals: bool,
    pub reexport_interrupt: bool,
    pub ident_formats: IdentFormats,
    pub ident_formats_theme: IdentFormatsTheme,
    pub base_address_shift: u64,
}

#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Target {
    #[cfg_attr(feature = "serde", serde(rename = "cortex-m"))]
    #[default]
    CortexM,
    #[cfg_attr(feature = "serde", serde(rename = "msp430"))]
    Msp430,
    #[cfg_attr(feature = "serde", serde(rename = "riscv"))]
    RISCV,
    #[cfg_attr(feature = "serde", serde(rename = "xtensa-lx"))]
    XtensaLX,
    #[cfg_attr(feature = "serde", serde(rename = "mips"))]
    Mips,
    #[cfg_attr(feature = "serde", serde(rename = "none"))]
    None,
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Target::CortexM => "cortex-m",
            Target::Msp430 => "msp430",
            Target::RISCV => "riscv",
            Target::XtensaLX => "xtensa-lx",
            Target::Mips => "mips",
            Target::None => "none",
        })
    }
}

impl Target {
    pub fn parse(s: &str) -> Result<Self> {
        Ok(match s {
            "cortex-m" => Target::CortexM,
            "msp430" => Target::Msp430,
            "riscv" => Target::RISCV,
            "xtensa-lx" => Target::XtensaLX,
            "mips" => Target::Mips,
            "none" => Target::None,
            _ => bail!("unknown target {}", s),
        })
    }

    pub const fn all() -> &'static [Target] {
        use self::Target::*;
        &[CortexM, Msp430, RISCV, XtensaLX, Mips]
    }
}

#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(rename_all = "lowercase")
)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SourceType {
    #[default]
    Xml,
    #[cfg(feature = "yaml")]
    Yaml,
    #[cfg(feature = "json")]
    Json,
}

impl SourceType {
    /// Make a new [`SourceType`] from a given extension.
    pub fn from_extension(s: &str) -> Option<Self> {
        match s {
            "svd" | "xml" => Some(Self::Xml),
            #[cfg(feature = "yaml")]
            "yml" | "yaml" => Some(Self::Yaml),
            #[cfg(feature = "json")]
            "json" => Some(Self::Json),
            _ => None,
        }
    }
    pub fn from_path(path: &Path) -> Self {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(Self::from_extension)
            .unwrap_or_default()
    }
}

#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(rename_all = "lowercase")
)]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Case {
    #[default]
    Constant,
    Pascal,
    Snake,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize), serde(default))]
pub struct IdentFormat {
    // Ident case. `None` means don't change
    pub case: Option<Case>,
    pub prefix: String,
    pub suffix: String,
}

impl IdentFormat {
    pub fn case(mut self, case: Case) -> Self {
        self.case = Some(case);
        self
    }
    pub fn constant_case(mut self) -> Self {
        self.case = Some(Case::Constant);
        self
    }
    pub fn pascal_case(mut self) -> Self {
        self.case = Some(Case::Pascal);
        self
    }
    pub fn snake_case(mut self) -> Self {
        self.case = Some(Case::Snake);
        self
    }
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.into();
        self
    }
    pub fn suffix(mut self, suffix: &str) -> Self {
        self.suffix = suffix.into();
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize), serde(default))]
pub struct IdentFormats(HashMap<String, IdentFormat>);

impl IdentFormats {
    fn common() -> Self {
        Self(HashMap::from([
            ("field_accessor".into(), IdentFormat::default().snake_case()),
            (
                "register_accessor".into(),
                IdentFormat::default().snake_case(),
            ),
            (
                "enum_value_accessor".into(),
                IdentFormat::default().snake_case(),
            ),
            ("cluster".into(), IdentFormat::default().pascal_case()),
            (
                "cluster_accessor".into(),
                IdentFormat::default().snake_case(),
            ),
            ("register_mod".into(), IdentFormat::default().snake_case()),
            ("cluster_mod".into(), IdentFormat::default().snake_case()),
            ("peripheral_mod".into(), IdentFormat::default().snake_case()),
            (
                "peripheral_feature".into(),
                IdentFormat::default().snake_case(),
            ),
        ]))
    }

    pub fn new_theme() -> Self {
        let mut map = Self::common();

        map.extend([
            (
                "field_reader".into(),
                IdentFormat::default().pascal_case().suffix("R"),
            ),
            (
                "field_writer".into(),
                IdentFormat::default().pascal_case().suffix("W"),
            ),
            ("enum_name".into(), IdentFormat::default().pascal_case()),
            (
                "enum_write_name".into(),
                IdentFormat::default().pascal_case().suffix("WO"),
            ),
            ("enum_value".into(), IdentFormat::default().pascal_case()),
            ("interrupt".into(), IdentFormat::default()),
            ("register".into(), IdentFormat::default().pascal_case()),
            (
                "register_spec".into(),
                IdentFormat::default().pascal_case().suffix("Spec"),
            ),
            ("peripheral".into(), IdentFormat::default().pascal_case()),
            (
                "peripheral_singleton".into(),
                IdentFormat::default().snake_case(),
            ),
        ]);

        map
    }
    pub fn legacy_theme() -> Self {
        let mut map = Self::common();

        map.extend([
            (
                "field_reader".into(),
                IdentFormat::default().constant_case().suffix("_R"),
            ),
            (
                "field_writer".into(),
                IdentFormat::default().constant_case().suffix("_W"),
            ),
            (
                "enum_name".into(),
                IdentFormat::default().constant_case().suffix("_A"),
            ),
            (
                "enum_write_name".into(),
                IdentFormat::default().constant_case().suffix("_AW"),
            ),
            ("enum_value".into(), IdentFormat::default().constant_case()),
            ("interrupt".into(), IdentFormat::default().constant_case()),
            ("cluster".into(), IdentFormat::default().constant_case()),
            ("register".into(), IdentFormat::default().constant_case()),
            (
                "register_spec".into(),
                IdentFormat::default().constant_case().suffix("_SPEC"),
            ),
            ("peripheral".into(), IdentFormat::default().constant_case()),
            (
                "peripheral_singleton".into(),
                IdentFormat::default().constant_case(),
            ),
        ]);

        map
    }
}

impl Deref for IdentFormats {
    type Target = HashMap<String, IdentFormat>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for IdentFormats {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(rename_all = "lowercase")
)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IdentFormatsTheme {
    #[default]
    New,
    Legacy,
}
