use codedefender_config::{AnalysisResult, YamlSymbol};

// Resolve symbol names to RVA's. If a symbol is specified via RVA
// then validate it before including it in the result.
pub fn resolve_symbols(
    symbols: &[YamlSymbol],
    analysis: &AnalysisResult,
) -> Result<Vec<u64>, Box<dyn std::error::Error>> {
    let mut resolved = Vec::new();

    for symbol in symbols {
        match symbol {
            YamlSymbol::Name(name) => {
                // Search in returned functions and rejects for symbol by name.
                // If it was rejected for "ReadWriteToCode" we will force resolve it.
                let rva = analysis
                    .functions
                    .iter()
                    .find(|f| f.symbol == *name)
                    .map(|e| e.rva)
                    .or_else(|| {
                        analysis
                            .rejects
                            .iter()
                            .find(|r| r.symbol == *name && r.ty == "ReadWriteToCode")
                            .map(|e| e.rva)
                    });

                match rva {
                    Some(rva) => resolved.push(rva),
                    None => {
                        log::error!("Symbol `{}` not found in analysis result", name);
                        return Err("Missing symbol".into());
                    }
                }
            }
            YamlSymbol::Rva(rva) => {
                if !is_valid_rva(*rva, analysis) {
                    log::error!("RVA {:X} not found in analysis", rva);
                    return Err("Invalid RVA".into());
                }
                resolved.push(*rva);
            }
        }
    }

    Ok(resolved)
}

pub fn is_valid_rva(rva: u64, analysis: &AnalysisResult) -> bool {
    analysis.functions.iter().any(|f| f.rva == rva)
        || analysis
            .rejects
            .iter()
            .any(|r| r.rva == rva && r.ty == "ReadWriteToCode")
}
