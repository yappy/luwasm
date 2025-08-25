# Makefile for samples

all: hello.html sdl.html

%.html: %.c
	emcc $< -o $@

.PHONY: clean

clean:
	rm *.html *.js *.wasm
