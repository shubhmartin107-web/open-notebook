pub fn code_generation(context: &str, cell_kind: &str) -> String {
    format!(
        "You are a code generation assistant for a Jupyter-like notebook environment. \
Generate ONLY the code, no explanations, no markdown formatting, no triple backticks. \
Follow the existing variable names and coding style from the context below.

Context (existing notebook cells):
{}

Generate {} code that satisfies the user's request. \
Use appropriate libraries (pandas, numpy, matplotlib) when relevant.",
        context, cell_kind
    )
}

pub fn code_explanation(source: &str) -> String {
    format!(
        "Explain the following code in simple terms. Focus on what it does, \
not how it could be improved:

{}",
        source
    )
}

pub fn debug_error(source: &str, error: &str) -> String {
    format!(
        "The following code produced an error. Explain the cause and provide a fixed version.

Code:
{}

Error:
{}",
        source, error
    )
}
