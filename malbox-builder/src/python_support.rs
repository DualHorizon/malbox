use pyo3::prelude::*;

fn greet(name: &str) -> String {
    format!("Hello, {}! From Rust.", name)
}

fn malbox_plugin(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(greet, m)?)?;
    Ok(())
}
