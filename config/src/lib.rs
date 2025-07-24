//! `codedefender-config` provides the Rust data structures used for serializing and deserializing
//! CodeDefender YAML configuration files and analysis results. These structures are used by both
//! the CodeDefender CLI and its backend services.
//!
//! This crate is intended to be consumed by tools that integrate with or generate CodeDefender config files.

use serde::{Deserialize, Serialize};

/// Current supported YAML config version.
pub const YAML_CONFIG_VERSION: &str = "1.0.0";

/// Available SIMD extension types used by mutation engines.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MutationEngineExtension {
    /// Generic (no special SIMD usage)
    Generic,
    /// SSE-enabled
    SSE,
}

/// Supported PE environments.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PeEnvironment {
    /// User-mode PE (exe, dll)
    UserMode,
    /// Kernel-mode PE (sys)
    KernelMode,
    /// UEFI firmware image
    UEFI,
}

/// Configuration settings for lifting x86 instructions into IR.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LifterSettings {
    /// Whether to lift calls into IR.
    pub lift_calls: bool,
    /// Calling convention used for lifting, only `WindowsAbi`, and `Conservative` are supported.
    pub calling_convention: String,
    /// Max stack copy size in bytes when lifting.
    pub max_stack_copy_size: u32,
    /// Fallback: split on calls if lifting fails.
    pub split_on_calls_fallback: bool,
}

/// IR optimization settings.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OptimizationSettings {
    /// Enable constant propagation.
    pub constant_propagation: bool,
    /// Enable instruction combining.
    pub instruction_combine: bool,
    /// Enable dead code elimination.
    pub dead_code_elim: bool,
    /// Enable pruning of unused block parameters.
    pub prune_useless_block_params: bool,
    /// Number of optimization iterations to run.
    pub iterations: u32,
}

/// Assembler-level codegen settings.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssemblerSettings {
    /// Whether to shuffle basic blocks.
    pub shuffle_basic_blocks: bool,
    /// Instruction prefix to prepend to emitted instructions.
    pub instruction_prefix: String,
    /// Chance of randomly applying the prefix.
    pub random_prefix_chance: f64,
}

/// Compiler configuration (IR + codegen) for a profile.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CDCompilerSettings {
    /// Assembler settings.
    pub assembler_settings: AssemblerSettings,
    /// Optimization settings.
    pub optimization_settings: OptimizationSettings,
    /// IR lifter settings.
    pub lifter_settings: LifterSettings,
}

/// Fake PDB string settings to confuse debuggers.
#[derive(Debug, Serialize, Deserialize)]
pub struct FakePdbString {
    /// Whether the fake PDB string is enabled.
    pub enabled: bool,
    /// Value to emit as the fake PDB string.
    pub value: String,
}

/// Custom `.text` section name override.
#[derive(Debug, Serialize, Deserialize)]
pub struct CustomSectionName {
    /// Whether this feature is enabled.
    pub enabled: bool,
    /// Custom section name value.
    pub value: String,
}

/// Global obfuscation settings for the module.
#[derive(Debug, Serialize, Deserialize)]
pub struct CDModuleSettings {
    /// Whether to crash the IDA decompiler intentionally.
    pub ida_crasher: bool,
    /// Whether to enable IAT/Import protection.
    pub import_protection: bool,
    /// Obscure the entry point of the module with anti tamper and anti debug tactics
    pub obscure_entry_point: bool,
    /// Clear unwind information. makes it harder for attackers to locate functions, however
    /// structured exception handling will not work.
    pub clear_unwind_info: bool,
    /// Fake PDB string settings.
    pub fake_pdb_string: FakePdbString,
    /// Custom PE section name settings.
    pub custom_section_name: CustomSectionName,
}

/// Instruction-level semantics used in transformations.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Semantics {
    pub add: bool,
    pub sub: bool,
    pub and: bool,
    pub xor: bool,
    pub or: bool,
    pub not: bool,
    pub neg: bool,
}

/// Bit widths to apply transformations to.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BitWidths {
    pub bit8: bool,
    pub bit16: bool,
    pub bit32: bool,
    pub bit64: bool,
}

/// Configuration for the Loop Encode Semantics pass.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoopEncodeSemantics {
    /// Number of times to attempt transformation.
    pub iterations: u32,
    /// Percent chance to apply transformation (0–100).
    pub probability: u32,
    /// Instruction semantics to consider.
    pub semantics: Semantics,
    /// Bit widths to target.
    pub bitwidths: BitWidths,
}

/// Configuration for Mixed Boolean Arithmetic pass.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MixedBooleanArithmetic {
    pub iterations: u32,
    pub probability: u32,
    pub semantics: Semantics,
    pub bitwidths: BitWidths,
}

/// Configuration for Mutation Engine pass.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MutationEngine {
    pub iterations: u32,
    pub probability: u32,
    pub extension: MutationEngineExtension,
    pub semantics: Semantics,
    pub bitwidths: BitWidths,
}

/// Pass that crashes IDA’s decompiler.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IDADecompilerCrasher;

/// Constant obfuscation pass.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObscureConstants;

/// Memory reference obfuscation pass.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObscureReferences;

/// Control-flow obfuscation pass.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObscureControlFlow;

/// All possible obfuscation passes.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ObfuscationPass {
    LoopEncodeSemantics(LoopEncodeSemantics),
    MixedBooleanArithmetic(MixedBooleanArithmetic),
    MutationEngine(MutationEngine),
    IDADecompilerCrasher,
    ObscureConstants,
    ObscureReferences,
    ObscureControlFlow,
}

/// Profile definition used to apply passes to symbols.
#[derive(Debug, Serialize, Deserialize)]
pub struct CDProfile {
    /// Name of the profile.
    pub name: String,
    /// Obfuscation passes for this profile.
    pub passes: Vec<ObfuscationPass>,
    /// Compiler settings for this profile.
    pub compiler_settings: CDCompilerSettings,
    /// List of symbol RVAs this profile targets.
    pub symbols: Vec<u64>,
}

/// Top-level config file structure.
#[derive(Debug, Serialize, Deserialize)]
pub struct CDConfig {
    /// Module-wide settings.
    pub module_settings: CDModuleSettings,
    /// All profiles to apply during obfuscation.
    pub profiles: Vec<CDProfile>,
}

/// Information about a single function found during analysis.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AnalysisFunction {
    /// RVA of the function.
    pub rva: u64,
    /// Function name.
    pub symbol: String,
    /// Number of references to this function.
    pub ref_count: usize,
}

/// Reason why a function was rejected from analysis.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AnalysisReject {
    /// RVA of the rejected function.
    pub rva: u64,
    /// Symbol name.
    pub symbol: String,
    /// Mnemonic reason string (e.g., internal enum).
    pub ty: String,
    /// Stringified reason (human-readable).
    pub reason: String,
}

/// Grouping of functions under a named macro profile.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AnalysisMacroProfile {
    /// Name of the macro profile.
    pub name: String,
    /// List of function RVAs in this macro.
    pub rvas: Vec<u64>,
}

/// Results from binary analysis, returned to the frontend.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AnalysisResult {
    /// Environment type (UserMode, KernelMode, UEFI).
    pub environment: PeEnvironment,
    /// Functions found during analysis.
    pub functions: Vec<AnalysisFunction>,
    /// Rejected functions and reasons.
    pub rejects: Vec<AnalysisReject>,
    /// Macro profiles generated from analysis.
    pub macros: Vec<AnalysisMacroProfile>,
}

/// Symbol representation used in YAML: either name or RVA.
#[derive(Debug, Serialize, Deserialize)]
pub enum YamlSymbol {
    /// Symbol name
    Name(String),
    /// Symbol RVA.
    Rva(u64),
}

/// Obfuscation profile for YAML configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct YamlProfile {
    /// Profile name (referenced by source macros).
    pub name: String,
    /// Passes to apply to this profile.
    pub passes: Vec<ObfuscationPass>,
    /// Compiler configuration for this profile.
    pub compiler_settings: CDCompilerSettings,
    /// Symbols targeted by this profile.
    pub symbols: Vec<YamlSymbol>,
    /// Only used by the SaaS UI. Not used by the CLI.
    pub color: Option<String>,
}

/// Root YAML config structure.
#[derive(Debug, Serialize, Deserialize)]
pub struct YamlConfig {
    /// Version of the config file format.
    pub version: String,
    /// Global module-wide obfuscation settings.
    pub module_settings: CDModuleSettings,
    /// Obfuscation profiles to apply.
    pub profiles: Vec<YamlProfile>,
}
