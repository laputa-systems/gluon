defined_symbols() {
  case $(file "$1" 2>/dev/null) in
    *Mach-O*) nm -gU "$1" 2>/dev/null ;;
    *ELF*|*'current ar archive'*) nm -g --defined-only "$1" 2>/dev/null ;;
  esac | awk '{print $NF}' | LC_ALL=C sort
}
