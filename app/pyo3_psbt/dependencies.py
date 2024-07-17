from typing import Annotated, Any

from fastapi import Depends, Request

from pyo3_psbt.btc_api import BitcoinApi
from pyo3_psbt.schema import AppSettings


class AppStateDependency:

    def __init__(self, name: str) -> None:
        self._name = name

    def __call__(self, request: Request) -> Any:
        return getattr(request.app.state, self._name)


BitcoinApiFromState = Annotated[BitcoinApi, Depends(AppStateDependency("btc_api"))]
SettingsFromState = Annotated[AppSettings, Depends(AppStateDependency("settings"))]
