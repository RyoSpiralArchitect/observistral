from __future__ import annotations

from dataclasses import dataclass


@dataclass(slots=True)
class ProviderConfig:
    provider: str
    model: str
    api_key: str | None = None
    base_url: str | None = None
    timeout_seconds: int = 120
    device: str = "auto"

    @property
    def normalized_provider(self) -> str:
        return self.provider.strip().lower()
