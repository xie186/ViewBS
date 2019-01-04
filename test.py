# Use of this source code is governed by the MIT license.
__license__ = "MIT"

try:
    from collections.abc import Callable # Python 3.6
except ImportError , e:
    from collections import Callable
import re
import sys
import warnings
try:
    import soupsieve
except ImportError, e:
    soupsieve = None
    warnings.warn(
        'The soupsieve package is not installed. CSS selectors cannot be used.'
    )

from bs4.dammit import EntitySubstitution

