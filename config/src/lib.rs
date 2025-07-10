use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const YAML_CONFIG_VERSION: &str = "1.0.0";

#[derive(Debug, Serialize, Deserialize)]
pub enum MutationEngineExtension {
    Generic = 0,
    SSE = 1,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PeEnvironment {
    UserMode,
    KernelMode,
    UEFI,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LifterSettings {
    pub lift_calls: bool,
    pub max_stack_copy_size: u32,
    pub split_on_calls_fallback: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizationSettings {
    pub constant_propagation: bool,
    pub instruction_combine: bool,
    pub dead_code_elim: bool,
    pub prune_useless_block_params: bool,
    pub iterations: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssemblerSettings {
    pub shuffle_basic_blocks: bool,
    pub instruction_prefix: String,
    pub random_prefix_chance: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CDCompilerSettings {
    pub assembler_settings: AssemblerSettings,
    pub optimization_settings: OptimizationSettings,
    pub lifter_settings: LifterSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FakePdbString {
    pub enabled: bool,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomSectionName {
    pub enabled: bool,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CDModuleSettings {
    pub ida_crasher: bool,
    pub import_protection: bool,
    pub fake_pdb_string: FakePdbString,
    pub custom_section_name: CustomSectionName,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Semantics {
    pub add: bool,
    pub sub: bool,
    pub and: bool,
    pub xor: bool,
    pub or: bool,
    pub not: bool,
    pub neg: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BitWidths {
    pub bit8: bool,
    pub bit16: bool,
    pub bit32: bool,
    pub bit64: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoopEncodeSemantics {
    pub iterations: u32,
    pub probability: u32,
    pub semantics: Semantics,
    pub bitwidths: BitWidths,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MixedBooleanArithmetic {
    pub iterations: u32,
    pub probability: u32,
    pub semantics: Semantics,
    pub bitwidths: BitWidths,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MutationEngine {
    pub iterations: u32,
    pub probability: u32,
    pub extension: MutationEngineExtension,
    pub semantics: Semantics,
    pub bitwidths: BitWidths,
}

/// Unit‑struct passes
#[derive(Debug, Serialize, Deserialize)]
pub struct IDADecompilerCrasher;

#[derive(Debug, Serialize, Deserialize)]
pub struct ObscureConstants;

#[derive(Debug, Serialize, Deserialize)]
pub struct ObscureReferences;

#[derive(Debug, Serialize, Deserialize)]
pub struct ObscureControlFlow;

#[derive(Debug, Serialize, Deserialize)]
pub enum ObfuscationPass {
    LoopEncodeSemantics(LoopEncodeSemantics),
    MixedBooleanArithmetic(MixedBooleanArithmetic),
    MutationEngine(MutationEngine),
    IDADecompilerCrasher(IDADecompilerCrasher),
    ObscureConstants(ObscureConstants),
    ObscureReferences(ObscureReferences),
    ObscureControlFlow(ObscureControlFlow),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CDProfile {
    pub name: String,
    pub passes: Vec<ObfuscationPass>,
    pub compiler_settings: CDCompilerSettings,
    pub symbols: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CDConfig {
    pub module_settings: CDModuleSettings,
    pub profiles: Vec<CDProfile>,
}

/// High level function information
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct AnalysisFunction {
    /// Rva of the function
    pub rva: u64,
    /// Name of the function
    pub symbol: String,
    /// However many times this function is referenced
    pub ref_count: usize,
}

/// Reject string stuff for saas
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct AnalysisReject {
    /// Address of rejected function
    pub rva: u64,
    /// Name of rejected function
    pub symbol: String,
    /// MlnFunctionRejectReason mnemonic
    pub ty: String,
    /// .to_string()'ed MlnFunctionRejectReason
    pub reason: String,
}

/// This structure gets sent to the browser as json.
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct AnalysisResult {
    /// (UserMode/Kernel/UEFI)
    pub environment: PeEnvironment,
    /// Function name (or hex of the rva) --> rva of the function.
    pub functions: Vec<AnalysisFunction>,
    /// All of the rejected functions and why they were rejected.
    pub rejects: Vec<AnalysisReject>,
}

/// Abstraction for symbols to specify them via name or RVA.
#[derive(Debug, Serialize, Deserialize)]
pub enum YamlSymbol {
    /// Name of a symbol
    Name(String),
    /// RVA of a symbol
    Rva(u64),
}

/// High level profile abstraction
#[derive(Debug, Serialize, Deserialize)]
pub struct YamlProfile {
    /// Name of the profile, this is also used by the source macros to specify which profile to obfuscate the function with!
    pub name: String,
    /// Passes to run on the symbols contained inside of this profile
    pub passes: Vec<ObfuscationPass>,
    /// Compiler settings for this profile.
    pub compiler_settings: CDCompilerSettings,
    /// Symbols contained inside of this profile
    pub symbols: Vec<YamlSymbol>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlConfig {
    /// Version of this CodeDefender CLI file
    pub version: String,
    /// API generated on the website
    pub api_key: String,
    /// Input file for processing (exe, dll, sys)
    pub input_file: PathBuf,
    /// Input PDB file for processing (optional)
    pub pdb_file: Option<PathBuf>,
    /// Module wide obfuscation settings
    pub module_settings: CDModuleSettings,
    /// All of the profiles used for obfuscation
    pub profiles: Vec<CDProfile>,
}