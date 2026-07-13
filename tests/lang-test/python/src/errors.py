"""ripex-lang-test: Python exceptions hierarchy."""
class RipexError(Exception):
    pass


class ValidationError(RipexError):
    pass


class ConfigError(RipexError):
    pass


class NotFoundError(RipexError):
    pass
