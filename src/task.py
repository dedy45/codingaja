from __future__ import annotations

from dataclasses import dataclass


@dataclass
class PortingTask:
    """A single porting task with name and description."""

    name: str
    description: str


__all__ = ["PortingTask"]
