"""Pin the public surface of the Prover class across bindings."""

from wickra_proof import Prover

EXPECTED_METHODS = {"command", "version"}


def test_expected_methods_present() -> None:
    for name in EXPECTED_METHODS:
        assert hasattr(Prover, name), f"missing method: {name}"


def test_no_unexpected_public_methods() -> None:
    public = {name for name in dir(Prover) if not name.startswith("_")}
    assert public == EXPECTED_METHODS
