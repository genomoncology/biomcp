# CSpec fixture lacks a selectable BRAF document

The routine CSpec fixture lists BRAF resource IRIs, but selecting its listed
short version `1.0.0` fails with the generic ClinGen CSpec API error. The
fixture needs a receipt-backed BRAF document or must not advertise a selectable
manifest entry. This is outside ticket 1033's short-version selection change.
