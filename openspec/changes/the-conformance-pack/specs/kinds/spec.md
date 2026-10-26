# Spec Delta

## ADDED Requirements

### Requirement: The pack holds the kind format's cases

The conformance pack SHALL hold cases of the kinds shape at `pack/kinds/<case>/case.toml` (C.4; C.7): each a declaration, written in the case, and either the kinds it reads into, each with its version and its stages in order, each stage's name and its deliverable or that it is a conversation, and the sections returned unread; or the refusal, each field named with a phrase its reason holds; citing the clauses it exercises (67). The kinds crate's tests SHALL read every case and fail naming each one whose kinds or refusal differ (65, 87; core.36). The pack SHALL hold a case citing each of 51 to 58, among them flume's own declaration read twice the same (S10), an elaboration kind read with its section returned unread (S11), and a stage with two deliverables and a declaration with no stage, each refused naming the field (S12). A field the case format does not name SHALL fail the case's read, naming the case and the field.

#### Scenario: Every kinds case is answered as it says
- **WHEN** each kinds case of the pack is read by the kinds crate
- **THEN** every declaration reads into the kinds its case expects, or is refused naming the fields its case expects

#### Scenario: A stage with two deliverables (S12)
- **WHEN** a case's declaration gives a stage `deliverables = ["commits", "tasks"]`
- **THEN** the case expects the refusal naming that stage's `deliverables`, and the crate gives it

#### Scenario: A misspelt field of the case
- **WHEN** a kinds case names `kind` in place of `expect`
- **THEN** reading it fails, naming the case and the field
