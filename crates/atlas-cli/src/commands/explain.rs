/// `atlas explain ATXXXX` — look up an error code and print its description, help, and example.
use anyhow::Result;
use atlas_runtime::diagnostic::error_codes;

pub fn run(code: &str) -> Result<()> {
    // Normalise: accept "at1003", "AT1003", "1003"
    let normalised = normalise_code(code);

    match error_codes::lookup(&normalised) {
        None => {
            eprintln!("error: unknown error code `{normalised}`");
            eprintln!("      Run `atlas explain --list` to see all codes.");
            std::process::exit(1);
        }
        Some(desc) => {
            println!("{}", desc.code);
            println!();
            println!("Description");
            println!("-----------");
            println!("{}", desc.title);

            if let Some(help) = desc.static_help {
                println!();
                println!("Help");
                println!("----");
                println!("{}", help);
            }
        }
    }

    Ok(())
}

/// List all known error codes with their titles.
pub fn run_list() -> Result<()> {
    let codes = error_codes::DESCRIPTOR_REGISTRY;
    println!("{:<10}  Description", "Code");
    println!("{}", "-".repeat(70));
    for desc in codes {
        println!("{:<10}  {}", desc.code, desc.title);
    }
    Ok(())
}

/// Normalise a user-supplied code string to uppercase with AT/AW prefix.
/// "at1003" → "AT1003"
/// "1003"   → "AT1003"
/// "aw3059" → "AW3059"
fn normalise_code(input: &str) -> String {
    let upper = input.to_uppercase();
    if upper.starts_with("AT") || upper.starts_with("AW") {
        upper
    } else {
        format!("AT{upper}")
    }
}
