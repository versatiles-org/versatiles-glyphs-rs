cargo metadata --format-version=1 --no-deps |
	sed -n 's/.*"version":"\([^"]*\)".*/\1/p' |
	head -n1