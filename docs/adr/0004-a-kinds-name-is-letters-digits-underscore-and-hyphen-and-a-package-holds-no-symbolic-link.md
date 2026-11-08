---
number: 4
title: A kind's name is letters, digits, _ and -, and a package holds no symbolic link
status: accepted
date: 2026-11-05
decision-makers:
- Chuck Swanberg
---

# A kind's name is letters, digits, _ and -, and a package holds no symbolic link

Status: accepted by the user on 2026-11-08, on their card.

Deciders: the user, 2026-11-05, answering the two questions the construct stage of the change weir-kinds-reads-a-package-and-a-lock asked through `flume needs`: for a kind's name and its source's name, "Letters, digits, _ and - only; anything else is refused, naming it"; for links, "Refuse them, naming the file". With the second they noted, as the orchestrator relayed it, that a kind is a folder like a skill's, whose files may reference each other by Markdown relative links, and those stay allowed, since every file is inside the package and its hash.
Sources: `agentplot/blueprints:design/weir/requirements.md` 88 to 93, C.4, C.7 and S17, at 57a1ba9, and C.4, C.7 and S17 as amended at 361a599, which states these rulings; flume's record 0022 and record 0040; the change weir-kinds-reads-a-package-and-a-lock.

## Context and problem statement

weir-kinds is to read a kind's identity, `<source>/<name>@<version>`, a package with its content hash, and a lock (88 to 93). Two places in the statement left a choice the code had to make.

1. **What a source and a name may be.** C.4 said each is "a key the format allows under `kinds`", and `read-kinds` reads any TOML key, `kinds."a/b@1"` among them. So `<source>/<name>@<version>` could not be read back for every name, and flume's launch forms `review@2` and `acme/review` would be ambiguous.
2. **A link that stays inside a package.** C.4 refused a `prompt` or `standing` path through a link leading out, and hashed regular files alone. A prompt, or `kind.toml`, that is a link to a file inside the package could then be pointed at another file inside it without moving the hash, and the lock would no longer fix what runs (91; flume 155).

## Considered options

For the name:

1. Each is a TOML bare key, one or more ASCII letters, digits, `_` and `-`, and `read-kinds` refuses a kind's table or a table of settings under any other key, naming it (57), so every kind has an identity (88).
2. Any non-empty key holding no `/` or `@`.

For links:

1. Refuse a symbolic link in a package, naming the file.
2. Hash a link as its path and its target.
3. Build C.4 as written and accept the gap.

## Decision outcome

Option 1 of each, as the user decided.

- A kind's name and its source's name are each one or more ASCII letters, digits, `_` and `-`. A kind's table, or a table of settings, under any other key is refused by `read-kinds`, naming it (C.4, 57, 88).
- A package holds no symbolic link, to a file inside it or out. One that does is refused, naming the link's file (C.4, 89). A part may name another by a relative path, as a Markdown link does, since weir parses no part but `kind.toml` and every part lies inside the package and its hash (90).

### Consequences

- The statement states both at blueprints 361a599, on the branch `design/weir-a-kinds-name-and-a-package-without-links`: C.4's identity and package, C.7's kinds line naming package and lock cases, and S17 with a link refused.
- core.42 and core.43 rest on this record.
- A declaration that names a kind by a quoted key, `kinds."a b"`, reads today and is refused from this change on. No declaration of the family's tools names one.
- A package built with links, as a nix `symlinkJoin` lays one out, is refused; its builder copies the files in.
