#!/bin/bash
set -euo pipefail

# use ggrep for macOS, and grep for Linux
case "$OSTYPE" in
    darwin*) GREP="ggrep" ;;
    linux-gnu*) GREP="grep" ;;
    *) echo "Unknown OS: $OSTYPE" && exit 1 ;;
esac

check_binary() {
    # exactly one argument is passed
    if [ $# -ne 1 ] || [ "$1" = "" ]; then
        echo "Usage: check_binary <binary>"
        exit 1
    fi

    if ! (type "$1" >/dev/null 2>&1); then
        echo "$1 is not present"
        exit 1
    fi
}

check_binary "$GREP"
check_binary find

check_code_quality() {
    # exactly three arguments are passed
    if [ $# -ne 3 ] || [ "$1" = "" ] || [ "$2" = "" ] || [ "$3" = "" ]; then
        echo "Usage: check_code_quality <option> <pattern> <message>"
        exit 1
    fi

    if find . -type f -name "*.toml" -o -name "*.rs" \
        -not -path '*/.*' -not -path '*/target/*' \
        -exec "$GREP" "$1" "$2" {} \; | "$GREP" '.*'; then
        echo "$3"
        return 1
    else
        return 0
    fi
}

exit_code=0

check_code_quality -nHIP "\s+$" "found: trailing whitespaces" || exit_code=1
check_code_quality -nHIP $"\t" "found: tabs" || exit_code=1
check_code_quality -zLIP ".*\n\Z" "found: no newline at EOF" || exit_code=1

if [ "$exit_code" -eq 0 ]; then
    echo "All code quality checks passed successfully."
fi

exit "$exit_code"
