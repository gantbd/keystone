# Local checkpoint V1

One register, one private directory, one lifetime OS writer lock. Key bytes are
stored in the record, never interpolated into paths. Creation requires a new
0700 leaf under an existing trusted parent; recovery requires a committed head.
Files are owned by the current UID, regular, single-link and 0600. Symlink leafs,
unexpected entries and resource replacement refuse. An uninitialized orphan is
not silently reset. Only a validated `.pending` stage may be removed at recovery.

`KSDIRCP\x01`, big-endian u64 revision, u32 key length, u64 value length, UTF-8 key
and opaque value form the closed record. Lengths are checked before allocation;
truncation, trailing bytes and unsupported versions refuse. Logical limits cover
key and value separately; the header adds 28 bytes. There is no checksum,
authentication, encryption, migration or external rollback witness in this layer.
Domain hosts must validate the contained schema and revision correspondence.

Publication allocates first, creates `.pending` exclusively, writes/synchronizes
the file, renames it over `checkpoint.bin`, then synchronizes the directory.
macOS additionally requests file `F_FULLFSYNC` without a weaker fallback.
Only completion of these barriers yields `Durable`. A publication-attempt error
is indeterminate; errors/uncertainty fence the store until explicit reopen.
Recovery locks, validates the whole winner, synchronizes it and the directory,
and returns strong load evidence. Revisions start at zero and never wrap.

This follows [file/directory synchronization precedent](https://www.man7.org/linux/man-pages/man2/fsync.2.html)
and [Apple's full-sync guidance](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fsync.2.html).
The trusted-parent/cooperative-same-UID profile is not hostile-directory or
network-filesystem hardening. Replacing a whole closed store with an older valid
copy is undetectable without an external witness. Hardware may not honor barriers;
no physical power-loss certification is claimed. Unsupported barriers fail closed.

Native focused tests cover CAS refusal identity, all record truncations, bounds,
private modes/aliases, lost/replaced resources, fixed-cardinality counters,
before/after-rename/sync faults, second-process writer exclusion and process exit
without destructors. These are process/fault tests, not simulated hardware crashes.
The recorded execution environment is Darwin 25.6.0 arm64; Linux and other Unix
filesystems have not been live-qualified by this delivery pass.
Hermes's concrete adapter additionally qualifies domain replay and cursor recovery;
Aegis's separate concrete tests qualify committed recovery fencing, allocation
frontiers, lost revocation receipts and abrupt process exit. Both consumers use
the same opaque revisioned replacement contract; neither assigns domain policy
to this adapter.

RED/golden signals: load/commit offers, outcomes and aggregate residence times.
USE: confirmed retained value bytes versus configured ceiling; one writer slot.
No queue, deadline, RSS, latency distribution, SLO or constant-time claim is made.
Full snapshots are O(record size) and barriers can block. Signals and `Debug`
exclude paths, keys, values and raw OS error messages; counters saturate visibly.

STRIDE/LINDDUN: private files and alias checks narrow disclosure/tampering;
exclusive lock/CAS narrow competing-writer confusion; bounded input narrows DoS;
uncertainty fences authority/publication, not historical attribution. Store access
does not authenticate callers or confer domain authority. Plaintext backups,
filesystem timestamps and same-UID attackers remain host/privacy responsibilities.
