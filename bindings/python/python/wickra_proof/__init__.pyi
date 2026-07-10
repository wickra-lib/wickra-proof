"""Type stubs for the wickra_proof package."""

__version__: str

class Prover:
    """A stateless prover driven by JSON commands."""

    def __init__(self) -> None:
        """Create a stateless prover."""
        ...

    def command(self, cmd_json: str) -> str:
        """Apply a command JSON and return the resulting response JSON.

        Raises ``ValueError`` if the command envelope cannot be parsed.
        """
        ...

    @staticmethod
    def version() -> str:
        """The library version."""
        ...
