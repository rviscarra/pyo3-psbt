import btc


def test_get_address_type():
    assert (
        btc.get_address_type("bcrt1qg3gmqfdwgteve988hvps7kws2kdzagtkqf6gu0") == "p2wpkh"
    )
