from httpx import AsyncClient

from pyo3_psbt.schema import Utxo


class BitcoinApi:

    def __init__(self, base_url: str) -> None:
        self._client = AsyncClient(base_url=base_url)

    async def fetch_utxos(self, address: str):
        response = await self._client.get(f"/api/address/{address}/utxo")
        payload = response.json()
        return [Utxo.model_validate(u) for u in payload]
