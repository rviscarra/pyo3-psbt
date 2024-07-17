## PyO3 example: Build a Bitcoin transaction with Python and Rust

The sample application exposes a simple UI that allows the user to submit a Bitcoin payment by signing (and optionally 
broadcasting) a Partially Signed Bitcoin Transaction (PSBT) that was built by the application's API. The code has been 
tested in Chrome using the XVerse Wallet extension, by default the application is configured to use Bitcoin's Testnet3 
chain. The application's code can be split into three major components:

1. Frontend code that handles user input and wallet interaction, implemented in vanilla Javascript.
2. Python backend code that takes care of the inbound HTTP request and the outbound API request to mempool.space.
3. Rust code that relies on the `bitcoin` crate to build the PSBT.

[This article](https://viscarra.dev/post/pyo3-psbt/) offers a more in-depth explanation of the code.

### Running the code

The application is dockerized so you can run it using Docker/Podman:


```bash
docker build . -t pyo3-psbt
docker run -it -p 9000:9000 \
  -e SAMPLE_API_BASE=https://mempool.space/testnet \
  -e SAMPLE_RECIPIENT_ADDRESS=bitcoin_address pyo3-psbt

# even easier if you are using Docker Compose:
SAMPLE_RECIPIENT_ADDRESS=bitcoin_address docker compose run
```

`bitcoin_address` is expected to be a valid Bitcoin address.
