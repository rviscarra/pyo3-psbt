from base64 import b64encode

from fastapi import APIRouter, Body

from pyo3_psbt.dependencies import BitcoinApiFromState, SettingsFromState
from pyo3_psbt.lib import build_payment_transaction
from pyo3_psbt.schema import PayWithPsbtRequest, PayWithPsbtResponse

router = APIRouter()


@router.post("/payment-psbt")
async def pay_with_psbt(
    btc_api: BitcoinApiFromState,
    settings: SettingsFromState,
    payload: PayWithPsbtRequest = Body(),
):
    utxos = await btc_api.fetch_utxos(payload.payer_address)
    builder = build_payment_transaction(
        settings, payload, utxos, settings.charge_amount
    )

    vbytes = builder.estimate_vbytes()
    miner_fee = payload.fee_rate * vbytes
    builder = build_payment_transaction(
        settings, payload, utxos, settings.charge_amount, miner_fee=miner_fee
    )

    psbt = builder.serialize()

    return PayWithPsbtResponse(psbt=b64encode(psbt).decode("ascii"))
