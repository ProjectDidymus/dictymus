# winapp UI tests

Prerequisites: winapp CLI installed (`winapp ui --help` works); a fixture
StarDict dictionary (`.ifo` file). Build the GUI first: `cargo build -p dictymus-gui`.

Run: `pwsh winapp-tests/smoke.ps1 -Binary <path-to-dictymus-gui.exe> -Fixture <path-to.ifo>`

After running, inspect the automation tree with:
`winapp ui inspect -a <pid>`
to find actual control names/automation IDs for the selectors.
