# Provenance

The bookkeeping implementation was extracted from a local experimental
application's standalone streaming component. No game assets, saves, application
configuration, external engine or vendored dependency was copied.

Original `src/lib.rs` SHA-256:

`4ae09ca79e6647ab6167d8606414cf6b982df4e5c7ee31b9a65e042210c23d4d`

The extracted file is `src/bookkeeping.rs`. Its admission comment was clarified
to distinguish checking capacity from panic-safe reservation; the implementation
and its three existing tests were preserved. Standalone packaging, a public API
wrapper, documentation, a demo and consumer integration tests were added.

The original application files were not modified. This records source origin,
not a license grant or an independent legal authorship audit. Licensing is
pending; see [LICENSING.md](../LICENSING.md).
