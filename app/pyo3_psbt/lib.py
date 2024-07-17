from btc import PsbtBuilder

from pyo3_psbt.schema import (
    DUST_VALUE,
    AppSettings,
    PayWithPsbtRequest,
    UserException,
    Utxo,
)


def build_payment_transaction(
    settings: AppSettings,
    req: PayWithPsbtRequest,
    payment_utxos: list[Utxo],
    charge_amount: int,
    miner_fee=0,
):
    spend_utxo = None
    required_amount = charge_amount + miner_fee
    for utxo in payment_utxos:
        if utxo.value >= required_amount:
            spend_utxo = utxo
            break

    if not spend_utxo:
        raise UserException("insufficient funds")

    builder = PsbtBuilder(settings.network)
    pub_key = bytes.fromhex(req.payer_pub_key)
    builder.add_input(
        spend_utxo,
        owner_address=req.payer_address,
        owner_pub_key=pub_key,
    )

    builder.add_output(
        address=settings.recipient_address,
        amount=charge_amount,
    )

    change_amount = spend_utxo.value - (charge_amount + miner_fee)
    if change_amount > DUST_VALUE:
        builder.add_output(
            address=req.payer_address,
            amount=change_amount,
        )

    return builder
