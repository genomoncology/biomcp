# Splitting the NCI field-name bundle

2026-09-03, BioMCP lead.

## What happened

An audit of the NCI reader on 2026-09-03 found five fields where the code asks the payload a question it cannot answer. All five were filed as one ticket, 1132, because they share a cause. That was wrong. One ticket per behavior is the rule, and this repository had already paid for breaking it hours earlier: ticket 1107 carried conditions and interventions together, the two halves turned out to have different causes, and the attempt refused because it could not tell which of two statements of correct behavior to follow. A bundle of five is that risk five times over.

So 1132 is superseded and held as a draft, and each behavior now stands alone.

## The mapping

| Behavior | BioData case | BioMCP ticket | Priority |
| --- | --- | --- | --- |
| Interventions are nested in the arms and never read | 18 | 1133 | 8 |
| Study type falls through and reports the primary purpose | 20 | 1134 | 8 |
| Age bounds sit in structured eligibility and are never read | 19 | 1135 | 7 |
| Eligibility text is read with `as_str()` on an object | 21 | 1136 | 7 |
| The stop reason is hardcoded absent | 22 | 1137 | 6 |
| Enrollment reads three names NCI does not send | 8 | 1119 | 5 |
| No key list in the code may name a field no provider sends | 17, code side | 1138 | 5 |

Two more sit beside them and are not part of the split. Ticket 1107 fixes NCI conditions, where the key is found and the elements are discarded. Ticket 1126 is 1138's sibling and guards fixtures rather than code.

## How the priorities were chosen

By how wrong the answer is that a user gets today, not by how much work the fix is.

1134 is highest of the group with 1133 because it returns a confident wrong value in a field that looks populated. A reader has no signal that "TREATMENT" answered a different question from the one asked. An empty field at least reads as empty.

1133 sits with it because an NCI trial reporting no intervention at all is a wrong answer to the most common question asked of a trial.

1135 and 1136 are eligibility. Both are absences rather than wrong values, and both block a real clinical question about whether a patient qualifies.

1137 affects only stopped trials, which is a subset, and its display half is already ticket 1097.

1119 is an absent number with the least clinical weight of the six. 1138 matches ticket 1126, its sibling.

## What did not survive the audit

Defect 7, the comma split inside a condition name. The branch is unreachable from every provider BioMCP supports, so no fixture can prove a correction. Draft 1118 is retired and its requirement now stands in full on ticket 1107. BioData retired case 7 the same day.

The original description of defect 8 was wrong about its cause, and so was the original interventions clause of case 4. Both were restated against measurement rather than dropped.

## The evidence gap this left open

`testdata/sources/nci_cts/search_melanoma.json` carries all 58 field names and is the file every one of these tickets rests on. It is classified `pending_verification` in `testdata/sources/capture-receipts.json` and has no provider receipt. The one receipted NCI capture was recorded with the response minimized to six fields and carries none of the names in question.

So for NCI the repository holds evidence without provenance and provenance without evidence. Ticket 1138 takes closing that as part of its work. Every ticket in the table says so in its own body, so no attempt discovers it late.
