use std::str::FromStr;

use bitcoin::{Amount, OutPoint, Txid};
use pyo3::{exceptions::PyValueError, FromPyObject, PyAny, PyResult};

pub struct InputUtxo {
    pub tx_id: Txid,
    pub vout: u32,
    pub(crate) value: Amount,
}

impl FromPyObject<'_> for InputUtxo {
    fn extract(obj: &'_ PyAny) -> PyResult<Self> {
        let tx_id = obj
            .getattr("tx_id")?
            .extract()
            .map(Txid::from_str)?
            .map_err(|_| PyValueError::new_err("invalid tx hash"))?;

        let value = obj.getattr("value")?.extract().map(Amount::from_sat)?;

        let vout = obj.getattr("vout")?.extract()?;

        Ok(Self { tx_id, vout, value })
    }
}

impl From<&InputUtxo> for OutPoint {
    fn from(utxo: &InputUtxo) -> Self {
        Self {
            txid: utxo.tx_id,
            vout: utxo.vout,
        }
    }
}
