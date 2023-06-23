#![allow(dead_code)]

use jsonc_to_json::jsonc_to_json;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Data {
    arr: Vec<i32>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let jsonc = r#"
{
    // JSON with Comments allow line comments
    /* Multi
       Line
       Block comments
       are also allowed */
    "arr": [
        2,
        3,
        4,
        // JSON with Comments also allow trailing commas
        5,,
    // Trailing comma
    ],
    /*
      Calling `jsonc_to_json()` removes e.g. comments
      and trailing commas to ensure that valid JSONC
      is turned into valid JSON
    */
}
"#;

    let json = jsonc_to_json(jsonc);
    let data: Data = serde_json::from_str(&json)?;

    println!("{}", json);
    println!("{:?}", data);

    Ok(())
}
