use std::str::FromStr;

use bitcoin::Address;
use pyo3::{exceptions::PyValueError, prelude::*};

mod input_utxo;
mod psbt;

// The following comment will go into the function's Docstring
/// get_address_type returns the provided address' type as a string
#[pyfunction]
fn get_address_type(address: &str) -> PyResult<String> {
    let address = Address::from_str(address)
        .map_err(|_| PyValueError::new_err("invalid address"))?
        .assume_checked();

    let address_type = address
        .address_type()
        .map_or("other".to_owned(), |at| at.to_string());

    Ok(address_type)
}

/// A Bitcoin helper module implemented in Rust
#[pymodule]
fn btc(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<psbt::PsbtBuilder>()?;
    m.add_function(wrap_pyfunction!(get_address_type, m)?)?;
    Ok(())
}

#[cfg(test)]
mod test {

    use super::get_address_type;

    #[test]
    pub fn test_get_address_type() {
        assert!(
            get_address_type("bcrt1qg3gmqfdwgteve988hvps7kws2kdzagtkqf6gu0").unwrap() == "p2wpkh"
        )
    }
}
