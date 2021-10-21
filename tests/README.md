# Testing
This folder has two purposes:
1. It contains, at the root level, integration tests for `psbattletools`. We welcome new test cases!
2. It also acts as a library crate with helper functions that can be shared between *unit tests* (in the regular source files) and *integration tests* (in this directory). Creating a separate, albeit small, crate seemed like the easiest way to share this code to me.