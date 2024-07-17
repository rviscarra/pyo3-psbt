class PsbtBuilder:

    def __init__(self, network: str) -> None: ...

    def add_output(self, address: str, amount: int) -> bool: ...
    def add_input(
        self, utxo: object, owner_address: str, owner_pub_key: bytes
    ) -> bool: ...
    def estimate_vbytes(self) -> int: ...
    def serialize(self) -> bytes: ...


def get_address_type(address: str) -> str: ...