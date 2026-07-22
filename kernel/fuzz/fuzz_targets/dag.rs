#![no_main]

use libfuzzer_sys::fuzz_target;
use onb_kernel::notebook::{Notebook, CellKind};
use onb_kernel::dag;

fuzz_target!(|data: &[u8]| {
    // Create a notebook with cells from fuzz data
    let mut nb = Notebook::new("fuzz");
    let source = String::from_utf8_lossy(data);
    
    // Add cells of different types using the fuzzed data
    nb.add_cell(CellKind::Python, &source);
    nb.add_cell(CellKind::Sql, &source);
    nb.add_cell(CellKind::Markdown, &source);
    
    // Build DAG should never panic
    let _ = dag::build_dag(&mut nb);
});
