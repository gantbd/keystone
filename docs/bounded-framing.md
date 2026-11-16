# Bounded request/reply framing

Status: qualified shared mechanism through Aegis, Hermes and Moira.

`keystone-framing` owns one mechanical connection law: a nonzero, bounded
four-byte big-endian length prefix is fully read and allocated before a consumer
command is decoded or dispatched. A finite request limit bounds one serving
call. Clean EOF between frames closes normally; a partial header or body is
pre-dispatch failure. Once the consumer dispatch returns an outcome, failure to
write or flush the complete reply is outcome-unknown.

The consumer supplies frame bounds, its decoder and encoder, the command
dispatcher, and mappings from mechanical frame refusals into its closed wire
error type. The mechanism owns no socket, timeout, authentication, session,
command, reply, retry, health, logging, daemon lifecycle or protocol version.

Aegis retains connection-session substitution checks. Hermes retains session
opening and ledger authorization. Moira retains workflow, attempt-lookup and
saga profile dispatch plus command-scoped operator authorization. Their existing
connection behavior suites exercise clean EOF, request bounds, truncated input,
wire refusal, service failure and reply-write uncertainty through the shared
loop.

From a workspace checkout, `make qualify-framing` runs the focused suites in all
three consumers. This qualification does not cover streaming, multiplexing,
deadlines, cancellation, backpressure, network transport or retry safety.
