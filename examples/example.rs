use jsonc_to_json::{jsonc_to_json, jsonc_to_json_iter};

fn main() {
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
        5,
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
    println!("{}", json);
    println!();

    // Alternatively, use `jsonc_to_json_iter()` for a non-allocating
    // iterator, that outputs string slices of valid JSON
    for part in jsonc_to_json_iter(jsonc) {
        println!("{:?}", part);
    }
}
