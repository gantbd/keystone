# Operating rigor

Keystone follows the same closure discipline as Chronos while remaining a mechanism
library:

- keep public vocabularies closed and versioned where persisted or exchanged;
- keep domain identity and errors out of shared crates;
- make bounds caller-visible and check arithmetic before allocation/publication;
- make refusal an identity transition;
- keep preparation separate from effect dispatch and publication;
- distinguish logical success from storage or delivery durability;
- preserve unknown effect knowledge until attributable evidence resolves it;
- avoid a universal repository, error, ID, configuration, or event type;
- require executable laws or conformance scenarios for every reusable claim;
- migrate existing engines incrementally while their own gates remain green.

Extraction requires two real consumers with matching semantics. Similar-looking
code with different failure meaning is not duplication.

