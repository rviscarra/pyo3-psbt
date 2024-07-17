from contextlib import asynccontextmanager
from logging import getLogger

from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse
from fastapi.staticfiles import StaticFiles

from pyo3_psbt.btc_api import BitcoinApi
from pyo3_psbt.routes import router
from pyo3_psbt.schema import AppSettings, UserException

logger = getLogger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI):
    settings = AppSettings()  # type: ignore

    app.state.settings = settings
    app.state.btc_api = BitcoinApi(str(settings.api_base))
    yield


static = StaticFiles(directory="static")
app = FastAPI(lifespan=lifespan)
app.mount("/static", static, name="static")
app.include_router(router)


@app.exception_handler(UserException)
def handle_user_exception(_req: Request, exc: UserException):
    return JSONResponse(
        status_code=422,
        content={"error": exc.user_error},
    )


@app.exception_handler(Exception)
def handle_exception(req: Request, exc: Exception):
    return JSONResponse(
        status_code=500,
        content={"error": "an error occurred, check the application logs"},
    )


@app.get("/")
async def index(req: Request):
    return await static.get_response("index.html", req.scope)
