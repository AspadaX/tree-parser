use std::{error::Error, result};

use tokio::main;
use tree_parser::{parse_directory, search_by_node_type, ParseOptions, ParsedProject};

#[main]
async fn main() -> Result<(), Box<dyn Error>> {
    let project: ParsedProject = parse_directory("./vllm-main", ParseOptions::default()).await?;
    
    for file in project.files {
        let result = search_by_node_type(&file, "class_definition", Some("AdapterMapping"));
    }
    
    Ok(())
}