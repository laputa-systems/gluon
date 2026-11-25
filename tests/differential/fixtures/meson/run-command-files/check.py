#!/usr/bin/env python3

import os
import sys

srcdir = os.path.dirname(os.path.abspath(__file__))
expected = [
    '--symbols',
    os.path.join(srcdir, 'variantsMapping.lst'),
    '--dry-run',
]
if sys.argv[1:] != expected:
    print('unexpected argv: %r' % (sys.argv[1:],), file=sys.stderr)
    sys.exit(1)
print('ok')
