# Project State

## Current Focus
Add a new `apply_command_output` method to the base `Widget` trait to allow widgets to update internal state when their bound commands are rerun after the `refresh_seconds` interval. Implement this method for the Gauge, KeyValueGrid, LogViewer, and StatusBadge widgets with logic tailored to their respective parsed command output types.

## Completed
- [x] Add `apply_command_output` method to `Widget` trait with default no-op implementation, invoked when a widget's bound command is rerun post-refresh
- [x] Import `ParsedOutput` from the command module into `widget.rs` to support the new trait method
- [x] Implement `apply_command_output` for Gauge to parse Scalar parsed output as f64 and update the gauge value
- [x] Implement `apply_command_output` for KeyValueGrid to trigger state updates via existing `update_from_output` logic
- [x] Implement `apply_command_output` for LogViewer to append new parsed output via existing `append_output` logic
- [x] Implement `apply_command_output` for StatusBadge to update status text from Scalar parsed output
