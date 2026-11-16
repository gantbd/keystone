# Identity-guarded private Unix listener

Status: qualified shared physical mechanism through Aegis, Hermes and Moira.

`keystone-unix-listener::OwnedUnixListener` owns one narrow local law. Binding
refuses any existing leaf, publishes a Unix listener with mode `0600`, records
the resulting device/inode identity and removes the path on drop only while the
same identity still names a socket. Replacement files and replacement sockets
are never removed by the stale owner.

The consumer selects the parent directory and socket name and retains trust in
that parent. The mechanism does not authenticate a peer, inspect credentials,
configure deadlines, frame requests, route commands, report health, recover a
stale socket, acquire a process lock or define daemon lifecycle. It does not
claim hostile-parent safety, cross-platform support or remote transport.

Aegis, Hermes and both Moira local daemon profiles delegate listener publication
and cleanup to the same adapter. Their own Unix listener suites retain the
authentication, session, lost-reply, durable-reopen and domain behavior evidence.
`make qualify-unix-listener` runs those consumer suites from a workspace checkout.
