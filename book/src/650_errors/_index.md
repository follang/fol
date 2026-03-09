# Error Handling

FOL does not center error handling around exceptions.

This section distinguishes two broad error categories:
- breaking errors:
  unrecoverable failures, typically associated with `panic`
- recoverable errors:
  errors that can be propagated or handled, typically associated with `report`

The detailed chapters explain:
- how each category behaves
- how routines expose recoverable error types
- how error-aware forms interact with control flow and pipes
