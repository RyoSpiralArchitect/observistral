from __future__ import annotations

from abc import ABC, abstractmethod

from observistral.types import ChatRequest, ChatResponse


class ChatProvider(ABC):
    """Provider-agnostic chat interface."""

    def __init__(self, model: str) -> None:
        self.model = model

    @abstractmethod
    def chat(self, request: ChatRequest) -> ChatResponse:
        raise NotImplementedError
