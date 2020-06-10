SHELL := /bin/bash

pages:
	mkdir -p docs
	cp www/{bootstrap.js,index.html,index.js} ./docs/
	cp pkg/{pack_stack_bg.wasm,pack_stack.js} ./docs/
	git add docs
	git commit
