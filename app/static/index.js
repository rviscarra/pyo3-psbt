'use strict';

document.addEventListener("DOMContentLoaded", () => {
  // Get references to our UI nodes
  const broadcastElem = document.querySelector("#broadcast")
  const payButtonElem = document.querySelector("#pay-now");
  const feeRateElem = document.querySelector("#fee-rate");
  const logListElem = document.querySelector(".action-log");
  const ERROR_COLOR = "#ff0000", WARN_COLOR = "#e87632";

  // No dependency url-safe base64 encoding
  function b64Encode(data) {
    return btoa(data)
      .replaceAll("=", "")
      .replaceAll("+", "-")
      .replaceAll("/", "_");
  }

  // XVerse assumes it will receive an unsecured token
  function createUnsecuredToken(payload) {
    const header = { typ: "JWT", alg: "none" };

    const encodedHeader = b64Encode(JSON.stringify(header));
    const encodedPayload = b64Encode(JSON.stringify(payload));

    return `${encodedHeader}.${encodedPayload}.`;
  }

  // Utility function to append logs
  function appendLog(statusText, style = {}) {
    style.color = style.color ?? "#000";
    const logElem = document.createElement("pre");
    for (const key in style) {
      logElem.style[key] = style[key];
    }
    logElem.classList.add("log-entry");
    logElem.appendChild(document.createTextNode(statusText));
    logListElem.appendChild(logElem);
  }

  const appendError = (message, style = {}) => appendLog(message, { ...style, color: ERROR_COLOR });

  appendLog("Waiting for user input");

  async function payWithWallet() {
    const broadcast = broadcastElem.checked;

    if (!window.BitcoinProvider || typeof (window.BitcoinProvider) !== "object") {
      appendError("No bitcoin provider was found, install a wallet extension");
      return;
    }

    const wallet = window.BitcoinProvider;
    if (!wallet.hasOwnProperty("request") || typeof (wallet.request) !== "function") {
      appendError("Wallet not supported, try a different one");
      return;
    }

    while (logListElem.childNodes.length) {
      logListElem.removeChild(logListElem.lastChild);
    }

    // Fetch the user address for payment
    appendLog("Waiting for accounts ...");
    const getAccountsReply = await wallet.request("getAccounts", { purposes: ["payment"] });
    if (getAccountsReply.error) {
      appendError(getAccountsReply.error?.message ?? getAccountsReply.error);
      return;
    }

    const feeRate = +feeRateElem.value;
    if (isNaN(feeRate)) {
      appendLog(`Using ${feeRate} sats/vByte`);
    }

    const account = getAccountsReply.result[0];
    appendLog(`Using account "${account.address}"`);
    appendLog("POST /payment-psbt");

    const requestPayload = JSON.stringify({
      payer_address: account.address,
      payer_pub_key: account.publicKey,
      fee_rate: feeRate,
    }, null, 2);
    appendLog(`${requestPayload}`, { fontSize: '0.8rem' });

    // Fetch the PSBT from the backend
    const psbtResponse = await fetch("/payment-psbt", {
      method: "post",
      body: requestPayload,
      headers: {
        "content-type": "application/json",
      }
    });
    
    // Verify everything worked as expected
    appendLog(`Server replied with HTTP ${psbtResponse.status}`);
    const payload = await psbtResponse.json();
    if (payload.error) {
      appendError(`Error: ${payload.error}`);
      return;
    }
    appendLog(payload.psbt, { color: "#3535AA", fontSize: "0.8rem" });

    const signPayload = createUnsecuredToken({
      network: {
        type: "Testnet",
      },
      psbtBase64: payload.psbt,
      broadcast,
      inputsToSign: [{
        address: account.address,
        signingIndexes: [0],
      }],
      message: "Please sign"
    });

    appendLog("Awaiting user signature ...");
    if (broadcast) {
      appendLog("Transaction will be sent to Bitcoin network if signed ...", { color: WARN_COLOR });
    }
    
    // Ask the user to sign the transaction, and broadcast it, if the checkbox was selected
    const signResult = await wallet.signTransaction(signPayload);
    if (signResult.psbtBase64) {
      appendLog("Wallet returned signed PSBT:");
      appendLog(signResult.psbtBase64, { color: "#35AA35", fontSize: '0.8rem' });
    }
  }

  // Register the event listeners
  payButtonElem.addEventListener("click", () => {
    payWithWallet().catch(ex => {
      appendError(`${ex}`);
    });
  });

  broadcastElem.addEventListener("change", (evt) => {
    if (evt.target.checked) {
      appendLog("Warning: The signed TX will be sent to the Bitcoin network", { color: WARN_COLOR });
    } else {
      appendLog("Not broadcasting");
    }
  });
});
