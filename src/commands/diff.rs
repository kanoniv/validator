use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

pub fn run(file1: &Path, file2: &Path) -> Result<()> {
    // Read files
    let content1 = fs::read_to_string(file1)
        .with_context(|| format!("Failed to read file: {}", file1.display()))?;
    let content2 = fs::read_to_string(file2)
        .with_context(|| format!("Failed to read file: {}", file2.display()))?;

    // Parse YAML
    let spec1: serde_json::Value = serde_yaml::from_str(&content1)?;
    let spec2: serde_json::Value = serde_yaml::from_str(&content2)?;

    // Compare key sections
    println!(
        "{} {} vs {}",
        "Comparing:".bold(),
        file1.display(),
        file2.display()
    );
    println!();

    // Compare identity versions
    let v1 = spec1
        .get("identity_version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let v2 = spec2
        .get("identity_version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    if v1 != v2 {
        println!("{}:", "identity_version".cyan());
        println!("  {} {}", "-".red(), v1);
        println!("  {} {}", "+".green(), v2);
        println!();
    }

    // Compare rules
    let rules1 = spec1.get("rules").and_then(|r| r.as_array());
    let rules2 = spec2.get("rules").and_then(|r| r.as_array());

    if let (Some(r1), Some(r2)) = (rules1, rules2) {
        if r1.len() != r2.len() {
            println!("{}:", "rules".cyan());
            println!("  {} {} rules", "-".red(), r1.len());
            println!("  {} {} rules", "+".green(), r2.len());
            println!();
        }

        // Compare individual rules by name
        for rule2 in r2 {
            let name2 = rule2.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let matching_rule1 = r1
                .iter()
                .find(|r| r.get("name").and_then(|n| n.as_str()) == Some(name2));

            if let Some(rule1) = matching_rule1 {
                // Compare weights
                let w1 = rule1.get("weight").and_then(|w| w.as_f64());
                let w2 = rule2.get("weight").and_then(|w| w.as_f64());

                if w1 != w2 {
                    println!("{} '{}':", "rule".cyan(), name2);
                    println!("  {} weight: {:?}", "-".red(), w1);
                    println!("  {} weight: {:?}", "+".green(), w2);
                    println!();
                }
            } else {
                // New rule
                println!("{} '{}' (new)", "rule".cyan(), name2);
                println!("  {} {}", "+".green(), serde_json::to_string_pretty(rule2)?);
                println!();
            }
        }
    }

    // Compare thresholds
    let t1 = spec1.get("decision").and_then(|d| d.get("thresholds"));
    let t2 = spec2.get("decision").and_then(|d| d.get("thresholds"));

    if t1 != t2 {
        println!("{}:", "thresholds".cyan());
        if let Some(thresholds) = t1 {
            println!("  {} {}", "-".red(), thresholds);
        }
        if let Some(thresholds) = t2 {
            println!("  {} {}", "+".green(), thresholds);
        }
        println!();
        println!("{} Threshold change may affect match rates", "⚠".yellow());
    }

    Ok(())
}
