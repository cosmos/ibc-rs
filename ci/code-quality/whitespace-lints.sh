#!/bin/bash
set -euo pipefail

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


check_binary grep
check_binary find

check_code_quality() {
    # exactly three arguments are passed
    if [ $# -ne 3 ] || [ "$1" = "" ] || [ "$2" = "" ] || [ "$3" = "" ]; then
        echo "Usage: check_code_quality <option> <pattern> <message>"
        exit 1
    fi

    if find . -type f \( -name "*.toml" -o -name "*.rs" \) -not -path '*/.*' -exec grep "$1" "$2" {} \; | grep '.*'; then
        echo "$3"
        return 1
    else
        return 0
    fi
}

exit_code=0

check_code_quality -nHIP "\s+$" "found: trailing whitespaces" || exit_code=1
check_code_quality -nHIP $"\t" "found: tabs" || exit_code=1
check_code_quality -zLIP ".*\n\Z" "not found: newline at EOF" || exit_code=1

exit "$exit_code"
