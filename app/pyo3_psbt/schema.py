from typing import Annotated, Literal

import btc
from pydantic import AfterValidator, BaseModel, Field
from pydantic_core import PydanticCustomError, Url
from pydantic_settings import BaseSettings, SettingsConfigDict

DUST_VALUE = 600


class AppSettings(BaseSettings):
    api_base: Url
    recipient_address: "BitcoinAddress"
    network: Literal["bitcoin", "testnet"] = "testnet"
    charge_amount: int = 1000

    model_config = SettingsConfigDict(env_prefix="SAMPLE_")


class PayWithPsbtRequest(BaseModel):
    payer_address: str
    payer_pub_key: str
    fee_rate: int


class PayWithPsbtResponse(BaseModel):
    psbt: str | None = None
    error: str | None = None


class Utxo(BaseModel):
    tx_id: str = Field(alias="txid")
    vout: int
    value: int


class UserException(Exception):

    def __init__(self, user_error: str, internal_error: str | None = None) -> None:
        self.user_error = user_error
        super().__init__(internal_error or user_error)


def validate_bitcoin_address(raw_addr: str):
    try:
        btc.get_address_type(raw_addr)
        return raw_addr
    except:
        raise PydanticCustomError(
            "bitcoin_error",
            "{address} is not a valid Bitcoin address",
            {"address": raw_addr},
        )


BitcoinAddress = Annotated[str, AfterValidator(validate_bitcoin_address)]
