SHELL := /bin/bash

pages:
	mkdir -p docs
	cp -r www/{bootstrap.js,index.html,index.js,css} ./docs/
	cp pkg/{pack_stack_bg.wasm,pack_stack.js} ./docs/
	git add docs
	git commit
