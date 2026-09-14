#!/bin/sh
set -e
cd "$(dirname "$0")/.."

cargo build -q --bins --release
bin=target/release/versatiles_glyphs
assets=pages/web/assets
font=noto_sans_regular

# Glyphs for index.html
$bin merge -o "$assets/glyphs/$font/" testdata/Noto\ Sans/*.ttf

# Glyphs for sdf-step.html, one set per SDF step, plus their sizes in sizes.json

# Prints the total size of all given files.
raw_size() {
	cat "$@" | wc -c | tr -d ' '
}

# Prints the total size of all given files, each compressed with gzip -9.
gzip_size() {
	for file in "$@"; do gzip -9c "$file" | wc -c; done | awk '{ sum += $1 } END { print sum }'
}

sizes="$assets/sizes.json"
printf '{"font":"%s","steps":[' "$font" >"$sizes"
separator=''
for step in 1 2 4 8 16; do
	dir="$assets/glyphs-step-$step/$font"
	if [ "$step" = 1 ]; then
		# Step 1 is the default output, no need to render it again.
		rm -rf "$assets/glyphs-step-1"
		mkdir -p "$assets/glyphs-step-1"
		cp -R "$assets/glyphs/$font" "$dir"
	else
		$bin merge --sdf-step "$step" -o "$dir/" testdata/Noto\ Sans/*.ttf
	fi

	echo "Measuring sizes for SDF step $step" >&2
	tar_gz=$(tar -cf - -C "$dir" . | gzip -9 | wc -c | tr -d ' ')
	printf '%s{"step":%s,"raw":%s,"gzip":%s,"tar_gz":%s,"ranges":{' \
		"$separator" "$step" "$(raw_size "$dir"/*.pbf)" "$(gzip_size "$dir"/*.pbf)" "$tar_gz" >>"$sizes"
	range_separator=''
	for range in 0-255 1536-1791 19968-20223; do
		file="$dir/$range.pbf"
		printf '%s"%s":{"raw":%s,"gzip":%s}' \
			"$range_separator" "$range" "$(raw_size "$file")" "$(gzip_size "$file")" >>"$sizes"
		range_separator=','
	done
	printf '}}' >>"$sizes"
	separator=','
done
printf ']}\n' >>"$sizes"
