ifdef release
  $(info building 'release')
  mode := --release
else
  mode :=
endif

ifdef target
  $(info Target is '$(target)')
  target := target
else
  target :=
endif

build:
	cargo build $(mode) $(target)

help:
	@echo "usage: make [release=1] [target=<target>]"