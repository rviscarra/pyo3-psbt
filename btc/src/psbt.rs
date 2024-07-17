use std::{borrow::Cow, str::FromStr};

use bitcoin::{
    absolute::LockTime, psbt, transaction::Version, Address, AddressType, Amount,
    CompressedPublicKey, Network, PublicKey, Script, Sequence, Transaction, TxIn, TxOut, Witness,
};
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    pyclass, pymethods,
    types::PyBytes,
    PyResult,
};

use crate::input_utxo::InputUtxo;

struct PsbtBuilderIn {
    prevout: InputUtxo,
    owner_address: Address,
    owner_pub_key: Option<PublicKey>,
}

#[pyclass]
pub struct PsbtBuilder {
    network: Network,
    inputs: Vec<PsbtBuilderIn>,
    outputs: Vec<TxOut>,
}

#[pymethods]
impl PsbtBuilder {
    // Our constructor, this will be executed when we call `PstbBuilder(...)` in Python
    #[new]
    pub fn new(network: &str) -> PyResult<Self> {
        let network =
            Network::from_str(network).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self {
            network,
            inputs: Vec::with_capacity(5),
            outputs: Vec::with_capacity(5),
        })
    }

    /// Add a new input to the PSBT, owner_pub_key is required for p2sh addresses
    fn add_input(
        &mut self,
        utxo: InputUtxo,
        owner_address: &str,
        owner_pub_key: Option<&PyBytes>,
    ) -> PyResult<()> {
        let owner_address = address_from_str(self.network, owner_address)?;
        let owner_pub_key = owner_pub_key
            .map(|pk| PublicKey::from_slice(pk.as_bytes()))
            .transpose()
            .map_err(|_| PyValueError::new_err("invalid public key"))?;

        self.inputs.push(PsbtBuilderIn {
            prevout: utxo,
            owner_address,
            owner_pub_key,
        });

        Ok(())
    }

    /// Add a new output to the PSBT
    fn add_output(&mut self, address: &str, amount: u64) -> PyResult<()> {
        let address = address_from_str(self.network, address)?;
        let amount = Amount::from_sat(amount);

        self.outputs.push(TxOut {
            script_pubkey: address.script_pubkey(),
            value: amount,
        });

        Ok(())
    }

    fn __str__(&self) -> String {
        format!(
            "<PsbtBuilder: {} inputs, {} outputs>",
            self.inputs.len(),
            self.outputs.len()
        )
    }

    /// Serialize the PSBT as bytes
    fn serialize(&self) -> PyResult<Cow<'_, [u8]>> {
        let psbt = self.build()?;
        Ok(psbt.serialize().into())
    }

    /// Estimate the final transaction size in vbytes
    /// For simplicity's sake it assumes the witnesses and script sig will be 50 bytes
    fn estimate_vbytes(&self) -> PyResult<u64> {
        const PAYLOAD_SIZE: usize = 50;

        let psbt = self.build()?;
        let mut unsigned_tx = psbt
            .extract_tx()
            .map_err(|ex| PyRuntimeError::new_err(format!("failed to extract tx: {}", ex)))?;
        for (tx_in, psbt_in) in unsigned_tx.input.iter_mut().zip(&self.inputs) {
            if let Some(addr_type) = psbt_in.owner_address.address_type() {
                match addr_type {
                    AddressType::P2sh => {
                        tx_in.witness = Witness::from_slice(&[[0; PAYLOAD_SIZE]]);
                        tx_in.script_sig = Script::from_bytes(&[0; PAYLOAD_SIZE]).into();
                    }
                    AddressType::P2wsh => {
                        tx_in.witness = Witness::from_slice(&[[0; PAYLOAD_SIZE]]);
                    }
                    _ => {
                        tx_in.script_sig = Script::from_bytes(&[0; PAYLOAD_SIZE]).into();
                    }
                }
            }
        }
        Ok(unsigned_tx.weight().to_vbytes_ceil())
    }
}

impl PsbtBuilder {
    fn build(&self) -> PyResult<psbt::Psbt> {
        let mut psbt_inputs = Vec::with_capacity(self.inputs.len());
        let mut tx_inputs = Vec::with_capacity(self.inputs.len());

        for input in &self.inputs {
            tx_inputs.push(TxIn {
                previous_output: (&input.prevout).into(),
                sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                ..Default::default()
            });

            let redeem_script = match (input.owner_address.address_type(), input.owner_pub_key) {
                (Some(AddressType::P2sh), Some(pub_key)) => {
                    if let Ok(pub_key) = CompressedPublicKey::try_from(pub_key) {
                        let addr = Address::p2wpkh(&pub_key, self.network);
                        Some(addr.script_pubkey())
                    } else {
                        None
                    }
                }
                _ => None,
            };

            psbt_inputs.push(psbt::Input {
                witness_utxo: Some(TxOut {
                    value: input.prevout.value,
                    script_pubkey: input.owner_address.script_pubkey(),
                }),
                redeem_script,
                ..Default::default()
            });
        }

        let tx_outputs = self.outputs.clone();

        let psbt_outputs = vec![Default::default(); tx_outputs.len()];

        let unsigned_tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: tx_inputs,
            output: tx_outputs,
        };

        Ok(psbt::Psbt {
            version: 0,
            inputs: psbt_inputs,
            outputs: psbt_outputs,
            unsigned_tx,
            proprietary: Default::default(),
            unknown: Default::default(),
            xpub: Default::default(),
        })
    }
}

fn address_from_str(network: Network, address: &str) -> PyResult<Address> {
    let address = Address::from_str(&address)
        .map_err(|e| PyValueError::new_err(e.to_string()))?
        .require_network(network)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(address)
}

#[cfg(test)]
mod test {

    use super::*;
    use bitcoin::Txid;

    const TX_ID: &str = "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b";
    const PAYMENT_ADDRESS: &str = "bcrt1qg3gmqfdwgteve988hvps7kws2kdzagtkqf6gu0";
    const RECIPIENT_ADDRESS: &str = "bcrt1qjp2yc9gtcke005ugg5895vv0yhx77nvv9cltga";

    #[test]
    fn single_in_single_out() {
        let mut builder = PsbtBuilder::new("regtest").expect("create builder");

        let tx_id = Txid::from_str(TX_ID).expect("valid tx hash");
        let input = InputUtxo {
            tx_id,
            vout: 0x1337,
            value: Amount::from_sat(10000),
        };

        builder
            .add_input(input, PAYMENT_ADDRESS, None)
            .expect("input to be added");

        builder
            .add_output(RECIPIENT_ADDRESS, 8000)
            .expect("output to be added");

        let psbt = builder.build().expect("psbt to build");

        let tx = psbt.extract_tx().expect("tx to be extracted");

        assert!(tx.input.len() == 1, "single input expected");
        assert!(tx.output.len() == 1, "single output expected");

        assert!(
            tx.input[0].previous_output.txid == tx_id,
            "input tx id mismatch"
        );
        assert!(
            tx.input[0].previous_output.vout == 0x1337,
            "input tx vout mismatch"
        );

        let out_address = Address::from_str(RECIPIENT_ADDRESS)
            .expect("valid address")
            .require_network(Network::Regtest)
            .expect("regtest address")
            .script_pubkey();

        assert!(
            tx.output[0].script_pubkey == out_address,
            "output script pubkey mismatch"
        );
        assert!(
            tx.output[0].value == Amount::from_sat(8000),
            "output value mismatch"
        );
    }
}
