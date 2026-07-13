pub const IMPORT_SIMPLE: &str = "import os\n";
pub const IMPORT_MULTI: &str = "import os, sys\n";

pub const FUNC_NO_ARGS: &str = "def calc():\n    pass\n";
pub const FUNC_WITH_ARGS: &str = "def calc(a, c):\n    pass\n";
pub const FUNC_ASYNC: &str = "async def load():\n    pass\n";

pub const CLASS_EMPTY: &str = "class MyClass:\n    pass\n";
pub const CLASS_WITH_INIT: &str = "class MyClass:\n    def __init__(a):\n        pass\n";
pub const CLASS_DECORATED: &str = "@dataclass\nclass Point:\n    pass\n";

pub const ASSIGN_SIMPLE: &str = "x = 1\n";
pub const ANN_ASSIGN: &str = "x: int\n";
pub const AUG_ASSIGN: &str = "x += 1\n";

pub const DECORATOR_FUNC: &str = "@decorator\ndef index():\n    pass\n";

pub const ALL_FIXTURES: &[&str] = &[
    IMPORT_SIMPLE,
    IMPORT_MULTI,
    FUNC_NO_ARGS,
    FUNC_WITH_ARGS,
    FUNC_ASYNC,
    CLASS_EMPTY,
    CLASS_WITH_INIT,
    CLASS_DECORATED,
    ASSIGN_SIMPLE,
    ANN_ASSIGN,
    AUG_ASSIGN,
    DECORATOR_FUNC,
];
