"""Third-party data-provider credentials and region/exchange mappings.

API keys MUST NOT be hardcoded in this module. They are loaded from
environment variables at import time. The previous version of this file
shipped three live credentials in plaintext — those have been rotated
and removed, and callers are expected to set the corresponding env vars
(see `.env.example`).

If a key is missing, the constant is set to `None` rather than raising,
because many quantbench code paths run offline (e.g. unit tests, doc
builds) and should not fail just because a data-provider key is unset.
Code that actually issues HTTP requests against these providers should
check for `None` and fail fast with a clear error message.
"""

import os


def _required_env(name: str) -> str | None:
    """Return `os.getenv(name)` but collapse empty strings to `None`.

    `.env.example` ships with empty assignments like `EODHD_API_KEY=`, so
    a user who copies `.env.example` to `.env` without filling in values
    gets `""` from `os.getenv`, not `None`. That breaks the contract of
    this module (which promises a missing key is `None`) and would let a
    caller that writes `if EODHD_API_KEY is None:` silently issue an
    HTTP request with an empty API key. Normalise both cases to `None`.
    """
    value = os.getenv(name)
    return value if value else None


EODHD_API_KEY = _required_env("EODHD_API_KEY")
POLYGON_IO_KEY = _required_env("POLYGON_IO_KEY")
AZURE_LANGUAGE_ENDPOINT = os.getenv(
    "AZURE_LANGUAGE_ENDPOINT",
    "https://aaai24.cognitiveservices.azure.com/",
)
AZURE_LANGUAGE_KEY = _required_env("AZURE_LANGUAGE_KEY")

REGION_XCHG_MAPPING = {
    "EODHD": {
        "us": ["US"],
        "cn": ["SHE", "SHG"],
        "uk": ["LSE"],
        "fr": ["PA"],
        "euronext": ["PA", "BR", "LI", "AS"],
    },
    "Wikidata": {
        "us": ["NASDAQ", "New York Stock Exchange"],
        "cn": ["Shanghai Stock Exchange", "SHENZHEN STOCK EXCHANGE"],
        "hk": ["Hong Kong Exchanges and Clearing Limited"],
        "jp": [
            "TOKYO STOCK EXCHANGE JASDAQ",
            "TOKYO STOCK EXCHANGE-TOKYO PRO MARKET",
            "Tokyo Stock Exchange",
        ],
        "uk": ["London Stock Exchange"],
        "fr": ["EURONEXT - EURONEXT PARIS"],
    },
}
