#!/bin/bash
# Sort pages by hit count (lowest first)
# Pages with no hit count show "NONE" at the top
# Usage: ./sort-hit-counts.sh ./snapshots-local

if [ -z "$1" ]; then
    echo "Usage: $0 <directory>"
    exit 1
fi

DIR="$1"

if [ ! -d "$DIR" ]; then
    echo "Error: $DIR is not a directory"
    exit 1
fi

# Extract hit counts from each HTML file
# Format: "Page Hits:" followed by <td>NUMBER</td> or <td>ELIDED_HIT_COUNT</td>
{
    for file in "$DIR"/*.html; do
        [ -f "$file" ] || continue
        basename="${file##*/}"

        # Extract the hit count line after "Page Hits:"
        # Look for <td>NUMBER</td> pattern
        count=$(grep -A1 "Page Hits:" "$file" 2>/dev/null | grep -oP '<td>\K[^<]+' | head -1)

        if [ -z "$count" ]; then
            echo "-2	NONE	$basename"
        elif [ "$count" = "ELIDED_HIT_COUNT" ]; then
            echo "-1	ELIDED	$basename"
        else
            echo "$count	$count	$basename"
        fi
    done
} | sort -t$'\t' -k1,1n | cut -f2-
