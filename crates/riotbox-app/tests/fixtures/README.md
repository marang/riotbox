# Riotbox App Regression Fixtures

These files are shared by multiple regression seams, including app-shell tests, app-state tests, and selected core view tests.

## Scene Regression Taxonomy

`scene_regression.json` uses section labels as a compact fixture vocabulary. The app crate centralizes the label mapping in its `#[cfg(test)]` support module; the core crate mirrors the same fixture-only mapping because it cannot depend on app test helpers.

Current mapping:

- `drop`, `chorus` -> `high`
- `break`, `outro` -> `low`
- `intro`, `build`, `verse`, `bridge` -> `medium`
- unknown labels -> `unknown`

If the fixture vocabulary changes, update the app test-support mapping, the core fixture projection helper, and any affected JSON expectations in the same slice.
