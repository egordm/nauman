# Changelog
## [1.1.2] - 2021-12-11

### Bug Fixes
- [BUGFIX] Fixed wrong parsing of env section

### Continuous Integration
- [CI] Bumped cargo version

### Testing
- [TESTS] Added integration tests

## [1.1.1] - 2021-12-11

### Bug Fixes
- [BUGFIX] Bumped version in cargo.toml
- [BUGFIX] Fixed clap requiring a derive feature flag in newer versions

## [1.1.0] - 2021-12-11

### Bug Fixes
- [BUGFIX] Updated dry-run to be a flag in cli

### Continuous Integration
- [CI] Fixed code style
- [CI] Added cliff for changelog gen

### Documentation
- [DOCS] Split Job Syntax into a separate file
- [Docs] Fixed anchors in job syntax doc

### Features
- [FEATURE] Updated cargo toml
- [FEATURE] Added badges
- [FEATURE] Added binary stripping
- [FEATURE] Added dotenv loading
- [FEATURE] Added task output file support
- [FEATURE] Added unit tests for utils
- [FEATURE] Console logger is always added except explicitly configured
- [FEATURE] Added summary printing

### Other
- Update README.md

## [1.0.0] - 2021-12-05

### Features
- [FEATURE] Created initial project
- [FEATURE] Added an example job script
- [FEATURE] Added common types module
- [FEATURE] Added configuration parsing module
- [FEATURE] Added flow structs and config to flow transformation
- [FEATURE] Added output streaming utilities
- [FEATURE] Added basic execution functionality
- [FEATURE] Added main
- [FEATURE] Added pretty printing. Cleaned job script a bit
- [FEATURE] Code style fixes
- [FEATURE] Improved task id generation function
- [FEATURE] Added desired log config parsing
- [FEATURE] Added dual output writer for stdout and stderr splitting
- [FEATURE] Added output creation from log config
- [BUGFIX] Added wait till both stdout and stderr buffers are done
- [FEATURE] Added output specification
- [FEATURE] Added context based output streaming
- [FEATURE] Added custom flow iterator to schedule complex routines
- [FEATURE] Added customer hook formatting
- [FEATURE] Improved hook parsing
- [FEATURE] Added task based hooks
- [FEATURE] Added task based before and after hooks
- [FEATURE] Added task result hooks
- [FEATURE] Added execution policy
- [FEATURE] Added on success execution policy
- [FEATURE] Added stateful logger
- [FEATURE] Added options / global config
- [FEATURE] Added log grouping and focus to the execution
- [FEATURE] Added dry run support. Added separate dir logging
- [FEATURE] Added cli
- [FEATURE] Code style fixes
- [FEATURE] Added default job setting
- [FEATURE] Added default job setting
- [FEATURE] Added documentation for config
- [FEATURE] Added more shell types (python etc).
- [FEATURE] Added more verbose logging
- [FEATURE] Added some basic documentation
- [FEATURE] Added more examples and started on writing docs
- [FEATURE] Code style fixes
- [FEATURE] Added ci
- [FEATURE] Added license, release workflow
- [DOCS] Updated readme

