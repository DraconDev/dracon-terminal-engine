# Project State

## Current Focus
Expand and harden the command framework’s output parsing with robust, test-driven coverage: new JSON key and JSON path extractors, JSON array flattening, regex extraction (with group support), line counting, exit-code capture, and severity-line classification — all validated by 27 new unit tests for missing keys, malformed input, empty data, and edge cases.

## Completed
- [x] Add `OutputParser::JsonKey` extraction and tests for missing keys and malformed JSON.
- [x] Add `OutputParser::JsonPath` extraction and tests for missing/empty paths and null fallbacks.
- [x] Add `OutputParser::JsonArray` flattening (with optional `item_key`) and tests for malformed and non-array payloads.
- [x] Add `OutputParser::Regex` extraction with configurable group index and tests for no-match, invalid patterns, out-of-bounds groups, and full-match behavior.
- [x] Add `OutputParser::LineCount` and tests for empty, single-line, and newline-delimited input.
- [x] Add `OutputParser::ExitCode` capture and tests for zero and non-zero codes.
- [x] Add `OutputParser::SeverityLine` classification by pattern map and tests for empty and multi-pattern/multi-line input.
- [x] Deliver 27 targeted unit tests covering failure modes and boundary conditions for all new parsers.
