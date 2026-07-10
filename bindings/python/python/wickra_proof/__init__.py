"""Wickra Proof — the deterministic Proof-of-Backtest core.

Create a :class:`Prover`, drive it with command JSONs (``prove``, ``verify``,
``canonicalize``, ``version``) and read back response JSONs. The same command
protocol crosses every language binding, so this Python front-end drives the
exact same core as the native CLI.
"""

from ._wickra_proof import Prover, __version__

__all__ = ["Prover", "__version__"]
