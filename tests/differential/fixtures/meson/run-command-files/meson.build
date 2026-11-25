project('run-command-files', 'c')

script = find_program('check.py')
variants = files('variantsMapping.lst')
cmd = [
  script,
  '--symbols',
  variants,
]
result = run_command(cmd + ['--dry-run'], check: true)

executable('main', 'main.c')
