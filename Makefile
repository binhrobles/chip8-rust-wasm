
run-desktop:
	cd desktop && LIBRARY_PATH="$$(brew --prefix)/lib" cargo run $(GAME_PATH)

which-brew:
	echo "$$(brew --prefix)/lib"
