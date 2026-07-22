use crate::notebook::{CellOutput, OutputItem};
use anyhow::Result;

/// Execute a Python cell using PyO3.
#[cfg(feature = "python")]
pub fn execute_python_cell(source: &str) -> Result<CellOutput> {
    use pyo3::prelude::*;
    use pyo3::types::PyModule;

    let start = std::time::Instant::now();

    Python::with_gil(|py| -> PyResult<CellOutput> {
        let io = PyModule::import(py, "io")?;
        let sys = PyModule::import(py, "sys")?;
        let string_io = io.getattr("StringIO")?.call0()?;
        sys.setattr("stdout", string_io.clone())?;
        sys.setattr("stderr", string_io.clone())?;

        let code = std::ffi::CString::new(source)?;
        let name = std::ffi::CString::new("<cell>")?;
        let result = PyModule::from_code(py, code.as_c_str(), name.as_c_str(), name.as_c_str());
        let duration = start.elapsed().as_millis() as u64;

        let captured: String = string_io
            .getattr("getvalue")?
            .call0()?
            .extract()?;

        match result {
            Ok(_module) => Ok(CellOutput {
                items: if captured.is_empty() {
                    vec![]
                } else {
                    vec![OutputItem {
                        mime_type: "text/plain".to_string(),
                        data: captured.as_bytes().to_vec(),
                        text: Some(captured),
                        render_priority: 0,
                    }]
                },
                error: None,
                duration_ms: duration,
            }),
            Err(e) => {
                let traceback = e.to_string();
                Ok(CellOutput {
                    items: vec![],
                    error: Some(traceback),
                    duration_ms: duration,
                })
            }
        }
    })
    .map_err(|e: PyErr| anyhow::anyhow!("Python execution error: {}", e))
}

#[cfg(not(feature = "python"))]
pub fn execute_python_cell(source: &str) -> Result<CellOutput> {
    let _ = source;
    Ok(CellOutput {
        items: vec![],
        error: Some("Python execution not available (compiled without 'python' feature)".to_string()),
        duration_ms: 0,
    })
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "python")]
    #[test]
    fn test_python_simple_execution() {
        let output = super::execute_python_cell("x = 1 + 2\nprint(x)").unwrap();
        assert!(output.error.is_none());
    }
}
