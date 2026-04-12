#!/usr/bin/env bash
# Detect host type (desktop vs laptop)

set -euo pipefail

# Priority: DOTFILES_HOST env var > battery detection > hostname detection
if [[ -n "${DOTFILES_HOST:-}" ]]; then
    echo "$DOTFILES_HOST"
    exit 0
fi

# Check for battery (laptops have batteries)
if [[ -d /sys/class/power_supply ]]; then
    for supply in /sys/class/power_supply/*; do
        if [[ -r "$supply/type" ]]; then
            type=$(cat "$supply/type" 2>/dev/null || echo "")
            if [[ "$type" == "Battery" ]]; then
                echo "laptop"
                exit 0
            fi
        fi
    done
fi

# Check hostname for clues
hostname=$(hostname -s 2>/dev/null || echo "")
case "$hostname" in
    *laptop*|*mobile*|*notebook*)
        echo "laptop"
        ;;
    *desktop*|*pc*|*workstation*)
        echo "desktop"
        ;;
    *)
        # Default to desktop if no battery and no hostname clue
        echo "desktop"
        ;;
esac
