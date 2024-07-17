FROM python:3.11-alpine AS base

ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    PIP_DEFAULT_TIMEOUT=100 \
    PIP_NO_CACHE_DIR=off \
    PIP_DISABLE_PIP_VERSION_CHECK=on \
    POETRY_VERSION=1.8.2 \
    POETRY_VIRTUALENVS_IN_PROJECT=true

WORKDIR /app

FROM base AS poetry

RUN apk add --no-cache gcc musl-dev libffi-dev cargo rust && \
    pip install --no-cache-dir poetry==${POETRY_VERSION}

COPY app/pyproject.toml app/poetry.lock ./

RUN poetry export --only=dev --without-hashes | pip install -r /dev/stdin
RUN poetry export --only=main --without-hashes > requirements.txt

COPY btc/ .

RUN maturin build --release --out /app/wheels/

FROM base AS runtime

RUN poetry export --only=main --without-hashes | pip install -r /dev/stdin

COPY --from=poetry /app/requirements.txt .

RUN pip install -r requirements.txt

COPY --from=poetry /app/wheels/*.whl .

RUN pip install *.whl

COPY app/pyo3_psbt ./pyo3_psbt/
COPY app/static ./static/

EXPOSE 9000

CMD ["uvicorn", "pyo3_psbt:app", "--port=9000", "--host=0.0.0.0"]