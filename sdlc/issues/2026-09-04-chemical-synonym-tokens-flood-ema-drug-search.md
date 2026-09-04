# Chemical synonym tokens flood an exact EMA drug search with unrelated products

This command reported six U.S. matches and 279 European matches on `biomcp 0.9.0-dev.6` on 2026-09-04:

```bash
biomcp --json search drug eflornithine --region all --limit 2
```

The first European row was Vaniqa. That row carried the correct active-substance match. The second row was Prasugrel Viatris and carried `match_kind: broad_text`. The cached EMA row for Prasugrel contains no eflornithine in its product name, active substance, or therapeutic indication.

`src/entities/drug/mod.rs::build_ema_identity` passes `drug.brand_names` into the EMA identity. `merge_mychem_hits` fills that field from arbitrary DrugBank synonyms. The eflornithine values include chemical names such as `(RS)-2,5-diamino-2-(difluoromethyl)pentanoic acid`. `EmaDrugIdentity::search_tokens` splits every alias into tokens of three or more characters. `search_medicines` accepts a match when any token occurs in an active substance or indication. A common token such as `acid` therefore matches unrelated products.

## Recommended design

Keep drug identity aliases typed. Product names, active-substance names, and verified brand names can support exact normalized phrase matching. Chemical and systematic synonyms must not supply loose one-token EMA matches. Keep the special vaccine bridge on its vetted CVX aliases and require all distinctive tokens when that bridge needs token matching.

This design can lose some loose name-to-indication matches. Callers who want indication search should use an explicit indication filter instead of receiving those rows from a drug-name query.

## Done, observably

- The eflornithine European results include products whose name or active substance matches eflornithine.
- Prasugrel and other rows matched only by common chemical-name tokens disappear.
- Exact product, active-substance, and verified alias matches retain their current rank order.
- Vaccine alias tests keep passing without admitting one-token noise.
- Structured output explains the matched typed alias.
