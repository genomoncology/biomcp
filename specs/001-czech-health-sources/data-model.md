# Data Model: Czech Healthcare Data Sources

**Feature**: 001-czech-health-sources
**Date**: 2026-02-17

## Entities

### 1. Drug (Lek) - SUKL Module

```
Drug
├── sukl_code: str              # Unique SUKL identifier (e.g., "0000123")
├── name: str                   # Trade name (e.g., "Nurofen 400mg")
├── active_substances: list[ActiveSubstance]
│   ├── name: str               # e.g., "Ibuprofen"
│   └── strength: str           # e.g., "400 mg"
├── pharmaceutical_form: str    # e.g., "potahované tablety"
├── atc_code: str               # ATC classification (e.g., "M01AE01")
├── registration_number: str    # Marketing authorization number
├── mah: str                    # Marketing Authorization Holder
├── registration_valid_to: str  # Validity date (ISO 8601)
├── availability: AvailabilityStatus
│   ├── status: str             # "available" | "limited" | "unavailable"
│   ├── last_checked: str       # ISO 8601 datetime
│   └── note: str | None       # Optional note about availability
├── spc_url: str | None         # URL to SmPC document
├── pil_url: str | None         # URL to PIL document
└── source: str                 # Always "SUKL"
```

### 2. DrugSearchResult

```
DrugSearchResult
├── total: int                  # Total matches
├── page: int                   # Current page
├── page_size: int              # Results per page
└── results: list[DrugSummary]
    ├── sukl_code: str
    ├── name: str
    ├── active_substance: str   # Primary active substance
    ├── atc_code: str
    └── pharmaceutical_form: str
```

### 3. Diagnosis (Diagnoza) - MKN-10 Module

```
Diagnosis
├── code: str                   # MKN-10 code (e.g., "J06.9")
├── name_cs: str                # Czech name
├── name_en: str | None         # English name (if available)
├── definition: str | None      # Description/definition
├── hierarchy: DiagnosisHierarchy
│   ├── chapter: str            # e.g., "X" (Chapter code)
│   ├── chapter_name: str       # e.g., "Nemoci dýchací soustavy"
│   ├── block: str              # e.g., "J00-J06"
│   ├── block_name: str         # e.g., "Akutní infekce horních cest dýchacích"
│   └── category: str           # e.g., "J06"
├── includes: list[str]         # Inclusion terms
├── excludes: list[str]         # Exclusion terms
├── modifiers: list[Modifier]   # Applicable modifiers
│   ├── code: str
│   └── name: str
└── source: str                 # Always "UZIS/MKN-10"
```

### 4. DiagnosisCategory

```
DiagnosisCategory
├── code: str                   # Category/block code
├── name: str                   # Category name
├── type: str                   # "chapter" | "block" | "category"
├── children: list[DiagnosisCategory]  # Subcategories
└── parent_code: str | None     # Parent category code
```

### 5. HealthcareProvider (Poskytovatel) - NRPZS Module

```
HealthcareProvider
├── provider_id: str            # NRPZS identifier
├── name: str                   # Organization/provider name
├── legal_form: str             # e.g., "s.r.o.", "příspěvková organizace"
├── ico: str                    # Company identification number
├── address: Address
│   ├── street: str
│   ├── city: str
│   ├── postal_code: str
│   └── region: str             # Kraj
├── specialties: list[str]      # Medical specialties
├── care_types: list[str]       # Types of care provided
├── workplaces: list[Workplace]
│   ├── workplace_id: str
│   ├── name: str
│   ├── address: Address
│   ├── specialties: list[str]
│   └── contact: Contact
│       ├── phone: str | None
│       ├── email: str | None
│       └── website: str | None
├── registration_number: str    # Registration in NRPZS
└── source: str                 # Always "NRPZS"
```

### 6. HealthProcedure (Zdravotni vykon) - SZV Module

```
HealthProcedure
├── code: str                   # Procedure code (e.g., "09513")
├── name: str                   # Procedure name
├── category: str               # Category code
├── category_name: str          # Category name
├── point_value: int            # Point value for reimbursement
├── time_minutes: int | None    # Estimated time in minutes
├── frequency_limit: str | None # Frequency limitation
├── specialty_codes: list[str]  # Required specialty codes
├── material_requirements: str | None  # Material needs
├── notes: str | None           # Additional notes
└── source: str                 # Always "MZCR/SZV"
```

### 7. CodebookEntry (Polozka ciselniku) - VZP Module

```
CodebookEntry
├── codebook_type: str          # Type of codebook
├── code: str                   # Entry code
├── name: str                   # Entry name/description
├── description: str | None     # Detailed description
├── valid_from: str | None      # Validity start (ISO 8601)
├── valid_to: str | None        # Validity end (ISO 8601)
├── rules: list[str]            # Applicable rules/conditions
├── related_codes: list[str]    # Cross-references
└── source: str                 # Always "VZP"
```

## Relationships

```
Drug (SUKL)
  └─ atc_code ──> can map to ──> CodebookEntry (VZP drug codes)

Diagnosis (MKN-10)
  └─ code ──> referenced by ──> HealthProcedure (SZV diagnosis requirements)
  └─ code ──> maps to ──> CodebookEntry (VZP diagnosis codes)

HealthProcedure (SZV)
  └─ specialty_codes ──> maps to ──> HealthcareProvider.specialties (NRPZS)
  └─ code ──> referenced by ──> CodebookEntry (VZP procedure codes)
```

## Validation Rules

- **SUKL code**: Numeric string, 7 digits (zero-padded)
- **MKN-10 code**: Format `[A-Z]\d{2}(\.\d{1,2})?` (e.g., "J06.9", "C34")
- **NRPZS provider_id**: Numeric identifier from registry
- **SZV procedure code**: 5-digit numeric string
- **VZP codebook code**: Format varies by codebook type

## State Transitions

No state transitions apply. All entities are read-only snapshots from
their respective registries. The `availability.status` field for Drug
reflects the current state from SUKL but is not managed by CzechMedMCP.
